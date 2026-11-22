//! The state one schema check carries, and `check_against`, which runs
//! each check in order.

use std::{
    collections::HashSet,
    ops::{Range, RangeInclusive},
};

use super::{
    Accepted, Capture, CheckOutput, DocView, Gate, Host, Repair, anchors, frontmatter, keys, label,
    limits, markers, sections,
};
use crate::{
    Finding, Locator,
    grammar::{
        Block as Rule, LifecycleDimension, RuleCommon, Schema, Section, States, Text,
        Unit as UnitRule, Words, squash,
    },
    markdown::{Block, Heading, Unit},
};

// sokf:begin check-against
/// How a check runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct CheckOptions {
    /// Whether a scaffold is checked: it has no anchors until they are
    /// written, so no anchor finding is reported.
    pub scaffold: bool,
}
// sokf:end check-against

/// A heading matched to a declared section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Occurrence {
    /// The declared section, by its index among the sections admitted.
    pub section: usize,
    /// The heading's line.
    pub line: usize,
    /// The heading's level.
    pub level: u8,
    /// The heading's text.
    pub text: String,
}

/// A unit that breaks its rule (`TLMD_109`), by where it holds its anchors.
/// A unit in the body of a heading that no declared section takes, while the
/// heading breaks its rule, has the record of its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Broken {
    /// On the lines of its text: a paragraph, an item, or a row.
    Text(RangeInclusive<usize>),
    /// On the lines of its text: an item that holds a block its rule
    /// refuses, after its text or in place of it. A repair of the text
    /// leaves the block, so no repair sets the record aside.
    Beyond(RangeInclusive<usize>),
    /// On the line of anchors before the code or HTML block that starts on
    /// this line, of which the unit is a line.
    Before(usize),
    /// On the lines of a heading that captures no key and on which the
    /// match of the headings reports a finding that no repair of the
    /// heading corrects. That finding outlasts every repair of the heading,
    /// so no repair sets the record aside.
    Heading(RangeInclusive<usize>),
}

/// Where in a schema the rule being checked sits. The reader a finding
/// sends to the schema lands in a schema of hundreds of lines, so the
/// locator carries this beside the schema's id.
#[derive(Debug, Clone, Default)]
pub(crate) struct At {
    /// `section "Overview"`, or `preamble`.
    pub region: Option<String>,
    /// The declared position of each block entered, one-based and
    /// outermost first, so a group's inner block reads `block 1.2`.
    pub blocks: Vec<usize>,
    /// The `rule-id` nearest the finding, which names the rule alone.
    pub rule_id: Option<String>,
    /// A frontmatter key or a link's `rel`, where neither applies.
    pub named: Option<String>,
    /// The line in the schema of the rule nearest the finding.
    pub line: Option<usize>,
}

impl At {
    /// The locator of a declared section.
    pub(crate) fn section(section: &Section) -> At {
        At {
            region: Some(format!("section \"{}\"", section.label())),
            rule_id: section.rule_id.clone(),
            line: Some(section.common.line),
            ..At::default()
        }
    }

    /// The locator of the rule that made a capture, in the section that
    /// holds it among `sections`.
    pub(crate) fn capture(cap: &Capture, sections: &[&Section]) -> At {
        let section = cap.section.and_then(|j| sections.get(j));
        At {
            region: section.map(|s| format!("section \"{}\"", s.label())),
            rule_id: cap.rule_name.clone(),
            line: section.map(|s| s.common.line),
            ..At::default()
        }
    }

    /// `locator` narrowed to where this sits, where anything was entered.
    pub(crate) fn locate(&self, locator: &Locator) -> Locator {
        match self.render() {
            Some(at) => locator.at(at),
            None => locator.clone(),
        }
    }

