//! A block checked against its rule, and the repetitions of a group: a
//! group walks its inner rules over the blocks once for each repetition.

use super::{
    Checker,
    blocks::{check_count, check_plain},
    claim::{Docs, Dropped, admitted, is_text_error, joins, match_sequence, matches_rule},
};
use crate::{grammar::Block as Rule, markdown::BlockKind};

/// What the check of one block did.
pub(super) struct Checked {
    /// The blocks that the check consumed.
    pub(super) taken: usize,
    /// The blocks that a refusal of the check covers: those consumed, else
    /// those of the repetition that an absent group gave back, and at least
    /// one.
    pub(super) covers: usize,
}

/// Check one block against its rule. A group walks its inner rules over
/// the blocks from `di`.
pub(super) fn check_block(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
) -> Checked {
    if rule.kind != BlockKind::Group {
        check_plain(c, &docs[di], rule, section);
        return Checked {
            taken: 1,
            covers: 1,
        };
    }
    let line = docs[di].line;
    c.gate(&rule.common, line);
    let inner: Vec<&Rule> = admitted(rule, c).collect();
    // The group's `rule-id` names what its inner blocks capture.
    let outer = c.group_rule.take();
    c.group_rule = rule.rule_id.clone().or(outer.clone());
    // What the positions after the group take, read before the inner walk
    // sets its own.
    let later = std::mem::take(&mut c.later);
    let optional = c.block_optional(rule);
    let mut reps = 0;
    let mut released = None;
    let mut span = 1;
    let mut pos = di;
    while pos < docs.len() {
        let opening = matches_rule(&docs[pos], rule, c);
        let claimed = later.contains(&docs[pos].kind);
        if !opening && (claimed || !joins(&docs[pos], rule, c)) {
            break;
        }
        let at = c.save();
        let blocks = docs.clone();
        let start = docs[pos].line;
        let first = start..=docs[pos].end_line;
        // The next repetition divides the run whatever the group's `max`,
        // because no count divides a run.
        let wraps = (inner.first().map(|r| r.kind) == inner.last().map(|r| r.kind))
            .then(|| inner.first().map(|r| r.kind))
            .flatten();
        let n = match_sequence(c, &inner, docs, pos, section, start, wraps);
        if n > 0 && !c.failed_by_text(&at) {
            pos += n;
            reps += 1;
            continue;
        }
        // A repetition the group needs to reach its `min`, one whose blocks
        // no position after it takes, and one whose opening block the group
        // accepted, are kept with the reason they failed: the opening says
        // whose they are. An optional group can be absent, so its `min`
        // keeps no first repetition.
        // sokf-implements: TLMD_71
        let owned = opening
            && !c.out.findings[at.findings..]
                .iter()
                .any(|f| is_text_error(f) && f.line.is_some_and(|l| first.contains(&l)));
        let needed = reps < rule.min && (reps > 0 || !optional);
        if n > 0 && (needed || !claimed || owned) {
            pos += n;
            reps += 1;
            continue;
        }
        // A repetition that cannot be completed is not a repetition, and
        // a run one of its positions divided is put back together: the
        // repetition after it would otherwise read the pieces.
        // sokf-implements: TLMD_61
        let end = (n > 0).then(|| docs[pos + n - 1].end_line);
        released = Some(c.rollback(&at));
        *docs = blocks;
        // The blocks that the repetition took whole, as they stand put back
        // together, its opening at least: a run that it took only the start
        // of stays for the rules after it.
        span = end.map_or(1, |end| {
            docs[pos..]
                .iter()
                .take_while(|b| b.end_line <= end)
                .count()
                .max(1)
        });
        break;
    }
    c.later = later;
    c.group_rule = outer;
    // An optional group that needs a repetition and released its first is
    // absent, so it refuses the block with the reason that repetition
    // failed, not with a count of none, and the refusal covers the blocks
    // of the repetition. A group whose `min` is 0 takes no block.
    // sokf-implements: TLMD_61
    let absent =
        |d: &Dropped| reps == 0 && optional && rule.min > 0 && d.findings.iter().any(is_text_error);
    let taken = pos - di;
    match released.filter(absent) {
        Some(dropped) => {
            c.keep(dropped);
            Checked {
                taken,
                covers: span,
            }
        }
        None => {
            check_count(c, rule, reps, line);
            Checked {
                taken,
                covers: taken.max(1),
            }
        }
    }
}
