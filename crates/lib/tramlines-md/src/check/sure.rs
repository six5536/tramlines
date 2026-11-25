//! Where the check is sure which position a unit belongs to, which a repair
//! of a fixed text needs: an error that decides a claim can move the blocks
//! after it to other positions, a unit that has the shape of another rule
//! of its walk can belong to that rule's position, and so can a unit that
//! an empty optional position after it would take.

use std::{borrow::Cow, ops::Range};

use super::{
    Checker,
    blocks::has_shape,
    claim::{Later, opening, shaped},
    trial::{Savepoint, is_text_error},
};
use crate::{
    Finding,
    grammar::{Block as Rule, States, Unit as UnitRule},
    markdown::{Block, BlockKind, Unit},
};

/// The block rules of the walk that holds `block`, among `rules` and the
/// blocks that their items and groups declare, each that the `possible`
/// states admit: the positions of one walk, with the inner positions of each
/// group among them, because a group's repetition is a sequence of
/// positions of the walk. `None` where no walk of `rules` holds `block`.
fn walk_of<'r>(rules: &'r [Rule], block: &Rule, possible: &States) -> Option<Vec<&'r Rule>> {
    fn flat<'r>(rules: &'r [Rule], possible: &States, out: &mut Vec<&'r Rule>) {
        for rule in rules.iter().filter(|r| r.common.only.admits(possible)) {
            out.push(rule);
            if let Some(inner) = &rule.blocks {
                flat(inner, possible, out);
            }
        }
    }
    let mut walk = Vec::new();
    flat(rules, possible, &mut walk);
    if walk.iter().any(|r| std::ptr::eq(*r, block)) {
        return Some(walk);
    }
    walk.iter()
        .flat_map(|r| r.units())
        .filter_map(|u| u.blocks.as_deref())
        .find_map(|nested| walk_of(nested, block, possible))
}

