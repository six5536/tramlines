//! How a position divides a run: the extents it may take of the run, the
//! longest that passes, and the cut that gives the rest to the position
//! after it.

use std::borrow::Cow;

use super::{
    Checker,
    blocks::check_plain,
    claim::{Docs, is_run},
};
use crate::{Finding, grammar::Block as Rule, markdown::Block};

/// The extents a position may take of a run: the fewest and the most
/// units, or `None` where the position takes the block whole. The most
/// leaves `reserve` units at the end of the run, where the bounds still
/// admit an extent, for a later group's opening.
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
    // A `max` of zero admits no units at all, so there is nothing to
    // divide: the block is taken whole and reported as too many.
    if !divides || !is_run(rule.kind) || block.units.len() < 2 || rule.max == Some(0) {
        return None;
    }
    let len = block.units.len();
    let mut upper = rule.max.map_or(len, |m| m.min(len));
    let lower = rule.min.max(1);
    if reserve > 0 && len >= lower + reserve {
        upper = upper.min(len - reserve);
    }
    Some((lower.min(upper), upper))
}

/// The largest extent worth trying once `n` failed: the units before the
/// first line a finding named, and never more than one unit fewer. A rule
/// reads a unit the same way whatever follows it, so no extent reaching
/// that line can pass either. Where the finding names the block rather
/// than a unit, one unit is given up and the search goes on.
fn narrow(block: &Block, n: usize, findings: &[Finding]) -> usize {
    findings
        .iter()
        .filter_map(|f| f.line)
        .min()
        .map_or(n, |l| {
            block.units.iter().take(n).filter(|u| u.line < l).count()
        })
        .min(n - 1)
}

/// The longest extent of a run the position writes no error against, cut
/// from the block and committed; the rest falls to the position after it.
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
) -> Result<usize, Vec<Finding>> {
    let mut first: Option<Vec<Finding>> = None;
    let mut n = upper;
    loop {
        let at = c.save();
        check_plain(c, &prefix(&docs[di], n), rule, section);
        if !c.failed(&at) {
            commit(docs, di, n);
            return Ok(1);
        }
        let dropped = c.rollback(&at);
        let next = narrow(&docs[di], n, &dropped);
        if first.is_none() {
            first = Some(dropped);
        }
        if next < lower {
            break;
        }
        n = next;
    }
    // Nothing satisfies the position. One that claims whatever it reports
    // takes the most its bounds admit, which is what it took before a run
    // was searched at all.
    // sokf-implements: TLMD_75
    if firm {
        check_plain(c, &prefix(&docs[di], upper), rule, section);
        commit(docs, di, upper);
        return Ok(1);
    }
    Err(first.unwrap_or_default())
}

/// Give `docs[di]` its first `n` units and insert the rest after it.
fn commit(docs: &mut Docs<'_>, di: usize, n: usize) {
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
    let line = units.first().map(|u| u.line).unwrap_or(block.end_line);
    let end_line = units.last().map(|u| u.end_line).unwrap_or(block.end_line);
    let html_lines = block
        .html_lines
        .iter()
        .copied()
        .filter(|l| *l >= line)
        .collect();
    block.html_lines.retain(|l| *l < line);
    block.end_line = block.units.last().map(|u| u.end_line).unwrap_or(block.line);
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
