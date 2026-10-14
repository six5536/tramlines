//! Which block a position gets, and how much of a run it takes. A position
//! is tried against a block and rolled back where it refuses it.

use std::borrow::Cow;

use super::{
    At, Checker,
    blocks::check_plain,
    divide::{extent, extents},
    short,
};
use crate::{
    Finding, Severity,
    grammar::{Block as Rule, States},
    markdown::{Block, BlockKind},
};

/// The blocks a walk holds, a list of its own from start to end, since a
/// division cuts a block in two while the walk is under way. A block is
/// borrowed until a position divides it, and only the divided block is
/// cloned.
pub(super) type Docs<'a> = Vec<Cow<'a, Block>>;

/// The length of everything a check writes, taken before a trial.
#[derive(Debug, Clone, Copy)]
pub(super) struct Savepoint {
    findings: usize,
    captures: usize,
    repairs: usize,
    gates: usize,
}

impl Checker<'_> {
    /// Where the output stands now.
    pub(super) fn save(&self) -> Savepoint {
        Savepoint {
            findings: self.out.findings.len(),
            captures: self.out.captures.len(),
            repairs: self.out.repairs.len(),
            gates: self.out.gates.len(),
        }
    }

    /// Whether an error was written since `at`. A warning is a document
    /// that passed, so it is not one.
    // sokf-implements: TLMD_65
    pub(super) fn failed(&self, at: &Savepoint) -> bool {
        self.out.findings[at.findings..]
            .iter()
            .any(|f| f.severity == Severity::Error)
    }

    /// Drop everything written since `at`, yielding the findings dropped.
    /// Every list grows and none shrinks, so savepoints nest.
    // sokf-implements: TLMD_64
    pub(super) fn rollback(&mut self, at: &Savepoint) -> Vec<Finding> {
        self.out.captures.truncate(at.captures);
        self.out.repairs.truncate(at.repairs);
        self.out.gates.truncate(at.gates);
        self.out.findings.split_off(at.findings)
    }
}

/// The inner blocks of a group that `states` admit, in order.
fn admitted<'r>(rule: &'r Rule, states: &States) -> impl Iterator<Item = &'r Rule> {
    rule.blocks
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(move |b| b.common.only.admits(states))
}

/// Whether `block` opens `rule`: a group opens with its first admitted
/// inner block.
fn matches_rule(block: &Block, rule: &Rule, states: &States) -> bool {
    opens(rule, states) == Some(block.kind)
}

/// The kind of block `rule` opens with: a group's first admitted inner
/// block's.
fn opens(rule: &Rule, states: &States) -> Option<BlockKind> {
    if rule.kind != BlockKind::Group {
        return Some(rule.kind);
    }
    admitted(rule, states)
        .next()
        .and_then(|first| opens(first, states))
}

/// The kinds of block that the positions of `rules` can open.
fn opened(rules: &[&Rule], states: &States) -> Vec<BlockKind> {
    rules.iter().filter_map(|r| opens(r, states)).collect()
}

/// Whether `block` fits a group's inner block after its first: a
/// repetition that is missing its opening blocks, such as a diagram with
/// no caption before it.
fn joins(block: &Block, rule: &Rule, states: &States) -> bool {
    rule.kind == BlockKind::Group
        && admitted(rule, states)
            .skip(1)
            .any(|b| matches_rule(block, b, states))
}

/// The units at the end of the run at `docs[di]` that a later group needs
/// for its opening: the block after the run fits that group only after its
/// opening, and no later position takes it by itself. The group opens with
/// the run's kind, and its opening needs at least one unit.
// sokf-implements: TLMD_193
fn reserve_for(rules: &[&Rule], ri: usize, docs: &Docs<'_>, di: usize, states: &States) -> usize {
    let kind = rules[ri].kind;
    let Some(next) = docs.get(di + 1) else {
        return 0;
    };
    if !is_run(kind) || docs[di].kind != kind {
        return 0;
    }
    let after = &rules[ri + 1..];
    if after.iter().any(|r| matches_rule(next, r, states)) {
        return 0;
    }
    after
        .iter()
        .find(|r| opens(r, states) == Some(kind) && joins(next, r, states))
        .and_then(|g| admitted(g, states).next())
        .map_or(0, |opening| opening.min.max(1))
}

