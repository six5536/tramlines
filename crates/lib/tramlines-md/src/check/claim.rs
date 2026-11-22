//! Which block a position gets, and how much of a run it takes. A position
//! is tried against a block and rolled back where it refuses it.

use std::borrow::Cow;

pub(super) use super::trial::{Dropped, is_text_error};
use super::{
    At, Checker,
    blocks::{check_plain, has_shape},
    divide::{self, extent, extents},
    free, group, short,
};
use crate::{
    grammar::{Block as Rule, RuleCommon, Unit as UnitRule},
    markdown::{Block, BlockKind, Unit},
};

/// The blocks a walk holds, a list of its own from start to end, since a
/// division cuts a block in two while the walk is under way. A block is
/// borrowed until a position divides it, and only the divided block is
/// cloned.
pub(super) type Docs<'a> = Vec<Cow<'a, Block>>;

/// What a claim knows of the walk around the position that it tries.
#[derive(Debug, Clone, Copy)]
pub(super) struct Around<'r> {
    /// The positions after the one tried, in order, which a group tries on
    /// the opening of a failed repetition (`TLMD_71`).
    pub(super) later: &'r [&'r Rule],
    /// The kinds of block that the positions after it open, those after
    /// the sequence that holds it among them: a block of one of them waits
    /// for its position.
    pub(super) kinds: &'r [BlockKind],
    /// The repetitions that a group already holds, from earlier claims of
    /// a walk in any order, which its `min` counts (`TLMD_62`).
    pub(super) held: usize,
    /// Whether the walk fixes the order of the blocks, so that a run may be
    /// divided (`TLMD_77`).
    pub(super) ordered: bool,
    /// The kinds of a run that the walk divides, as its claims of the
    /// later positions read them (`TLMD_78`).
    pub(super) dividing: &'r [BlockKind],
}

impl<'a> Checker<'a> {
    /// The inner blocks of a group that the states the document could be
    /// in admit, in order.
    pub(super) fn admitted<'r>(
        &self,
        rule: &'r Rule,
    ) -> impl Iterator<Item = &'r Rule> + use<'r, '_, 'a> {
        let possible = &self.possible;
        rule.blocks
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter(move |b| b.common.only.admits(possible))
    }

    /// Whether `block` opens `rule`, as [`opens`] reads it.
    pub(super) fn matches_rule(&self, block: &Block, rule: &Rule) -> bool {
        opens_with(rule, block.kind, self)
    }

    /// Whether `block` fits a group's inner block after its first: a
    /// repetition that is missing its opening blocks, such as a diagram
    /// with no caption before it.
    pub(super) fn joins(&self, block: &Block, rule: &Rule) -> bool {
        rule.kind == BlockKind::Group
            && self
                .admitted(rule)
                .skip(1)
                .any(|b| self.matches_rule(block, b))
    }
}

/// Whether `rule` can open with a block of `kind`, as [`opens`] reads it.
fn opens_with(rule: &Rule, kind: BlockKind, c: &Checker<'_>) -> bool {
    opening(rule, kind, c).is_some()
}

/// The position that opens `rule` with a block of `kind`: `rule` itself
/// where it is no group and of that kind, else the position that an inner
/// block that [`openers`] gives opens with, followed into a group that
/// opens a group.
fn opening<'r>(rule: &'r Rule, kind: BlockKind, c: &Checker<'_>) -> Option<&'r Rule> {
    if rule.kind != BlockKind::Group {
        return (rule.kind == kind).then_some(rule);
    }
    openers(rule, c).find_map(|inner| opening(inner, kind, c))
}

/// The inner blocks that a group `rule` can open with: its first admitted
/// one. Where the document's state is unknown, an inner block that its own
/// states do not admit may be left out, so the group can also open with
/// the inner blocks after it, up to the first that they admit (`TLMD_230`).
/// A block that is no group has none.
fn openers<'r>(rule: &'r Rule, c: &Checker<'_>) -> impl Iterator<Item = &'r Rule> {
    let states = c.states;
    let mut done = rule.kind != BlockKind::Group;
    c.admitted(rule).take_while(move |inner| {
        let take = !done;
        done = done || inner.common.only.admits(states);
        take
    })
}