impl Checker<'_> {
    /// Whether `unit`, which `rule` of `block` checks, has the shape of
    /// another unit rule of a position of its kind in its walk, its own
    /// block's other rules among them: it reads as the text that the rule
    /// fixes, or it matches its template, its pattern, or a shape of its
    /// format. A rule that declares no shape takes any unit, so it says
    /// nothing of where the unit belongs.
    // sokf-implements: TLMD_280
    pub(super) fn has_other_shape(&self, unit: &Unit, rule: &UnitRule, block: &Rule) -> bool {
        let Some(walk) = walk_of(self.body, block, &self.possible) else {
            return false;
        };
        walk.into_iter()
            .filter(|b| b.kind == block.kind)
            .flat_map(|b| b.units().map(move |u| (b, u)))
            .any(|(b, other)| {
                !std::ptr::eq(other, rule)
                    && other.common.only.admits(&self.possible)
                    && other.has_shape()
                    && has_shape(self, &self.read(unit, b, Some(other)).squashed.text, other)
            })
    }

    /// Record that the repair about to be written settles `finding`, the
    /// finding of the fixed text that it rewrites.
    pub(super) fn settles(&mut self, finding: &Finding) {
        self.settled.push((finding.line, finding.message.clone()));
    }

    /// Drop each repair of a fixed text written since `at`, the start of the
    /// check of `blocks`, the blocks under one heading or of the preamble,
    /// where they hold an error that decides a claim, as `is_text_error`
    /// reads it, other than a finding that a repair settles, or where
    /// `doubt` says that the heading above them is not surely the one whose
    /// blocks they are (`TLMD_285`). The findings stay. A repair settles the
    /// finding of the text that it rewrites, which `settles` records, on the
    /// line where the repair starts, and a value that no unit of its kind
    /// holds (`TLMD_276`) or a key that the document doesn't set
    /// (`TLMD_168`) is a fault of the frontmatter, whose unit the repair
    /// leaves as it stands, as is what the unit's block refuses in a text
    /// that is the value's own, where the unit's position took its block
    /// (`TLMD_312`). Where the claims are in doubt, each unit of `blocks`
    /// keeps its anchors where they stand, as a unit that breaks its rule
    /// does.
    // sokf-implements: TLMD_279
    // sokf-implements: TLMD_109
    // sokf-implements: TLMD_139
    pub(super) fn keep_sure_repairs(&mut self, at: &Savepoint, blocks: &[&Block], doubt: bool) {
        let repairs = &self.out.repairs[at.repairs..];
        let values = self.claimed_tags_since(at);
        let settled = |f: &Finding| match f.criterion.as_str() {
            "TL_74" | "TLMD_165" => {
                repairs.iter().any(|r| Some(r.line) == f.line)
                    && self
                        .settled
                        .iter()
                        .any(|(line, message)| *line == f.line && *message == f.message)
            }
            "TLMD_168" | "TLMD_276" => true,
            // The inline HTML or the ban of a unit that reads as its bound
            // value is the value's fault, which `bound::report_values`
            // reports on the frontmatter (`TLMD_312`).
            _ => values.contains(&(f.criterion.as_str(), f.line, f.message.as_str())),
        };
        let doubt = doubt
            || self.out.findings[at.findings..]
                .iter()
                .any(|f| is_text_error(f) && !settled(f));
        if doubt {
            self.out.repairs.truncate(at.repairs);
            for block in blocks {
                self.broke_block(block);
            }
        }
    }

    /// Drop each repair written since `at` on or after `line`, the first
    /// line of the second unit of a run that a position takes whole because
    /// no extent of it passes (`TLMD_75`): the position keeps the first unit,
    /// with what it holds, and the claim of each later unit rests on that
    /// failure.
    // sokf-implements: TLMD_281
    pub(super) fn drop_repairs_from(&mut self, at: &Savepoint, line: usize) {
        let written = self.out.repairs.split_off(at.repairs);
        self.out
            .repairs
            .extend(written.into_iter().filter(|r| r.line < line));
    }

    /// Drop each repair of a fixed text that a position of `rules` wrote over
    /// a unit that a later optional position of the unit's kind, after
    /// positions that took no block alone, would take: that position took no
    /// block, and declares no shape or a shape that the unit has, so the unit
    /// can be its own, with the fixed text before it missing. `took` and
    /// `wrote` say whether each of `rules` took a block and which repairs it
    /// wrote, and `docs` holds the blocks of the walk. Where the positions
    /// after the last that took a block took none, `later`, the positions
    /// after the group whose repetition the walk is, are read too, up to the
    /// first that the schema requires, and what they hold back waits in
    /// `Checker::deferred` for the group to keep the repetition as its last.
    // sokf-implements: TLMD_286
    pub(super) fn spare_for_empty(
        &mut self,
        rules: &[&Rule],
        took: &[bool],
        wrote: &[Range<usize>],
        docs: &[Cow<'_, Block>],
        later: &[Later<'_>],
    ) {
        // A group that admits no repetition can be absent as an optional
        // position can.
        let optional = |c: &Self, rule: &Rule| {
            c.block_optional(rule) || (rule.kind == BlockKind::Group && rule.min == 0)
        };
        // The repairs of position `i` over units that `rule`, or the position
        // that it opens with where it is a group, would take: the last unit
        // of a block, because a later position takes the end of a run.
        let spare = |c: &Self, i: usize, rule: &Rule| -> Vec<usize> {
            let kind = rules[i].kind;
            let Some(open) = opening(rule, kind, c) else {
                return Vec::new();
            };
            wrote[i]
                .clone()
                .filter(|&r| {
                    let line = c.out.repairs[r].line;
                    docs.iter()
                        .filter(|b| b.kind == kind)
                        .find_map(|b| b.units.last().filter(|u| u.line == line))
                        .is_some_and(|u| shaped(c, std::slice::from_ref(u), open) != Some(false))
                })
                .collect()
        };
        let mut spared: Vec<usize> = Vec::new();
        for (k, rule) in rules.iter().enumerate() {
            if took[k] || !optional(self, rule) {
                continue;
            }
            if let Some(i) = (0..k).rev().find(|&i| took[i]) {
                spared.extend(spare(self, i, rule));
            }
        }
        if let Some(last) = took.iter().rposition(|&t| t) {
            for after in later {
                if !optional(self, after.rule) {
                    break;
                }
                let lines: Vec<usize> = spare(self, last, after.rule)
                    .into_iter()
                    .map(|r| self.out.repairs[r].line)
                    .collect();
                self.deferred.extend(lines);
            }
        }
        if !spared.is_empty() {
            let mut r = 0;
            self.out.repairs.retain(|_| {
                r += 1;
                !spared.contains(&(r - 1))
            });
        }
    }

    /// Drop each repair written since `repairs` whose line the walks of a
    /// group's repetitions held back since `deferred`, which only those of
    /// the last repetition that the group keeps still hold, and end the
    /// wait (`TLMD_286`).
    pub(super) fn spare_deferred(&mut self, repairs: usize, deferred: usize) {
        let lines = self.deferred.split_off(deferred);
        if lines.is_empty() {
            return;
        }
        let written = self.out.repairs.split_off(repairs);
        self.out
            .repairs
            .extend(written.into_iter().filter(|r| !lines.contains(&r.line)));
    }
}
