//! The state one schema check carries, and `check_against`, which runs
//! each check in order.

use std::ops::Range;

use super::{
    Capture, CheckOutput, DocView, Gate, Host, Repair, anchors, frontmatter, keys, limits, markers,
    sections,
};
use crate::{
    Finding, Locator,
    grammar::{RuleCommon, Schema, Section, States, Text},
    markdown::BlockKind,
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
            rule_id: cap.rule_id.clone(),
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
            let at: Vec<String> = self.blocks.iter().map(|b| b.to_string()).collect();
            parts.push(format!("block {}", at.join(".")));
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

/// The state one schema check carries.
pub(crate) struct Checker<'a> {
    pub doc: DocView<'a>,
    pub schema: &'a Schema,
    /// The sections the states admit, in declared order. A section is
    /// named by its index here.
    pub sections: Vec<&'a Section>,
    /// Each admitted section's `parent`, by its index in `sections`.
    pub parent_of: Vec<Option<usize>>,
    pub states: &'a States,
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
    /// not (TLMD_151), after the lines of the body that it repeats.
    pub repeats: Vec<(Range<usize>, Range<usize>)>,
    /// The kinds of block the positions after the one being claimed can
    /// open, so a group knows which blocks it leaves to them.
    pub later: Vec<BlockKind>,
    /// The units at the end of the run being claimed that a later group's
    /// opening position needs, which the position leaves to it (TLMD_193).
    pub reserve: usize,
    /// The first line of the code or HTML block whose lines are being
    /// checked: a key captured there has its anchor before the block.
    pub lines_block: Option<usize>,
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
            .and_then(|v| v.scalar_text())
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
    /// the check built; no schema writes a sentence into it.
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
    /// the value is no scalar (TL_57), and not where TL_54 reports the key
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
    // sokf-implements: MD-4_AC-12
    let sections: Vec<&Section> = schema
        .sections
        .iter()
        .filter(|s| s.common.only.admits(states))
        .collect();
    let parent_of = parents(&sections);
    let mut c = Checker {
        doc,
        schema,
        sections,
        parent_of,
        states,
        host,
        options,
        locator,
        out: CheckOutput::default(),
        group_rule: None,
        at: At::default(),
        occurrences: Vec::new(),
        repeats: Vec::new(),
        later: Vec::new(),
        reserve: 0,
        lines_block: None,
    };
    // sokf-implements: MD-4_AC-1
    frontmatter::check(&mut c);
    sections::check(&mut c);
    // A block checked out of its place (TLMD_169) captures out of order, so
    // the captures are put back in the order of the document, which says
    // which capture of a key comes first.
    c.out.captures.sort_by_key(|cap| cap.line);
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