/// The kinds of block `rule` opens with: its own, or those of the inner
/// blocks that [`openers`] gives for a group.
fn opens(rule: &Rule, c: &Checker<'_>) -> Vec<BlockKind> {
    if rule.kind != BlockKind::Group {
        return vec![rule.kind];
    }
    openers(rule, c).flat_map(|inner| opens(inner, c)).collect()
}

/// The kinds of block that the positions of `rules` can open.
pub(super) fn opened(rules: &[&Rule], c: &Checker<'_>) -> Vec<BlockKind> {
    rules.iter().flat_map(|r| opens(r, c)).collect()
}

/// The units at the end of the run at `docs[di]` that a later group takes
/// for its opening: the block after the run fits that group only after its
/// opening, and no later position takes it by itself. The group opens with
/// the run's kind, and the units are those at the end of the run that have
/// the shape of its opening position, none where the opening has no shape:
/// a unit's own text says which position it belongs to, and no count does.
// sokf-implements: TLMD_193
fn reserve_for(c: &Checker<'_>, rules: &[&Rule], ri: usize, docs: &Docs<'_>, di: usize) -> usize {
    let kind = rules[ri].kind;
    let Some(next) = docs.get(di + 1) else {
        return 0;
    };
    if !is_run(kind) || docs[di].kind != kind {
        return 0;
    }
    let after = &rules[ri + 1..];
    if after.iter().any(|r| c.matches_rule(next, r)) {
        return 0;
    }
    after
        .iter()
        .find(|r| opens_with(r, kind, c) && c.joins(next, r))
        .and_then(|g| opening(g, kind, c))
        .map_or(0, |opening| {
            docs[di]
                .units
                .iter()
                .rev()
                .take_while(|u| shaped(c, std::slice::from_ref(u), opening) == Some(true))
                .count()
        })
}

/// Whether a position of this kind claims its candidate whatever it
/// reports: the schema states the block must be there, in the states of
/// the document, and once.
pub(super) fn firm(rule: &Rule, c: &Checker<'_>) -> bool {
    !c.block_optional(rule) && !rule.repeat && rule.kind != BlockKind::Group
}

/// The kinds a block can hold more of than its author wrote as one: the
/// parser gathers consecutive paragraphs into one prose block, and two
/// lists of one marker are one list. Two quotes stay two blocks.
// sokf-implements: TLMD_76
pub(crate) fn is_run(kind: BlockKind) -> bool {
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
    let rules: Vec<&Rule> = if let Some(r) = rules {
        r.iter()
            .filter(|b| b.common.only.admits(&c.possible))
            .collect()
    } else {
        prose_only = default_prose();
        vec![&prose_only]
    };
    let mut docs: Docs<'_> = docs.iter().map(|b| Cow::Borrowed(*b)).collect();
    if ordered {
        let consumed = c.match_sequence(&rules, &mut docs, 0, section, at, &Outside::default());
        // Every block a position refused was either claimed by a later
        // position or set aside with its reason, so what is left here is
        // a block too many.
        // sokf-implements: TLMD_39
        // sokf-implements: TLMD_67
        for b in &docs[consumed.min(docs.len())..] {
            unclaimed(c, b, matches!(b, Cow::Owned(_)));
        }
    } else {
        free::match_free(c, &rules, &mut docs, section, at);
    }
}

/// Report a block that no position claims. The rest of a run
/// that a position divided, `left_over` (only a division cuts a block, so
/// it is the owned one), is units that no position took, not a block of
/// its own, so it is reported at its first unit.
// sokf-implements: TLMD_50
pub(super) fn unclaimed(c: &mut Checker<'_>, b: &Block, left_over: bool) {
    // No rule takes the units of the block, so each breaks its rule.
    c.broke_block(b);
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
        common: RuleCommon::default(),
        kind: BlockKind::Prose,
        optional: None,
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
        example_columns: None,
        source_pattern: None,
        inline_html: false,
        label: None,
        blocks: None,
        rule_id: None,
        examples: Vec::new(),
        example_list: false,
        guidance: None,
        pos: 1,
    }
}

