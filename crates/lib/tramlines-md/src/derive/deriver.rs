//! The state of one derivation, and the trial that lets it try an
//! optional part and put back what the part wrote.

use std::collections::BTreeMap;

use super::Refusal;
use crate::{
    check::Host,
    grammar::{RuleCommon, Schema, States},
    markdown::{BlockKind, Dialect},
};

/// What one derivation carries: the schema, the states it stands for,
/// the keys its rules have captured so far, and what stopped it.
pub(crate) struct Deriver<'a> {
    pub(super) schema: &'a Schema,
    /// The host, which seeds and ends the frontmatter.
    pub(super) host: &'a dyn Host,
    /// How the scaffold spells its markers.
    pub(super) dialect: Dialect,
    pub(super) states: &'a States,
    /// The keys captured so far, by the `rule-id` that captured them.
    pub(super) keys: BTreeMap<String, Vec<String>>,
    /// Each block written so far, in order: its kind, whether its rule
    /// takes every unit of a run beyond those the scaffold wrote, the line
    /// of its rule, and the output line after it. Where two blocks of one
    /// run meet, the first may take the second.
    pub(super) kinds: Vec<(BlockKind, bool, usize, usize)>,
    /// The frontmatter as it was written, for a `text: { from: <key> }`.
    pub(super) values: BTreeMap<String, String>,
    /// The `rule-id` of the section being written, which names the keys
    /// captured inside it where the block declares none of its own.
    pub(super) section_rule: Option<String>,
    pub(super) refusals: Vec<Refusal>,
    /// The optional parts the derivation could not write, which it left
    /// out rather than refuse over.
    pub(super) omitted: Vec<Refusal>,
    /// Whether an optional region is open, so a part inside one is not
    /// marked again: the region already says it.
    pub(super) in_optional: bool,
    /// While a section with a `parent` is written, how many keys each rule
    /// had captured when that occurrence of the parent began: a pairing
    /// there reads the keys captured since.
    pub(super) scope: Option<BTreeMap<String, usize>>,
}

/// What a trial of an optional part writes, so it can be put back.
struct Trial {
    pub(super) lines: usize,
    pub(super) refusals: usize,
    pub(super) keys: BTreeMap<String, Vec<String>>,
    pub(super) kinds: usize,
}

impl Deriver<'_> {
    /// Write a part with `write`. An optional part is a trial: where it
    /// refuses, everything it wrote is put back.
    pub(super) fn attempt(
        &mut self,
        optional: bool,
        out: &mut Vec<String>,
        write: impl FnOnce(&mut Self, &mut Vec<String>),
    ) {
        if !optional {
            return write(self, out);
        }
        let at = self.save(out);
        write(self, out);
        if self.refusals.len() > at.refusals {
            self.undo(at, out);
        }
    }

    /// Where the derivation stands, before an optional part is tried.
    fn save(&self, out: &[String]) -> Trial {
        Trial {
            lines: out.len(),
            refusals: self.refusals.len(),
            keys: self.keys.clone(),
            kinds: self.kinds.len(),
        }
    }

    /// Put back everything the trial wrote, and keep what stopped it as an
    /// omission: an optional part the derivation cannot write is left out,
    /// not refused over, and the author is told which rule kept it out.
    // sokf-implements: MD-7_AC-7
    fn undo(&mut self, at: Trial, out: &mut Vec<String>) {
        out.truncate(at.lines);
        let stopped: Vec<Refusal> = self.refusals.split_off(at.refusals);
        self.keys = at.keys;
        self.kinds.truncate(at.kinds);
        push_unique(&mut self.omitted, stopped);
    }

    /// Write with `write` inside an optional region of its own, so that a
    /// part inside it is not marked again.
    pub(super) fn optional_region(
        &mut self,
        out: &mut Vec<String>,
        write: impl FnOnce(&mut Self, &mut Vec<String>),
    ) {
        let start = out.len();
        let held = std::mem::replace(&mut self.in_optional, true);
        write(self, out);
        self.in_optional = held;
        wrap(&self.dialect, start, out);
    }

    /// The text of the `only` marker a gated rule's part carries.
    pub(super) fn gate_text(&self, common: &RuleCommon) -> Option<String> {
        common
            .only
            .is_gated()
            .then(|| format!("{} {}", self.dialect.spell("only"), common.only.render()))
    }

    pub(super) fn refuse(&mut self, line: usize, what: impl Into<String>) {
        self.refuse_on(line, "TLMD_116", what);
    }

    /// Refuse the derivation under `criterion`, once for each rule and reason.
    // sokf-implements: MD-7_AC-6
    pub(super) fn refuse_on(
        &mut self,
        line: usize,
        criterion: &'static str,
        what: impl Into<String>,
    ) {
        let what = what.into();
        if !self
            .refusals
            .iter()
            .any(|r| r.line == line && r.what == what)
        {
            self.refusals.push(Refusal {
                line,
                criterion,
                what,
            });
        }
    }
}

/// Push each item of `from` that `into` does not hold yet.
pub(super) fn push_unique<T: PartialEq>(into: &mut Vec<T>, from: impl IntoIterator<Item = T>) {
    for one in from {
        if !into.contains(&one) {
            into.push(one);
        }
    }
}

/// Wrap everything written since `start` as one optional region, opened
/// and closed so a reader sees where the part begins and ends. Where the
/// run wrote nothing, the markers go too.
// sokf-implements: TLMD_114
fn wrap(dialect: &Dialect, start: usize, out: &mut Vec<String>) {
    while out.len() > start && out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    if out.len() == start {
        return;
    }
    out.insert(start, String::new());
    out.insert(start, format!("<!-- {} -->", dialect.spell("optional")));
    out.push(String::new());
    out.push(format!("<!-- /{} -->", dialect.spell("optional")));
    out.push(String::new());
}
