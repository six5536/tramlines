//! The state of one derivation, and the trial that lets it try an
//! optional part and put back what the part wrote.

use std::collections::{BTreeMap, HashMap};

use super::Refusal;
use crate::{
    check::Host,
    grammar::{Block, KeyText, RuleCommon, Schema, Section, Shape, States, Text, Unit, Words},
    markdown::{BlockKind, Dialect},
};

/// What one derivation writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    /// The scaffold, with its guidance lines, its markers and its optional
    /// regions.
    Scaffold,
    /// The text of the scaffold alone: no guidance line, no marker and no
    /// optional region, from which the guide takes each part's example.
    Plain,
    /// The skeleton: only what the schema prescribes and the document's
    /// values, and no example.
    Skeleton,
}

/// The text that a derivation in `Mode::Plain` wrote for each part, the
/// first time that it wrote the part, by the address of the part's rule in
/// the schema.
#[derive(Debug, Clone, Default)]
pub(super) struct Examples {
    /// The line of each frontmatter key, by key.
    pub(super) keys: BTreeMap<String, String>,
    /// The text of the preamble.
    pub(super) preamble: Option<String>,
    /// The text of each section with its subsections and its `examples`.
    pub(super) sections: HashMap<usize, String>,
    /// The text of each block.
    pub(super) blocks: HashMap<usize, String>,
}

/// The address of a rule of the schema, which names it in `Examples`.
pub(super) fn address<T>(rule: &T) -> usize {
    std::ptr::from_ref(rule) as usize
}

/// The lines `lines` as one text, without the blank lines at its ends.
pub(super) fn text_of(lines: &[String]) -> Option<String> {
    let start = lines.iter().position(|l| !l.trim().is_empty())?;
    let end = lines.iter().rposition(|l| !l.trim().is_empty())?;
    Some(lines[start..=end].join("\n"))
}

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
    pub(super) keys: BTreeMap<String, Vec<KeyText>>,
    /// The kind of each block written so far, in order.
    pub(super) kinds: Vec<BlockKind>,
    /// The frontmatter as it was written, for a `text: { from: <key> }`.
    pub(super) values: BTreeMap<String, String>,
    /// The `rule-id` of the section being written, which names the keys
    /// captured inside it where the block declares none of its own.
    pub(super) section_rule: Option<String>,
    /// The `rule-id` of the group being written, nearest first, which
    /// names the keys captured inside it where the block declares none.
    pub(super) group_rule: Option<String>,
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
    /// The labels of the schema, in the words that the host gives.
    pub(super) labels: Words,
    /// The key of the label that the host names for the units of a
    /// scaffold, where it names one that the schema declares.
    pub(super) named_label: Option<String>,
    /// What the derivation writes.
    pub(super) mode: Mode,
    /// The values of the document's frontmatter that the host gives, which
    /// the derivation writes in place of the schema's.
    pub(super) given: &'a BTreeMap<String, String>,
    /// The text of each part, where the derivation records it.
    pub(super) examples: Option<Examples>,
}

/// What a trial of an optional part writes, so it can be put back.
struct Trial {
    pub(super) lines: usize,
    pub(super) refusals: usize,
    pub(super) keys: BTreeMap<String, Vec<KeyText>>,
    pub(super) kinds: usize,
    /// The examples recorded before the trial, which a part left out takes
    /// back, so that the guide gives no example of a part that the
    /// scaffold doesn't write.
    pub(super) examples: Option<Examples>,
}

impl Deriver<'_> {
    /// Whether the derivation writes no guidance line, no marker and no
    /// optional region.
    // sokf-implements: TLMD_303
    pub(super) fn plain(&self) -> bool {
        self.mode != Mode::Scaffold
    }

    /// Whether the derivation writes the skeleton.
    pub(super) fn skeleton(&self) -> bool {
        self.mode == Mode::Skeleton
    }

    /// Whether the derivation writes the units of `rule`: every unit in a
    /// scaffold, and in a skeleton only a unit whose text the schema fixes
    /// with `text`, or binds to a frontmatter value that the skeleton
    /// writes.
    // sokf-implements: TLMD_303
    // sokf-implements: TLMD_307
    pub(super) fn keeps(&self, rule: Option<&Unit>) -> bool {
        if !self.skeleton() {
            return true;
        }
        match rule.and_then(|r| r.shape.as_ref()) {
            Some(Shape::Text(Text::Literal(_))) => true,
            Some(Shape::Text(Text::From(key))) => self.values.contains_key(key),
            _ => false,
        }
    }

    /// Record `lines`, the text written for the section `rule`, where the
    /// derivation records examples and has not recorded the section yet.
    pub(super) fn record_section(&mut self, rule: &Section, lines: &[String]) {
        if let Some(ex) = &mut self.examples
            && let Some(text) = text_of(lines)
        {
            ex.sections.entry(address(rule)).or_insert(text);
        }
    }

    /// Record `lines`, the text written for the block `rule`, where the
    /// derivation records examples and has not recorded the block yet.
    pub(super) fn record_block(&mut self, rule: &Block, lines: &[String]) {
        if let Some(ex) = &mut self.examples
            && let Some(text) = text_of(lines)
        {
            ex.blocks.entry(address(rule)).or_insert(text);
        }
    }

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
            examples: self.examples.clone(),
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
        self.examples = at.examples;
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
        if !self.plain() {
            wrap(&self.dialect, start, out);
        }
    }

    /// The text of the `only` marker a gated rule's part carries.
    pub(super) fn gate_text(&self, common: &RuleCommon) -> Option<String> {
        (!self.plain() && common.only.is_gated())
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