/// Whether a position of this kind claims its candidate whatever it
/// reports: the schema states the block must be there, and once.
fn firm(rule: &Rule) -> bool {
    !rule.optional && !rule.repeat && rule.kind != BlockKind::Group
}

/// The kinds a block can hold more of than its author wrote as one: the
/// parser gathers consecutive paragraphs into one prose block, and two
/// lists of one marker are one list. Two quotes stay two blocks.
// sokf-implements: TLMD_76
pub(super) fn is_run(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::Prose | BlockKind::BulletList | BlockKind::NumberedList
    )
}

/// A body's blocks against declared blocks. `None` means one prose block
/// of one or more paragraphs; `[]` an empty body.
// sokf-implements: TLMD_39
// sokf-implements: TLMD_40
pub(super) fn check_blocks(
    c: &mut Checker<'_>,
    rules: Option<&[Rule]>,
    docs: &[&Block],
    ordered: bool,
    section: Option<usize>,
    at: usize,
) {
    let prose_only;
    let rules: Vec<&Rule> = match rules {
        Some(r) => r
            .iter()
            .filter(|b| b.common.only.admits(c.states))
            .collect(),
        None => {
            prose_only = default_prose();
            vec![&prose_only]
        }
    };
    let mut docs: Docs<'_> = docs.iter().map(|b| Cow::Borrowed(*b)).collect();
    if ordered {
        let consumed = match_sequence(c, &rules, &mut docs, 0, section, at, None);
        // Every block a position refused was either claimed by a later
        // position or set aside with its reason, so what is left here is
        // a block too many.
        // sokf-implements: TLMD_39
        // sokf-implements: TLMD_67
        for b in &docs[consumed.min(docs.len())..] {
            unclaimed(c, b, matches!(b, Cow::Owned(_)));
        }
    } else {
        match_free(c, &rules, &mut docs, section, at);
    }
}

/// Blocks in any order: every admitting rule is tried, and where none is
/// clean the first unused rule of that kind claims the block and reports.
// sokf-implements: TLMD_62
fn match_free(
    c: &mut Checker<'_>,
    rules: &[&Rule],
    docs: &mut Docs<'_>,
    section: Option<usize>,
    at: usize,
) {
    let mut used = vec![0usize; rules.len()];
    let mut di = 0;
    while di < docs.len() {
        let free: Vec<usize> = (0..rules.len())
            .filter(|&ri| {
                matches_rule(&docs[di], rules[ri], c.states) && (used[ri] == 0 || rules[ri].repeat)
            })
            .collect();
        let mut taken = None;
        let mut refusal = None;
        c.later = opened(rules, c.states);
        for ri in &free {
            // Dividing a run needs an order to divide it along, so the
            // free walk takes whole blocks (LANG-5_AC-4).
            match claim(c, docs, di, rules[*ri], section, false, false) {
                // A group that took no block leaves it to the rules after
                // it, and to the report of a block nobody claimed.
                Ok(0) => {}
                Ok(n) => {
                    taken = Some((*ri, n));
                    break;
                }
                Err(dropped) => refusal = refusal.or(Some(dropped)),
            }
        }
        // Nothing clean: where a rule refused the block, the first of them
        // takes it, so a malformed block keeps its own finding.
        let taken = taken.or_else(|| {
            let dropped = refusal.take()?;
            let ri = *free.first()?;
            c.out.findings.extend(dropped);
            Some((ri, 1))
        });
        match taken {
            Some((ri, n)) => {
                used[ri] += 1;
                di += n;
            }
            None => {
                unclaimed(c, &docs[di], matches!(docs[di], Cow::Owned(_)));
                di += 1;
            }
        }
    }
    for (ri, rule) in rules.iter().enumerate() {
        if used[ri] == 0 && !rule.optional {
            let at_rule = at_block(&c.at, rule);
            c.located(at_rule, |c| {
                // sokf-implements: TLMD_152
                c.emit(
                    &rule.common,
                    Some(at),
                    "TLMD_152",
                    format!(
                        "{} block is missing",
                        rule.kind.spoken_in(&c.host.dialect())
                    ),
                );
            });
        }
    }
}