/// What lies after a sequence of positions that are one repetition of a
/// group, which the walk of the sequence reads as positions of its own.
#[derive(Debug, Default)]
pub(super) struct Outside<'r> {
    /// The kind that both opens and closes the sequence where another
    /// repetition can follow: the next repetition is then one more
    /// position of that kind, which takes what a run leaves at the end of
    /// this one.
    pub(super) wraps: Option<BlockKind>,
    /// The positions after the group, where the walk around it fixes the
    /// block order: each counts as one more position of the kind that it
    /// opens (`TLMD_78`), a block of that kind waits for it, and a group
    /// inside tries it before it keeps a needed repetition (`TLMD_71`).
    pub(super) later: &'r [&'r Rule],
    /// The kinds that those positions open.
    pub(super) after: Vec<BlockKind>,
}

/// What a claim took: the blocks, and, for a group, its repetitions.
#[derive(Debug, Clone, Copy)]
pub(super) struct Claimed {
    pub(super) blocks: usize,
    pub(super) repetitions: usize,
}

impl Checker<'_> {
    /// The number of blocks consumed by the positions of `rules` against `docs` from `start`, in order,
    /// with what lies `outside` them where they are one repetition of a group.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass of the check, a split of which is a refactor of its own"
    )]
    pub(super) fn match_sequence(
        &mut self,
        rules: &[&Rule],
        docs: &mut Docs<'_>,
        start: usize,
        section: Option<usize>,
        at: usize,
        outside: &Outside<'_>,
    ) -> usize {
        let c = self;
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
            .flat_map(|r| opens(r, c))
            .collect();
        let divides = |kind: BlockKind| {
            let declared = rules.iter().filter(|r| r.kind == kind).count();
            declared >= 2
                || outside.wraps == Some(kind)
                || group_opens.contains(&kind)
                || outside.after.contains(&kind)
        };
        // The kinds of a run that this walk divides, which a group's trial
        // of the positions after it reads as this walk would.
        let mut dividing: Vec<BlockKind> = rules
            .iter()
            .map(|r| r.kind)
            .chain(outside.after.iter().copied())
            .collect();
        dividing.retain(|&k| divides(k));
        let mut di = start;
        // The optional positions, no group among them, that took no block: a
        // block that fails a later position may be one of theirs out of place.
        let mut skipped: Vec<usize> = Vec::new();
        for (ri, rule) in rules.iter().enumerate() {
            let mut count = 0;
            let mut refusal: Option<Dropped> = None;
            let mut answered = false;
            let mut stray = false;
            // The kinds the positions after this one open, those after the
            // group that the sequence repeats among them: a block of one of
            // them waits for its position.
            let mut later = opened(&rules[ri + 1..], c);
            later.extend(outside.after.iter().copied());
            let later_rules: Vec<&Rule> = rules[ri + 1..]
                .iter()
                .chain(outside.later)
                .copied()
                .collect();
            while di < docs.len() {
                // A block a later position declares waits for it; one only a
                // group's inner block fits is a repetition missing its opening.
                // sokf-implements: TLMD_72
                let fits = c.matches_rule(&docs[di], rule)
                    || (c.joins(&docs[di], rule) && !later.contains(&docs[di].kind));
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
                            && !c.block_optional(rule)
                            && rule.kind != BlockKind::Group
                            && let Some(j) = (di + 1..docs.len()).find(|&j| {
                                c.matches_rule(&docs[j], rule) && !later.contains(&docs[j].kind)
                            })
                        {
                            let block = docs.remove(j);
                            // The block's findings name its position, as a
                            // claim's do (TL_73).
                            // sokf-implements: TL_73
                            c.located(at_block(&c.at, rule), |c| {
                                let message = misplaced(c, &block);
                                c.emit(&rule.common, Some(block.line), "TLMD_169", message);
                                check_plain(c, &block, rule, section);
                            });
                            count += 1;
                        }
                        break;
                    }
                    // Nobody declares it, and this position's own block comes
                    // later: report the block in the way, as one no position
                    // claims, and step over it.
                    // sokf-implements: TLMD_39
                    // sokf-implements: TLMD_68
                    if count == 0 && docs[di + 1..].iter().any(|b| c.matches_rule(b, rule)) {
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
                c.reserve = reserve_for(c, rules, ri, docs, di);
                // The block as it stands before the claim cuts a run, for an
                // earlier position that may own it out of place.
                let firm = firm(rule, c);
                let held = (firm && !skipped.is_empty()).then(|| docs[di].clone());
                let before = c.save();
                let around = Around {
                    later: &later_rules,
                    kinds: &later,
                    held: 0,
                    ordered: true,
                    dividing: &dividing,
                };
                let claimed = claim(c, docs, di, rule, section, firm, divides(rule.kind), around);
                c.reserve = 0;
                match claimed {
                    // A group whose `min` admits no repetition is satisfied by
                    // none, and takes no block. Counting it as one would eat
                    // the block it did not take.
                    Ok(Claimed { blocks: 0, .. }) => {
                        count += 1;
                        break;
                    }
                    // The block, or the start of the run, fails this position by
                    // its shape and has the shape of an optional position that
                    // took no block: it is that position's block out of place.
                    // It is reported once and checked there, and this position
                    // tries the next block, so the blocks after it reach their
                    // own positions. Only a block that already failed is tried,
                    // so a document that passed still passes and one that
                    // failed still fails.
                    // sokf-implements: TLMD_209
                    // sokf-implements: TLMD_210
                    // sokf-implements: TLMD_211
                    // sokf-implements: MD-4_AC-13
                    // sokf-implements: MD-11_AC-3
                    Ok(Claimed { blocks: n, .. }) => {
                        let found = held.filter(|_| c.failed(&before)).and_then(|held| {
                            owner(c, rules, &skipped, &held, rule, divides(rule.kind))
                                .map(|o| (held, o))
                        });
                        if let Some((held, (k, take))) = found {
                            c.rollback(&before);
                            // The claim may have cut the run where the position
                            // failed: the owner takes the run as it stood.
                            // sokf-implements: TLMD_226
                            if docs[di].units.len() < held.units.len() {
                                docs.remove(di + 1);
                            }
                            docs[di] = held;
                            divide::commit(docs, di, take);
                            // sokf-implements: TLMD_227
                            if !rules[k].repeat {
                                skipped.retain(|&s| s != k);
                            }
                            let owner = rules[k];
                            c.located(at_block(&c.at, owner), |c| {
                                let message = misplaced(c, &docs[di]);
                                c.error(Some(docs[di].line), "TLMD_169", message);
                                check_plain(c, &docs[di], owner, section);
                            });
                            di += 1;
                            continue;
                        }
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
                        // sokf-implements: TLMD_228
                        let blocks = dropped.blocks;
                        c.keep(dropped);
                        answered = true;
                        di += blocks;
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
            if count == 0 && c.block_optional(rule) && rule.kind != BlockKind::Group {
                skipped.push(ri);
            }
            if count == 0 && !c.block_optional(rule) && !answered {
                // The position refused a block of its own kind: why it
                // refused says more than the kind it declined. The block went
                // on to the positions after it, so the units the position
                // accepted are not its own. The units its findings report
                // still break its rules.
                // sokf-implements: TLMD_66
                if let Some(dropped) = refusal {
                    c.out.findings.extend(dropped.findings);
                    c.broken.extend(dropped.broken);
                } else {
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
                        c.broke_block(&docs[di]);
                        di += 1;
                    }
                }
            }
        }
        di - start
    }
}

/// The message of a block out of place.
// sokf-implements: TLMD_169
fn misplaced(c: &Checker<'_>, block: &Block) -> String {
    format!(
        "the {} block is out of place; the blocks are in the order the schema lists them",
        block.kind.name_in(&c.host.dialect())
    )
}

/// The first of the `skipped` positions whose block `block` is, out of
/// place, with the number of its units that the position takes: the
/// leading units of a run `divides` lets it divide that have the
/// position's shape, whatever its `max`, else the whole block where every
/// unit has it. The units it takes must fail `failed`, the position that
/// claimed the block, by shape. Shapes alone decide, so a block that
/// failed for another reason, such as a ban or a key, keeps its findings.
// sokf-implements: TLMD_209
// sokf-implements: TLMD_224
// sokf-implements: TLMD_225
// sokf-implements: TLMD_226
fn owner(
    c: &Checker<'_>,
    rules: &[&Rule],
    skipped: &[usize],
    block: &Block,
    failed: &Rule,
    divides: bool,
) -> Option<(usize, usize)> {
    skipped.iter().copied().find_map(|k| {
        let rule = rules[k];
        if !c.matches_rule(block, rule) {
            return None;
        }
        let take = if divides && is_run(rule.kind) {
            block
                .units
                .iter()
                .take_while(|u| shaped(c, std::slice::from_ref(u), rule) == Some(true))
                .count()
        } else if shaped(c, &block.units, rule) == Some(true) {
            block.units.len()
        } else {
            0
        };
        (take > 0 && shaped(c, &block.units[..take], failed) == Some(false)).then_some((k, take))
    })
}

/// Whether each of `units` has the shape of a unit rule of `rule`, as
/// `has_shape` reads it: `None` where no unit rule of `rule` declares a
/// template, a pattern, or a format, or where there are no units.
// sokf-implements: TLMD_224
// sokf-implements: TLMD_225
fn shaped(c: &Checker<'_>, units: &[Unit], rule: &Rule) -> Option<bool> {
    let shapes: Vec<&UnitRule> = rule
        .units()
        .filter(|u| u.shape.is_some() || u.format.is_some())
        .collect();
    if shapes.is_empty() || units.is_empty() {
        return None;
    }
    Some(units.iter().all(|u| {
        shapes
            .iter()
            .any(|r| has_shape(c, &c.read(u, rule, Some(r)).squashed.text, r))
    }))
}

/// The locator narrowed to one block rule: its `rule-id`, else where it
/// sits among the blocks declared beside it.
// sokf-implements: TL_73
pub(super) fn at_block(at: &At, rule: &Rule) -> At {
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

/// The locator narrowed to one unit rule or line rule: its `rule-id`,
/// where it declares one, and its line, with the place of its block, so a
/// rule without a `rule-id` names its block.
// sokf-implements: TL_73
pub(super) fn at_unit(at: &At, rule: &UnitRule) -> At {
    At {
        rule_id: rule.rule_id.clone().or_else(|| at.rule_id.clone()),
        line: Some(rule.common.line),
        ..at.clone()
    }
}

/// Claim `docs[di]` for `rule`: the blocks consumed, or the findings of
/// the refusal. A firm position claims whatever it reports.
// sokf-implements: TLMD_59
// sokf-implements: TLMD_60
#[expect(
    clippy::too_many_arguments,
    reason = "each is a fact of the walk that the claim reads"
)]
pub(super) fn claim(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    firm: bool,
    divides: bool,
    around: Around<'_>,
) -> Result<Claimed, Dropped> {
    c.located(at_block(&c.at, rule), |c| {
        claim_located(c, docs, di, rule, section, firm, divides, around)
    })
}