    /// The locator, or `None` where nothing was entered.
    pub(crate) fn render(&self) -> Option<String> {
        if let Some(id) = &self.rule_id {
            return Some(format!("rule `{id}`"));
        }
        if let Some(n) = &self.named {
            return Some(n.clone());
        }
        let mut parts: Vec<String> = self.region.clone().into_iter().collect();
        if !self.blocks.is_empty() {
            let at: Vec<String> = self.blocks.iter().map(ToString::to_string).collect();
            parts.push(format!("block {}", at.join(".")));
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

/// The state one schema check carries.
pub(crate) struct Checker<'a> {
    pub doc: DocView<'a>,
    pub schema: &'a Schema,
    /// The sections that `possible` admits, in declared order. A section
    /// is named by its index here.
    pub sections: Vec<&'a Section>,
    /// Each admitted section's `parent`, by its index in `sections`.
    pub parent_of: Vec<Option<usize>>,
    /// The document's own states, which decide whether a part is required.
    pub states: &'a States,
    /// The states that the document could be in, which decide whether a
    /// part is declared: its own, with every state of each dimension for
    /// which it holds no state that the lifecycle declares.
    pub possible: States,
    pub host: &'a dyn Host,
    pub options: CheckOptions,
    /// The schema that the findings name.
    pub locator: Locator,
    pub out: CheckOutput,
    /// The `rule-id` of the enclosing group block, while inside one.
    pub group_rule: Option<String>,
    /// Where the rule being checked sits.
    pub at: At,
    /// Every heading matched to a declared section, in document order.
    pub occurrences: Vec<Occurrence>,
    /// The lines of each body whose heading repeats where its section does
    /// not (`TLMD_151`), after the lines of the body that it repeats.
    pub repeats: Vec<(Range<usize>, Range<usize>)>,
    /// The units at the end of the run being claimed that a later group's
    /// opening position needs, which the position leaves to it (`TLMD_193`).
    pub reserve: usize,
    /// Whether a group is trying the positions after it on a block, in a
    /// trial that it rolls back: a group inside the trial keeps a needed
    /// repetition without a trial of its own, so the trials stay as many
    /// as the positions (`TLMD_71`).
    pub trying: bool,
    /// The first line of the code or HTML block whose lines are being
    /// checked: a key captured there has its anchor before the block.
    pub lines_block: Option<usize>,
    /// Whether the host asks for each unit a unit rule accepts.
    pub records_accepted: bool,
    /// Each unit that breaks its rule, whose anchors the anchor check
    /// leaves alone.
    pub broken: Vec<Broken>,
    /// The labels of the schema, in the words that the host gives.
    pub labels: Words,
    /// The label of each link reference definition of the document, in
    /// lower case, which no label's words can be (`TLMD_243`).
    pub links: HashSet<String>,
}

impl Checker<'_> {
    /// The lines an occurrence spans: from its heading to the next heading
    /// of its level or higher, which takes in its subsections.
    pub fn span(&self, o: &Occurrence) -> Range<usize> {
        self.span_from(o.line, o.level)
    }

    /// The lines from the heading at `line` to the next heading of `level`
    /// or higher.
    pub fn span_from(&self, line: usize, level: u8) -> Range<usize> {
        let end = self
            .doc
            .body
            .headings
            .iter()
            .find(|h| h.line > line && h.level <= level)
            .map_or(usize::MAX, |h| h.line);
        line..end
    }

    /// The scalar value of the frontmatter `key`, as text.
    pub fn front_value(&self, key: &str) -> Option<String> {
        self.doc
            .frontmatter
            .and_then(|f| f.get(key))
            .and_then(tramlines_core::Spanned::scalar_text)
    }

    /// The text a `text` rule demands: its literal, or the value of the
    /// frontmatter key it is bound to.
    pub fn text_of(&self, text: &Text) -> Option<String> {
        match text {
            Text::Literal(t) => Some(t.clone()),
            Text::From(key) => self.front_value(key),
        }
    }

    /// The schema, and where in it the rule being checked sits.
    // sokf-implements: TL_73
    fn locator(&self) -> Locator {
        self.at.locate(&self.locator)
    }

    /// Run `f` with the locator narrowed, and put it back afterwards.
    pub fn located<R>(&mut self, at: At, f: impl FnOnce(&mut Self) -> R) -> R {
        let held = std::mem::replace(&mut self.at, at);
        let out = f(self);
        self.at = held;
        out
    }

    /// A finding for a rule, at the rule's `severity`. The text is the one
    /// the check built: the engine's words, and only the texts of the
    /// schema that a criterion quotes, such as the message of a ban or the
    /// `when-at-limit` of a hard limit.
    // sokf-implements: TL_77
    // sokf-implements: MD-4_AC-2
    // sokf-implements: MD-4_AC-3
    // sokf-implements: MD-4_AC-4
    pub fn emit(
        &mut self,
        rule: &RuleCommon,
        line: Option<usize>,
        criterion: &str,
        message: impl Into<String>,
    ) {
        let locator = self.locator();
        self.out.findings.push(
            Finding::new(
                self.doc.path.clone(),
                line,
                rule.severity,
                criterion,
                message,
            )
            .of_rule(rule.line)
            .at(&locator),
        );
    }

    /// Whether a part bound to the frontmatter `key`, which holds no text
    /// for it, is a finding of its own: only where the document does not
    /// set the key, so not where the frontmatter does not read, not where
    /// the value is no scalar (`TL_57`), and not where `TL_54` reports the key
    /// missing.
    // sokf-implements: TLMD_168
    pub fn unbound_is_own_fault(&self, key: &str) -> bool {
        self.doc.frontmatter.is_some_and(|f| f.get(key).is_none())
            && !self
                .schema
                .core
                .frontmatter
                .iter()
                .any(|(k, rule)| k == key && !rule.optional && rule.common.only.admits(self.states))
    }

    /// Whether `section` may be absent from the document: its `optional`
    /// matches the states, or its `only` does not match the document's own
    /// states, so that only the states it could be in declare the section.
    // sokf-implements: TLMD_230
    pub fn section_optional(&self, section: &Section) -> bool {
        section.optional_at(self.states, &self.schema.core.lifecycle)
            || !section.common.only.admits(self.states)
    }

    /// Whether `block` may be absent from its body, as for a section in
    /// [`Checker::section_optional`].
    // sokf-implements: TLMD_230
    pub fn block_optional(&self, block: &Rule) -> bool {
        block.optional_at(self.states, &self.schema.core.lifecycle)
            || !block.common.only.admits(self.states)
    }

    /// Record that a gated rule matched the part at `line`.
    // sokf-implements: MD-4_AC-10
    pub fn gate(&mut self, rule: &RuleCommon, line: usize) {
        if rule.only.is_gated() {
            self.out.gates.push(Gate {
                line,
                only: rule.only.clone(),
                rule: rule.line,
            });
        }
    }

    /// Record that `rule` accepted the unit on `line` to `end_line`, where
    /// the host asks for it.
    // sokf-implements: TLMD_212
    // sokf-implements: MD-4_AC-14
    pub fn accept(&mut self, rule: &UnitRule, line: usize, end_line: usize) {
        self.out.accepted.push(Accepted {
            line,
            end_line,
            rule_index: rule.index,
            rule_id: rule.rule_id.clone(),
        });
    }

    /// Record that `unit` breaks its rule: the check of its rule reported
    /// it, or the check reported that no rule or position took it. The
    /// anchor check sets the record aside where a repair rewrites the unit.
    // sokf-implements: TLMD_109
    pub fn broke(&mut self, unit: &Unit) {
        let broken = self.broken_at(unit);
        self.record(broken);
    }

    /// Record that `unit` holds a block its rule refuses, which a repair
    /// of the text leaves.
    // sokf-implements: TLMD_109
    pub fn broke_beyond_text(&mut self, unit: &Unit) {
        let broken = match self.broken_at(unit) {
            Broken::Text(lines) => Broken::Beyond(lines),
            other => other,
        };
        self.record(broken);
    }

    /// The record of `unit`, by where it holds its anchors.
    fn broken_at(&self, unit: &Unit) -> Broken {
        // As `spot` in `blocks` places the anchor of a key the unit captures:
        // before the block where the text maps to no lines, as a code block
        // does, but on the line of an item whose text is empty.
        match self.lines_block {
            Some(line) => Broken::Before(line),
            None if unit.pieces.is_empty() && !unit.text.is_empty() => Broken::Before(unit.line),
            None => Broken::Text(unit.line..=unit.end_line.max(unit.line)),
        }
    }

    /// Keep `broken`, once for the lines of one fence.
    fn record(&mut self, broken: Broken) {
        if self.broken.last() != Some(&broken) {
            self.broken.push(broken);
        }
    }

    /// Record that `heading` breaks its rule: the match of the heading
    /// reported a finding on it that no repair of the heading corrects.
    // sokf-implements: TLMD_109
    pub fn broke_heading(&mut self, heading: &Heading) {
        self.record(Broken::Heading(heading.line..=heading.end_line));
    }

    /// Record that each unit of `block` breaks its rule, and each unit of
    /// the blocks they hold: the check reported that no position takes
    /// the block, or that no declared section takes the heading above it,
    /// and checks none of them.
    pub fn broke_block(&mut self, block: &Block) {
        for unit in &block.units {
            self.broke(unit);
            for inner in &unit.nested {
                self.broke_block(inner);
            }
        }
    }

    /// A heading rewritten whole. It replaces a repair of the same heading
    /// made before it, as the setext one, because it writes the heading as
    /// ATX too, so one pass reaches the fixed point.
    // sokf-implements: TL_82
    // sokf-implements: TLMD_138
    // sokf-implements: MD-4_AC-7
    pub fn repair_heading(&mut self, repair: Repair) {
        self.out.repairs.retain(|r| r.line != repair.line);
        self.out.repairs.push(repair);
    }

    /// An error the schema implies without a rule to carry a `message`.
    // sokf-implements: MD-4_AC-4
    pub fn error(&mut self, line: Option<usize>, criterion: &str, message: impl Into<String>) {
        let locator = self.locator();
        let finding = Finding::error(self.doc.path.clone(), line, criterion, message);
        self.out
            .findings
            .push(finding.of_rule(self.at.line.unwrap_or(0)).at(&locator));
    }
}

/// Check a document against a schema at its lifecycle `states`.
/// `locator` names the schema, which each finding ends with.
// sokf:begin check-against
pub fn check_against(
    doc: DocView<'_>,
    schema: &Schema,
    states: &States,
    host: &dyn Host,
    options: CheckOptions,
    locator: Locator,
) -> CheckOutput {
    // sokf:end check-against
    let possible = possible(states, &schema.core.lifecycle);
    // sokf-implements: MD-4_AC-12
    let sections: Vec<&Section> = schema
        .sections
        .iter()
        .filter(|s| s.common.only.admits(&possible))
        .collect();
    let parent_of = parents(&sections);
    let mut c = Checker {
        doc,
        schema,
        sections,
        parent_of,
        states,
        possible,
        host,
        options,
        locator,
        out: CheckOutput::default(),
        group_rule: None,
        at: At::default(),
        occurrences: Vec::new(),
        repeats: Vec::new(),
        reserve: 0,
        trying: false,
        lines_block: None,
        records_accepted: host.records_accepted(),
        broken: Vec::new(),
        labels: label::words_of(schema, host),
        links: doc
            .body
            .ref_defs
            .iter()
            .map(|(label, _, _)| squash(label).to_lowercase())
            .collect(),
    };
    // sokf-implements: MD-4_AC-1
    frontmatter::check(&mut c);
    sections::check(&mut c);
    // A block checked out of its place (TLMD_169) captures out of order, so
    // the captures are put back in the order of the document, which says
    // which capture of a key comes first.
    c.out.captures.sort_by_key(|cap| cap.line);
    c.out.accepted.sort_by_key(|a| a.line);
    keys::check(&mut c);
    anchors::check(&mut c);
    limits::check(&mut c);
    markers::check(&mut c);
    // The host's rules run last, so that they can read what the engine
    // captured.
    // sokf-implements: MD-5_AC-12
    c.host
        .check(&c.doc, c.states, c.options, &c.locator, &mut c.out);
    c.out
}

/// The states that a document in `states` could be in: its own, with every
/// state of each dimension for which it holds none that `lifecycle`
/// declares, so that a fault in the state is reported once, on the
/// frontmatter.
// sokf-implements: TLMD_230
fn possible(states: &States, lifecycle: &[LifecycleDimension]) -> States {
    let mut possible = states.clone();
    for dim in lifecycle {
        let known = states
            .0
            .get(&dim.key)
            .is_some_and(|have| have.iter().any(|s| dim.states.contains(s)));
        if !known {
            possible.0.insert(dim.key.clone(), dim.states.clone());
        }
    }
    possible
}

/// Each section's `parent`, by its index among `sections`.
fn parents(sections: &[&Section]) -> Vec<Option<usize>> {
    sections
        .iter()
        .map(|s| {
            let p = s.parent.as_deref()?;
            sections
                .iter()
                .position(|o| o.rule_id.as_deref() == Some(p))
        })
        .collect()
}

/// Shorten a unit's text for a message.
pub(crate) fn short(text: &str) -> String {
    const MAX: usize = 60;
    if text.chars().count() <= MAX {
        return text.to_string();
    }
    let cut: String = text.chars().take(MAX - 1).collect();
    format!("{cut}…")
}