/// Report a block that no position claims. The rest of a run
/// that a position divided, `left_over` (only a division cuts a block, so
/// it is the owned one), is units that no position took, not a block of
/// its own, so it is reported at its first unit.
// sokf-implements: TLMD_50
fn unclaimed(c: &mut Checker<'_>, b: &Block, left_over: bool) {
    let kind = b.kind.name_in(&c.host.dialect());
    match b.units.first().filter(|_| left_over) {
        Some(first) => c.error(
            Some(first.line),
            "TLMD_50",
            format!(
                "`{}` remains after the last {kind} position that takes the run",
                short(&first.text)
            ),
        ),
        _ => c.error(Some(b.line), "TLMD_39", format!("unexpected {kind} block")),
    }
}

fn default_prose() -> Rule {
    Rule {
        common: Default::default(),
        kind: BlockKind::Prose,
        optional: false,
        repeat: false,
        min: 1,
        max: None,
        each: None,
        sequence: None,
        rules: None,
        units_open: false,
        lang: None,
        fence: 3,
        columns: None,
        columns_open: false,
        source_pattern: None,
        inline_html: false,
        blocks: None,
        rule_id: None,
        examples: Vec::new(),
        example_list: false,
        guidance: None,
        pos: 1,
    }
}

/// The number of blocks consumed by the positions of `rules` against `docs` from `start`, in order.
/// `wraps` is the kind that both opens and closes `rules` where they are
/// one repetition of a group that another repetition can follow: the next
/// repetition is then one more position of that kind, which takes what a
/// run leaves at the end of this one.
#[allow(clippy::too_many_arguments)]
fn match_sequence(
    c: &mut Checker<'_>,
    rules: &[&Rule],
    docs: &mut Docs<'_>,
    start: usize,
    section: Option<usize>,
    at: usize,
    wraps: Option<BlockKind>,
) -> usize {
    // Dividing a run is the claim decision taken over units, here in the
    // matcher: the parser never reads a schema and its output is cached for
    // each file. The next repetition of a group is a position after the
    // last of this one, as a quantifier is unrolled.
    // A group counts as one more position of the kind that opens it,
    // whatever closes it, because its opening is what takes the end of a
    // run.
    // sokf-implements: TLMD_78
    let group_opens: Vec<BlockKind> = rules
        .iter()
        .filter(|r| r.kind == BlockKind::Group)
        .filter_map(|r| opens(r, c.states))
        .collect();
    let divides = |kind: BlockKind| {
        let declared = rules.iter().filter(|r| r.kind == kind).count();
        declared >= 2 || wraps == Some(kind) || group_opens.contains(&kind)
    };
    let mut di = start;
    for (ri, rule) in rules.iter().enumerate() {
        let mut count = 0;
        let mut refusal: Option<Vec<Finding>> = None;
        let mut answered = false;
        let mut stray = false;
        // The kinds the positions after this one open: a block of one of
        // them waits for its position.
        let later = opened(&rules[ri + 1..], c.states);
        while di < docs.len() {
            // A block a later position declares waits for it; one only a
            // group's inner block fits is a repetition missing its opening.
            // sokf-implements: TLMD_72
            let fits = matches_rule(&docs[di], rule, c.states)
                || (joins(&docs[di], rule, c.states) && !later.contains(&docs[di].kind));
            if !fits {
                // A block of another kind. Where a position after this one
                // declares it, the block waits for that position.
                if later.contains(&docs[di].kind) {
                    // This position's own block stands later, where no
                    // position after this one wants it: the two are in the
                    // wrong order, which is one fault. The block is checked
                    // against this position and leaves the walk.
                    // sokf-implements: TLMD_169
                    if count == 0
                        && !rule.optional
                        && rule.kind != BlockKind::Group
                        && let Some(j) = (di + 1..docs.len()).find(|&j| {
                            matches_rule(&docs[j], rule, c.states) && !later.contains(&docs[j].kind)
                        })
                    {
                        let block = docs.remove(j);
                        c.located(at_block(&c.at, rule), |c| {
                            c.emit(
                                &rule.common,
                                Some(block.line),
                                "TLMD_169",
                                format!(
                                    "the {} block is out of place; the blocks are in the order the schema lists them",
                                    block.kind.name_in(&c.host.dialect())
                                ),
                            );
                        });
                        check_plain(c, &block, rule, section);
                        count += 1;
                    }
                    break;
                }
                // Nobody declares it, and this position's own block comes
                // later: report the block in the way, as one no position
                // claims, and step over it.
                // sokf-implements: TLMD_39
                // sokf-implements: TLMD_68
                if count == 0
                    && docs[di + 1..]
                        .iter()
                        .any(|b| matches_rule(b, rule, c.states))
                {
                    unclaimed(c, &docs[di], matches!(docs[di], Cow::Owned(_)));
                    di += 1;
                    continue;
                }
                // Nobody declares it, and nothing later satisfies this
                // position: the report below names both kinds and claims
                // the block.
                // sokf-implements: TLMD_69
                stray = count == 0;
                break;
            }
            c.later = later.clone();
            c.reserve = reserve_for(rules, ri, docs, di, c.states);
            let claimed = claim(c, docs, di, rule, section, firm(rule), divides(rule.kind));
            c.reserve = 0;
            match claimed {
                // A group whose `min` admits no repetition is satisfied by
                // none, and takes no block. Counting it as one would eat
                // the block it did not take.
                Ok(0) => {
                    count += 1;
                    break;
                }
                Ok(n) => {
                    di += n;
                    count += 1;
                }
                Err(dropped) => {
                    // A released position advances nothing, so the block
                    // reaches the position after it.
                    // sokf-implements: TLMD_63
                    if later.contains(&docs[di].kind) {
                        refusal = Some(dropped);
                        break;
                    }
                    // No position after this one can claim the block, so
                    // it is set aside here with the reason it was refused.
                    // The positions after this one then see the blocks
                    // beyond it rather than stalling behind it.
                    c.out.findings.extend(dropped);
                    answered = true;
                    di += 1;
                    continue;
                }
            }
            // A group is a repeating sequence of blocks, so it takes every
            // repetition it can in one claim. Claiming it again could only
            // repeat the repetition that stopped it.
            if !rule.repeat || rule.kind == BlockKind::Group {
                break;
            }
        }
        if count == 0 && !rule.optional && !answered {
            match refusal {
                // The position refused a block of its own kind: why it
                // refused says more than the kind it declined.
                // sokf-implements: TLMD_66
                Some(dropped) => c.out.findings.extend(dropped),
                None => {
                    let (line, found) = match docs.get(di) {
                        Some(b) => (
                            Some(b.line),
                            format!("; found {} block", b.kind.spoken_in(&c.host.dialect())),
                        ),
                        None => (Some(at), String::new()),
                    };
                    let at_rule = at_block(&c.at, rule);
                    // sokf-implements: TLMD_152
                    c.located(at_rule, |c| {
                        c.emit(
                            &rule.common,
                            line,
                            if found.is_empty() {
                                "TLMD_152"
                            } else {
                                "TLMD_69"
                            },
                            format!(
                                "expected {} block{found}",
                                rule.kind.spoken_in(&c.host.dialect())
                            ),
                        );
                    });
                    // The failure above named this block, so the sweep of
                    // what nobody claimed leaves it alone.
                    // sokf-implements: TLMD_70
                    if stray {
                        di += 1;
                    }
                }
            }
        }
    }
    di - start
}