/// As `claim`, with the locator narrowed to `rule`.
#[expect(
    clippy::too_many_arguments,
    reason = "each is a fact of the walk that the claim reads"
)]
fn claim_located(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    firm: bool,
    divides: bool,
    around: Around<'_>,
) -> Result<Claimed, Dropped> {
    if let Some(bounds) = extents(&docs[di], rule, divides, c.reserve) {
        return extent(c, docs, di, rule, section, firm, bounds).map(|blocks| Claimed {
            blocks,
            repetitions: 0,
        });
    }
    let at = c.save();
    // A group's inner positions may divide a run, which is the one thing
    // a trial writes outside the checker's output.
    let blocks = (rule.kind == BlockKind::Group).then(|| docs.clone());
    let checked = group::check_block(c, docs, di, rule, section, around);
    let n = checked.taken;
    // A group that took blocks owns them and what it reports of them: the
    // repetitions it gave up were given back before it returned.
    // sokf-implements: TLMD_73
    let owns = rule.kind == BlockKind::Group && n > 0;
    if firm || owns || !c.failed_by_text(&at) {
        return Ok(Claimed {
            blocks: n,
            repetitions: checked.repetitions,
        });
    }
    let mut dropped = c.rollback(&at);
    dropped.blocks = checked.covers;
    if let Some(blocks) = blocks {
        *docs = blocks;
    }
    Err(dropped)
}
