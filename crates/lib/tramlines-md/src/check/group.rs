//! A block checked against its rule, and the repetitions of a group: a
//! group walks its inner rules over the blocks once for each repetition.

use super::{
    Checker, blocks,
    claim::{self, Around, Docs, Later, Outside, is_text_error},
};
use crate::{grammar::Block as Rule, markdown::BlockKind};

/// What the check of one block did.
pub(super) struct Checked {
    /// The blocks that the check consumed.
    pub(super) taken: usize,
    /// The blocks that a refusal of the check covers: those of the
    /// repetition that an absent group gave back, else one, because a
    /// group that took blocks owns them and is never refused.
    pub(super) covers: usize,
    /// The repetitions that a group took, past those it held before.
    pub(super) repetitions: usize,
}

/// Check one block against its rule. A group walks its inner rules over
/// the blocks from `di`, past the repetitions that it holds already.
pub(super) fn check_block(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    around: Around<'_>,
) -> Checked {
    if rule.kind != BlockKind::Group {
        blocks::check_plain(c, &docs[di], rule, section);
        return Checked {
            taken: 1,
            covers: 1,
            repetitions: 0,
        };
    }
    let line = docs[di].line;
    let inner: Vec<&Rule> = c.admitted(rule).collect();
    // The group's `rule-id` names what its inner blocks capture.
    let outer = c.group_rule.take();
    c.group_rule = rule.rule_id.clone().or(outer.clone());
    let optional = c.block_optional(rule);
    // The next repetition divides the run whatever the group's `max`,
    // because no count divides a run, and so does a position after the
    // group where the walk around it fixes the order (TLMD_78). A walk in
    // any order gives the inner walk none, because no order says that a
    // block of another position stands after the group.
    let later: &[Later<'_>] = if around.ordered { around.later } else { &[] };
    let outside = Outside {
        wraps: (inner.first().map(|r| r.kind) == inner.last().map(|r| r.kind))
            .then(|| inner.first().map(|r| r.kind))
            .flatten(),
        later,
        after: claim::opened(later.iter().map(|l| l.rule), c),
    };
    let mut reps = around.held;
    let mut absent = None;
    let mut pos = di;
    // The repairs that the last repetition kept holds back (TLMD_286).
    let repairs_from = c.out.repairs.len();
    let deferred_from = c.deferred.len();
    c.group_depth += 1;
    while pos < docs.len() {
        let opening = c.matches_rule(&docs[pos], rule);
        let claimed = around.kinds.contains(&docs[pos].kind);
        if !opening && (claimed || !c.joins(&docs[pos], rule)) {
            break;
        }
        let at = c.save();
        // Each repetition is a part that the group's `only` gates, as the
        // scaffold writes a marker before each, and the savepoint drops the
        // gate of a repetition that the group gives up.
        // sokf-implements: TLMD_272
        // sokf-implements: MD-4_AC-20
        c.gate(&rule.common, docs[pos].line);
        let blocks = docs.clone();
        let start = docs[pos].line;
        let before = c.deferred.len();
        let n = c.match_sequence(&inner, docs, pos, section, start, &outside);
        if n > 0 && !c.failed_by_text(&at) {
            pos += n;
            reps += 1;
            c.deferred.drain(deferred_from..before);
            continue;
        }
        // A repetition whose blocks no position after it takes, one whose
        // opening units the group accepted, and one the group needs to
        // reach its `min` where no position after it takes its opening
        // block cleanly, are kept with the reason they failed: the opening
        // says whose they are. The opening units are those that the
        // opening position took, the start of a run that it divided. An
        // optional group can be absent, so its `min` keeps no first
        // repetition.
        // sokf-implements: TLMD_71
        let first = docs[pos].line..=docs[pos].end_line;
        let owned = opening
            && !c.out.findings[at.findings..]
                .iter()
                .any(|f| is_text_error(f) && f.line.is_some_and(|l| first.contains(&l)));
        let needed = reps < rule.min && (reps > 0 || !optional);
        if n > 0
            && (!claimed
                || owned
                || (needed && (c.trying || !later_takes(c, &blocks, pos, around, section))))
        {
            pos += n;
            reps += 1;
            c.deferred.drain(deferred_from..before);
            continue;
        }
        // A repetition that cannot be completed is not a repetition, and
        // a run one of its positions divided is put back together: the
        // repetition after it would otherwise read the pieces.
        // sokf-implements: TLMD_61
        let end = (n > 0).then(|| docs[pos + n - 1].end_line);
        *docs = blocks;
        // An optional group that needs a repetition and gives back its first
        // is absent, so it refuses the block with the reason that the
        // repetition failed, not with a count of none: it keeps the
        // findings of the repetition and drops the rest of what it wrote.
        // The refusal covers the blocks that the repetition took whole, as
        // they stand put back together, its opening at least: a run that it
        // took only the start of stays for the rules after it.
        // sokf-implements: TLMD_61
        if reps == 0 && optional && rule.min > 0 && c.failed_by_text(&at) {
            c.discard(&at);
            absent = Some(end.map_or(1, |end| {
                docs[pos..]
                    .iter()
                    .take_while(|b| b.end_line <= end)
                    .count()
                    .max(1)
            }));
        } else {
            c.rollback(&at);
        }
        break;
    }
    c.group_rule = outer;
    // Only after the last repetition can an empty optional position after
    // the group take its unit, and only after the last of every group that
    // holds this one, because those positions stand after them too.
    c.group_depth -= 1;
    if c.group_depth == 0 {
        c.spare_deferred(repairs_from, deferred_from);
    }
    // A group whose `min` is 0 takes no block.
    let taken = pos - di;
    // A walk in any order counts a group's repetitions once, over all its
    // claims.
    if absent.is_none() && around.ordered {
        blocks::check_count(c, rule, reps, line);
    }
    Checked {
        taken,
        covers: absent.unwrap_or(1),
        repetitions: reps - around.held,
    }
}

/// Whether the position after the group that the walk would give
/// `docs[pos]`, the opening block of a failed repetition, takes it with no
/// error. The later positions that open with its kind are tried in the
/// walk's order, each in a trial that is rolled back, claiming for the
/// position as the walk that holds it would, which for a position after an
/// outer group is the walk around that group: dividing a run as that walk
/// would, and leaving to a later group of that walk the units at the end of
/// a run that the walk leaves to it (`TLMD_193`), after each required
/// position before it that the opening block doesn't fit has taken its own
/// block back from later, out of place (`TLMD_169`). The first that claims the
/// block whatever it reports, a required position or a group that takes
/// it, decides, and one that refuses it passes it on. A group inside the
/// trial tries nothing itself, so the trials stay as many as the positions.
// sokf-implements: TLMD_71
fn later_takes(
    c: &mut Checker<'_>,
    docs: &Docs<'_>,
    pos: usize,
    around: Around<'_>,
    section: Option<usize>,
) -> bool {
    // The blocks as the walk holds them when it reaches each position: a
    // required position that the opening block doesn't fit takes its own
    // block back from later, out of place, where no position after it
    // wants that block (TLMD_169), so a later position reads the block after
    // the opening block as the walk leaves it.
    let mut held = docs.clone();
    for (k, later) in around.later.iter().enumerate() {
        let rule = later.rule;
        if !c.matches_rule(&held[pos], rule) {
            if around.ordered {
                let kinds = claim::opened(around.later[k + 1..].iter().map(|l| l.rule), c);
                if kinds.contains(&held[pos].kind)
                    && let Some(j) = claim::out_of_place(c, &held, pos, rule, &kinds)
                {
                    held.remove(j);
                }
            }
            continue;
        }
        // The positions after this one, as the walk passes them: those after
        // it in order, or every other free rule in any order.
        let rest: Vec<Later<'_>> = if around.ordered {
            around.later[k + 1..].to_vec()
        } else {
            (0..around.later.len())
                .filter(|&j| j != k)
                .map(|j| around.later[j])
                .collect()
        };
        let kinds = claim::opened(rest.iter().map(|l| l.rule), c);
        let after = Around {
            later: &rest,
            kinds: &kinds,
            held: 0,
            ordered: around.ordered,
        };
        let divides = around.ordered && later.divides;
        // The units that the walk that holds the position leaves to a later
        // group of its own, read from its own positions after this one, as
        // the walk sets them before it claims (TLMD_193). A walk in any
        // order divides no run, so it reserves nothing.
        // sokf-implements: TLMD_193
        let reserve = if around.ordered {
            let own: Vec<&Rule> = std::iter::once(rule)
                .chain(rest.iter().take(later.own).map(|l| l.rule))
                .collect();
            claim::reserve_for(c, &own, 0, &held, pos)
        } else {
            0
        };
        // A walk in any order claims nothing whatever it reports.
        let firm = around.ordered && claim::firm(rule, c);
        let mut probe = held.clone();
        let at = c.save();
        let trying = std::mem::replace(&mut c.trying, true);
        let reserved = std::mem::replace(&mut c.reserve, reserve);
        let taken = claim::claim(c, &mut probe, pos, rule, section, firm, divides, after);
        c.reserve = reserved;
        c.trying = trying;
        let clean = !c.failed_by_text(&at);
        c.rollback(&at);
        match taken {
            Ok(t) if t.blocks > 0 => return clean,
            _ => {}
        }
    }
    false
}