/// The locator narrowed to one block rule: its `rule-id`, else where it
/// sits among the blocks declared beside it.
// sokf-implements: TL_73
fn at_block(at: &At, rule: &Rule) -> At {
    let mut blocks = at.blocks.clone();
    blocks.push(rule.pos);
    At {
        region: at.region.clone(),
        blocks,
        rule_id: rule.rule_id.clone(),
        named: None,
        line: Some(rule.common.line),
    }
}

/// Claim `docs[di]` for `rule`: the blocks consumed, or the findings of
/// the refusal. A firm position claims whatever it reports.
// sokf-implements: TLMD_59
// sokf-implements: TLMD_60
fn claim(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    firm: bool,
    divides: bool,
) -> Result<usize, Vec<Finding>> {
    c.located(at_block(&c.at, rule), |c| {
        claim_located(c, docs, di, rule, section, firm, divides)
    })
}

/// As `claim`, with the locator narrowed to `rule`.
fn claim_located(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    firm: bool,
    divides: bool,
) -> Result<usize, Vec<Finding>> {
    if let Some(bounds) = extents(&docs[di], rule, divides, c.reserve) {
        return extent(c, docs, di, rule, section, firm, bounds);
    }
    let at = c.save();
    // A group's inner positions may divide a run, which is the one thing
    // a trial writes outside the checker's output.
    let blocks = (rule.kind == BlockKind::Group).then(|| docs.clone());
    let n = check_block(c, docs, di, rule, section);
    // A group that took blocks owns them and what it reports of them: the
    // repetitions it gave up were given back before it returned.
    // sokf-implements: TLMD_73
    let owns = rule.kind == BlockKind::Group && n > 0;
    if firm || owns || !c.failed(&at) {
        return Ok(n);
    }
    let dropped = c.rollback(&at);
    if let Some(blocks) = blocks {
        *docs = blocks;
    }
    Err(dropped)
}

