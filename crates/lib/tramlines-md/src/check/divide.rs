//! How a position divides a run: the extents it may take of the run, the
//! longest that passes by the text of its units, and the cut that gives
//! the rest to the position after it. No count divides a run: neither a
//! `min` nor a `max` decides where a position ends.

use std::borrow::Cow;

use super::{
    Checker,
    blocks::check_plain,
    claim::{Docs, Dropped, is_run, is_text_error},
};
use crate::{Finding, grammar::Block as Rule, markdown::Block};

/// The extents a position may take of a run: the fewest and the most
/// units, or `None` where the position takes the block whole. The most is
/// the whole run, less the `reserve` units at its end that a later group's
/// opening takes, but never the first unit of the run.
// sokf-implements: TLMD_77
// sokf-implements: TLMD_78
// sokf-implements: TLMD_193
// sokf-implements: TLMD_194
pub(super) fn extents(
    block: &Block,
    rule: &Rule,
    divides: bool,
    reserve: usize,
) -> Option<(usize, usize)> {
    // A hard `max` of zero admits no units at all, so there is nothing to
    // divide: the block is taken whole and reported as too many.
    if !divides || !is_run(rule.kind) || block.units.len() < 2 || rule.hard_max() == Some(0) {
        return None;
    }
    let len = block.units.len();
    // The position keeps the first unit of its run, whatever the group
    // after it could also take.
    Some((1, len - reserve.min(len - 1)))
}

/// The largest extent worth trying once `n` failed: the units before the
/// first line a finding named, and never more than one unit fewer. A rule
/// reads a unit the same way whatever follows it, so no extent reaching
/// that line can pass either. Where the finding names the block rather
/// than a unit, one unit is given up and the search goes on.
fn narrow(block: &Block, n: usize, findings: &[Finding]) -> usize {
    findings
        .iter()
        .filter(|f| is_text_error(f))
        .filter_map(|f| f.line)
        .min()
        .map_or(n, |l| {
            block.units.iter().take(n).filter(|u| u.line < l).count()
        })
        .min(n - 1)
}

/// The longest extent of a run the position writes no error against, other
/// than an error on a count, cut from the block and committed; the rest
/// falls to the position after it. Only the text of the units decides the
/// extent, and its counts are reported against it.
// sokf-implements: TLMD_48
// sokf-implements: TLMD_49
// sokf-implements: TLMD_74
pub(super) fn extent(
    c: &mut Checker<'_>,
    docs: &mut Docs<'_>,
    di: usize,
    rule: &Rule,
    section: Option<usize>,
    firm: bool,
    (lower, upper): (usize, usize),
) -> Result<usize, Dropped> {
    let mut first: Option<Dropped> = None;
    let mut n = upper;
    loop {
        let at = c.save();
        check_plain(c, &prefix(&docs[di], n), rule, section);
        // The extent that the text of its units chose is the position's,
        // and its counts are reported against it: a count neither cuts the
        // extent shorter nor refuses it.
        if !c.failed_by_text(&at) {
            commit(docs, di, n);
            return Ok(1);
        }
        let dropped = c.rollback(&at);
        let next = narrow(&docs[di], n, &dropped.findings);
        if first.is_none() {
            first = Some(dropped);
        }
        if next < lower {
            break;
        }
        n = next;
    }
    // Nothing satisfies the position. One that claims whatever it reports
    // takes the whole run, less what a later group's opening takes, which
    // is what it took before a run was searched at all.
    // sokf-implements: TLMD_75
    if firm {
        check_plain(c, &prefix(&docs[di], upper), rule, section);
        commit(docs, di, upper);
        return Ok(1);
    }
    Err(first.unwrap_or_default())
}

/// Give `docs[di]` its first `n` units and insert the rest after it.
pub(super) fn commit(docs: &mut Docs<'_>, di: usize, n: usize) {
    if n >= docs[di].units.len() {
        return;
    }
    let tail = cut(docs[di].to_mut(), n);
    docs.insert(di + 1, Cow::Owned(tail));
}

/// Take the first `n` units of `block`, returning the rest as a block of
/// its own. Each piece carries the lines of the units it keeps.
pub(super) fn cut(block: &mut Block, n: usize) -> Block {
    let units = block.units.split_off(n);
    let line = units.first().map_or(block.end_line, |u| u.line);
    let end_line = units.last().map_or(block.end_line, |u| u.end_line);
    let html_lines = block
        .html_lines
        .iter()
        .copied()
        .filter(|l| *l >= line)
        .collect();
    block.html_lines.retain(|l| *l < line);
    block.end_line = block.units.last().map_or(block.line, |u| u.end_line);
    Block {
        html_lines,
        ..Block::new(block.kind, line, end_line, units)
    }
}

/// A copy of `block` holding its first `n` units, with the lines of those
/// units alone.
fn prefix(block: &Block, n: usize) -> Block {
    let mut head = block.clone();
    let _ = cut(&mut head, n);
    head
}