/// Check one block against its rule; the number of blocks consumed. A
/// group walks its inner rules over the blocks from `di`.
fn check_block(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
) -> usize {
    if rule.kind != BlockKind::Group {
        check_plain(c, &docs[di], rule, section);
        return 1;
    }
    let line = docs[di].line;
    c.gate(&rule.common, line);
    let inner: Vec<&Rule> = admitted(rule, c.states).collect();
    // The group's `rule-id` names what its inner blocks capture.
    let outer = c.group_rule.take();
    c.group_rule = rule.rule_id.clone().or(outer.clone());
    // What the positions after the group take, read before the inner walk
    // sets its own.
    let later = std::mem::take(&mut c.later);
    let mut reps = 0;
    let mut pos = di;
    while pos < docs.len() {
        let opening = inner
            .first()
            .is_some_and(|first| matches_rule(&docs[pos], first, c.states));
        let claimed = later.contains(&docs[pos].kind);
        if !opening && (claimed || !joins(&docs[pos], rule, c.states)) {
            break;
        }
        let at = c.save();
        let blocks = docs.clone();
        let start = docs[pos].line;
        let first = start..=docs[pos].end_line;
        let wraps = (rule.max.is_none_or(|max| reps + 1 < max)
            && inner.first().map(|r| r.kind) == inner.last().map(|r| r.kind))
        .then(|| inner.first().map(|r| r.kind))
        .flatten();
        let n = match_sequence(c, &inner, docs, pos, section, start, wraps);
        if n > 0 && !c.failed(&at) {
            pos += n;
            reps += 1;
            continue;
        }
        // A repetition the group needs to reach its `min`, one whose blocks
        // no position after it takes, and one whose opening block the group
        // accepted, are kept with the reason they failed: the opening says
        // whose they are.
        // sokf-implements: TLMD_71
        let owned = opening
            && !c.out.findings[at.findings..].iter().any(|f| {
                f.severity == Severity::Error && f.line.is_some_and(|l| first.contains(&l))
            });
        if n > 0 && (reps < rule.min || !claimed || owned) {
            pos += n;
            reps += 1;
            continue;
        }
        // A repetition that cannot be completed is not a repetition, and
        // a run one of its positions divided is put back together: the
        // repetition after it would otherwise read the pieces.
        // sokf-implements: TLMD_61
        c.rollback(&at);
        *docs = blocks;
        break;
    }
    c.later = later;
    c.group_rule = outer;
    super::blocks::check_count(c, rule, reps, line);
    pos - di
}
