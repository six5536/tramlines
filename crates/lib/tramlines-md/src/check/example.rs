//! A rule's `example` against the rule that carries it, and a format's
//! against the format (grammar `section`, `block`, `unit`,
//! `frontmatter-constraint`, `format`).
//!
//! The instance voice: text of the kind a rule demands. A scaffold
//! is built from these, and checking each one where it stands catches the
//! example on an optional rule that no scaffold reaches, and the example a
//! later edit to its rule left behind.

use std::{borrow::Cow, ops::Range};

use super::{
    At, Host, NullHost,
    derived::{derived, derived_unit},
    label,
};
use crate::{
    Finding, Locator, RepoPath,
    grammar::{
        Block, FmFormat, Format, FormatMatch, FormatRegistry, FormatSource, FrontmatterConstraint,
        Label, Schema, Section, Shape, Squashed, Text, Unit, Words, captured, carries_labels, read,
        read_row, refused_masked, row_code_spans, row_reading, squash,
    },
    markdown::{self, BlockKind, Reading, is_slug, is_url},
    text,
};

/// What one example failed, as the reason alone.
type Problem = Option<String>;

/// The schema every example of one schema is reported on.
struct Examples<'a> {
    schema: &'a Schema,
    path: &'a RepoPath,
    out: Vec<Finding>,
    /// The examples that failed.
    failed: Failed,
    /// The schema, which a finding of a rule's example names.
    locator: Locator,
    /// Where in the schema the rule being read sits.
    at: At,
    /// The labels of the schema in the words of the host.
    labels: Words,
}

/// Every `example` a schema carries, against the rule it sits on. The
/// finding of a rule's example names the schema `id` and the rule. The
/// label of an example is held to the keys of the schema's labels, as a
/// host that gives no words for them holds it.
pub fn check(schema: &Schema, path: &RepoPath, id: &str) -> Vec<Finding> {
    check_examples(schema, &NullHost, path, id).0
}

/// The examples that failed their rules: the text of each, and the line
/// of each rule whose example failed.
#[derive(Debug, Default)]
pub(super) struct Failed {
    pub texts: Vec<String>,
    pub rules: Vec<usize>,
}

/// The findings of `check`, and the examples that failed, the label of
/// each example held to the words that `host` gives.
// sokf-implements: TL_64
// sokf-implements: TL_65
// sokf-implements: TL_73
// sokf-implements: TL_127
pub(super) fn check_examples(
    schema: &Schema,
    host: &dyn Host,
    path: &RepoPath,
    id: &str,
) -> (Vec<Finding>, Failed) {
    let mut e = Examples {
        schema,
        path,
        out: Vec::new(),
        failed: Failed::default(),
        locator: Locator::schema(id),
        at: At::default(),
        labels: label::words_of(schema, host),
    };
    for (key, constraint) in &schema.core.frontmatter {
        if let Some(text) = &constraint.example {
            e.at = At {
                named: Some(format!("key `{key}`")),
                ..At::default()
            };
            e.said_of(
                text,
                constraint.common.line,
                "TL_64",
                frontmatter(key, constraint),
            );
        }
    }
    for section in &schema.sections {
        e.at = At {
            region: Some(format!("section \"{}\"", section.label())),
            rule_id: section.rule_id.clone(),
            ..At::default()
        };
        if let Some(text) = &section.heading.example {
            e.said_of(text, section.common.line, "TL_64", heading(section));
            if let (Some(kd), Some(_)) = (&section.heading.key_derives_from, &section.heading.key) {
                let problem = derived(schema, kd, section.example_key(), markdown::reading);
                e.said_of(text, section.common.line, "TL_127", problem);
            }
        }
        e.blocks(section.blocks.as_deref());
    }
    if let Some(preamble) = &schema.preamble {
        e.at = At {
            region: Some("preamble".to_string()),
            ..At::default()
        };
        e.blocks(preamble.blocks.as_deref());
    }
    // A format names itself in its finding.
    e.at = At::default();
    // An entry of a farther layer sits in another file, so its finding
    // goes on the schema's own line and says which layer wrote it. A
    // formats file that the caller names, and a formats file of a pack, is
    // checked on that file, with `check_formats`; its failed examples still
    // stand for the scaffold.
    // sokf-implements: MD-6_AC-8
    // A format's example reads as the rules that bind the format read a
    // unit: as prose, in its reading, against the format as read, as a row,
    // cell by cell, or as a line of a fence, as written (TLMD_259).
    let bound = schema.bound_formats();
    for (name, (format, source)) in &schema.core.formats.entries {
        let reads: Vec<Reads> = [
            (bound.prose.contains(name), Reads::Prose),
            (bound.rows.contains(name), Reads::Row),
            (bound.lines.contains(name), Reads::Lines),
        ]
        .into_iter()
        .filter_map(|(bound, reads)| bound.then_some(reads))
        .collect();
        let own = matches!(source, FormatSource::Schema(_));
        let layer = if own {
            String::new()
        } else {
            format!(" ({})", source.label())
        };
        for (criterion, message, text, line) in format_failures(name, format, &layer, &reads) {
            let at = if own { line } else { schema.core.line };
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                e.failed.texts.push(text);
            }
            // sokf-implements: MD-6_AC-8
            if !matches!(source, FormatSource::File(_) | FormatSource::Pack(_)) {
                e.out
                    .push(Finding::error(path.clone(), Some(at), criterion, message));
            }
        }
    }
    (e.out, e.failed)
}

/// The example of each format of `registry` that `source` wrote, against
/// its format, reported on `path` at the line of the format or its variant.
// sokf-implements: TL_43
// sokf-implements: TL_65
pub fn check_formats(
    registry: &FormatRegistry,
    source: &FormatSource,
    path: &RepoPath,
) -> Vec<Finding> {
    registry
        .entries
        .iter()
        .filter(|(_, (_, s))| s == source)
        .flat_map(|(name, (format, _))| {
            // A file of formats names no rule that binds them, so an example
            // fails only where it fails both as prose and as a line.
            format_failures(name, format, "", &[]).into_iter().map(
                |(criterion, message, _, line)| {
                    Finding::error(path.clone(), Some(line), criterion, message)
                },
            )
        })
        .chain(super::spelling::check_formats(registry, source, path))
        .collect()
}

/// How a rule that binds a format reads a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reads {
    /// A rule of prose other than a table row: in its reading, as a
    /// paragraph alone.
    Prose,
    /// A rule of a table row: in its reading, cell by cell.
    Row,
    /// A rule of the lines of a fence: as written.
    Lines,
}

impl Reads {
    /// A text as such a rule reads it, with the map of its bytes.
    fn reading(self, text: &str) -> Reading {
        match self {
            Reads::Prose => markdown::reading(text),
            Reads::Row => row_reading(text),
            Reads::Lines => Reading::written(text),
        }
    }

    /// The format as such a rule reads it: its shapes as read, or as
    /// written for a line of a fence.
    fn format(self, format: &Format) -> Cow<'_, Format> {
        match self {
            Reads::Prose => Cow::Owned(format.read_with(&read)),
            Reads::Row => Cow::Owned(format.read_with(&read_row)),
            Reads::Lines => Cow::Borrowed(format),
        }
    }

    /// The code spans of `text`, an example as such a rule reads it: those
    /// of each cell alone for a row, those of the text as one line for other
    /// prose, and none for a line of a fence, whose backticks are text.
    fn code_spans(self, text: &str) -> Vec<Range<usize>> {
        match self {
            Reads::Row => row_code_spans(text),
            Reads::Prose => markdown::code_spans_in(text),
            Reads::Lines => Vec::new(),
        }
    }
}

/// One failure of an example of a format: its criterion, its message, the
/// text of the example, and the line of the format or of its variant.
type FormatFailure = (&'static str, String, Option<String>, usize);

/// The failures of the examples of the format `name`, as each rule that
/// binds it reads them, `reads`, each against the format as that rule reads
/// it, one failure for each message, or, where no rule is known to bind it,
/// its failures as prose only where it fails as a line too.
fn format_failures(
    name: &str,
    format: &Format,
    layer: &str,
    reads: &[Reads],
) -> Vec<FormatFailure> {
    let failures = |r: Reads| format_examples(name, &r.format(format), layer, r);
    if reads.is_empty() {
        let failed = failures(Reads::Prose);
        return if failed.is_empty() || failures(Reads::Lines).is_empty() {
            Vec::new()
        } else {
            failed
        };
    }
    let mut failed: Vec<FormatFailure> = Vec::new();
    for &r in reads {
        for f in failures(r) {
            if !failed.iter().any(|g| g.1 == f.1) {
                failed.push(f);
            }
        }
    }
    failed
}

/// The failures of the examples of the format `name`: each criterion, its
/// message, the text of an example that failed its shape, and the line of
/// the format, or of the variant whose example it is. `layer` names a
/// farther layer in the message.
fn format_examples(name: &str, format: &Format, layer: &str, reads: Reads) -> Vec<FormatFailure> {
    // The example as the rules read it, mapped to the example as written,
    // which a failure quotes.
    let read_line = |text: &str| written(text, &reads.reading(text));
    let mut out = Vec::new();
    let line_of = |variant: Option<&str>| {
        variant
            .and_then(|v| format.raw.get("variants")?.get(v))
            .map_or(format.raw.line, |node| node.line)
    };
    // An example is written as its line stands, a fence's indent and all;
    // every shape reads it squashed (TL_23). A variant's example speaks for
    // its own variant, so it is held to that one first.
    let examples: Vec<(Option<(&str, &Shape)>, &str)> = match &format.example {
        Some(example) => vec![(None, example.as_str())],
        None => format
            .variants
            .iter()
            .map(|(vi, v)| (Some((vi.as_str(), &v.shape)), v.example.as_str()))
            .collect(),
    };
    for (own, line) in examples {
        let quoted = read_line(line);
        let text = quoted.squashed.text.clone();
        // The bans leave out the code spans of the example as the rule
        // reads it: of a paragraph or of a row, and none of a line of a
        // fence.
        // sokf-implements: TLMD_111
        let code = reads.code_spans(&text);
        let masked = markdown::mask_code(&text, &code);
        let checked = quoted.quoting(|q| format.check_quoting(&text, &masked, "line", q));
        let problem = match (own, checked) {
            (Some((vi, shape)), _) if !shape.matches(&text, |_| None) => Some(format!(
                "the example {} of variant `{vi}` of format `{name}`{layer} does not match its own variant: it {}", text::code(line),
                quoted.quoting(|q| shape.failure_quoting(&text, q))
            )),
            // The failure of a format shows its examples, which here are
            // the one that failed, so the example check leaves them out.
            (_, FormatMatch::Fail(reason) | FormatMatch::Refused(reason)) => {
                let reason = without_example(&reason);
                Some(format!("the example {} of format `{name}`{layer} {reason}", text::code(line)))
            }
            // The example speaks for the key as well as the shape, so one
            // that captures no key fails its format.
            // sokf-implements: TL_65
            (_, FormatMatch::Ok(None)) => format.key.as_ref().map(|key| {
                format!(
                    "the example {} of format `{name}`{layer} carries no key: the format captures `{key}`", text::code(line)
                )
            }),
            (_, FormatMatch::Ok(Some(_))) => None,
        };
        if let Some(message) = problem {
            let failed = squash(line);
            out.push((
                "TL_65",
                message,
                Some(failed),
                line_of(own.map(|(vi, _)| vi)),
            ));
        }
    }
    // Each variant's example tells it from the others, so a line is the
    // variant it reads as, and a failure names the one it meant. An example
    // that fails its own variant is reported above.
    // sokf-implements: TL_43
    // sokf-implements: TLMD_58
    for (i, (vi, variant)) in format.variants.iter().enumerate() {
        let text = read_line(&variant.example).squashed.text;
        if !variant.shape.matches(&text, |_| None) {
            continue;
        }
        for (j, (vj, other)) in format.variants.iter().enumerate() {
            if i != j && other.shape.matches(&text, |_| None) {
                out.push((
                    "TL_43",
                    format!(
                        "the example of variant `{vi}` of format `{name}`{layer} also matches variant `{vj}`: each variant's example matches its own variant only"
                    ),
                    None,
                    line_of(Some(vi)),
                ));
            }
        }
    }
    out
}

impl Examples<'_> {
    /// `say`, for the example `text`, which is kept where it failed.
    fn said_of(&mut self, text: &str, line: usize, criterion: &'static str, problem: Problem) {
        let text = squash(text);
        if problem.is_some() {
            self.failed.rules.push(line);
            if !text.is_empty() {
                self.failed.texts.push(text);
            }
        }
        self.say(line, criterion, problem);
    }

    fn say(&mut self, line: usize, criterion: &'static str, problem: Problem) {
        if let Some(reason) = problem {
            let locator = self.at.locate(&self.locator);
            self.out.push(
                Finding::error(self.path.clone(), Some(line), criterion, reason).at(&locator),
            );
        }
    }

    /// Read `f` with the place narrowed to `rule_id` or to the block at
    /// `pos`, as the check names a block's rule, and put it back after: a
    /// block names its own `rule-id` or its place, and a unit rule its own
    /// `rule-id` or its block's.
    fn inside(&mut self, pos: Option<usize>, rule_id: Option<&String>, f: impl FnOnce(&mut Self)) {
        let mut at = self.at.clone();
        at.blocks.extend(pos);
        if pos.is_some() || rule_id.is_some() {
            at.rule_id = rule_id.cloned();
        }
        let held = std::mem::replace(&mut self.at, at);
        f(self);
        self.at = held;
    }

    /// Every block of a list, and the unit rules under each.
    fn blocks(&mut self, rules: Option<&[Block]>) {
        for block in rules.unwrap_or(&[]) {
            self.inside(Some(block.pos), block.rule_id.as_ref(), |e| e.block(block));
        }
    }

    /// One block, and the unit rules under it.
    fn block(&mut self, block: &Block) {
        // The rules of a fence or of an HTML block govern its lines, whose
        // backticks are text and open no code span.
        let lines = matches!(block.kind, BlockKind::Code | BlockKind::Html);
        // A block's example stands for one of its units, so it is held
        // to the rules that would take it: those that state no text of
        // their own. A block whose rules each carry an example, and one
        // that declares no unit rule at all, answer to nothing here.
        let consumers = consumers(block);
        for (i, text) in block.examples.iter().enumerate() {
            for rule in &consumers {
                let problem = item(i, &block.examples, self.unit(block, rule, text, lines));
                self.said_of(text, block.common.line, "TL_64", problem);
                let problem = item(
                    i,
                    &block.examples,
                    derived_unit(self.schema, &self.labels, block, rule, text),
                );
                self.said_of(text, block.common.line, "TL_127", problem);
            }
            // A block with no unit rule holds its units to its own `label`.
            if block.units().next().is_none() {
                let (bare, found) = self.label_of(block, None, text);
                let problem = self.label_problem(block, None, text, &bare, found.as_ref());
                self.said_of(
                    text,
                    block.common.line,
                    "TL_64",
                    item(i, &block.examples, problem),
                );
            }
        }
        for rule in block.units() {
            self.units(block, rule, lines);
        }
        if let Some(rules) = &block.rules {
            self.grammar(block, rules);
        }
        self.blocks(block.blocks.as_deref());
    }

    /// One unit rule, the rules under its `lines`, and an item's blocks.
    /// `lines` says that the rule governs the lines of a fence or of an
    /// HTML block.
    fn units(&mut self, block: &Block, rule: &Unit, lines: bool) {
        for (i, text) in rule.examples.iter().enumerate() {
            let problem = item(i, &rule.examples, self.unit(block, rule, text, lines));
            self.inside(None, rule.rule_id.as_ref(), |e| {
                e.said_of(text, rule.common.line, "TL_64", problem);
            });
        }
        // The example that the scaffold writes for the rule, its own or else
        // its format's, carries the key that derives.
        let examples = examples_of(self.schema, rule);
        for (i, text) in examples.iter().enumerate() {
            let problem = item(
                i,
                &examples,
                derived_unit(self.schema, &self.labels, block, rule, text),
            );
            self.inside(None, rule.rule_id.as_ref(), |e| {
                e.said_of(text, rule.common.line, "TL_127", problem);
            });
        }
        for line in rule.lines.iter().flatten() {
            self.units(block, line, lines);
        }
        if let Some(lines) = &rule.lines {
            self.grammar(block, lines);
        }
        self.blocks(rule.blocks.as_deref());
    }

    /// A rule list is a grammar, so each production's example is taken by
    /// its own rule and by no earlier one.
    // sokf-implements: TLMD_58
    // sokf-implements: TL_43
    // sokf-implements: TLMD_58
    fn grammar(&mut self, block: &Block, rules: &[Unit]) {
        for (i, rule) in rules.iter().enumerate() {
            if !rule.is_production() {
                continue;
            }
            let examples = examples_of(self.schema, rule);
            for (n, text) in examples.iter().enumerate() {
                if let Some(earlier) = rules[..i].iter().position(|r| {
                    r.is_production() && has_shape(self.schema, &self.labels, block, r, text)
                }) {
                    let problem = Some(format!(
                        "the example {} is taken by the rule at line {} first, so this rule takes nothing: order the rules from the narrowest",
                        text::code(text),
                        rules[earlier].common.line
                    ));
                    // The finding is about the later rule, so it names it.
                    // sokf-implements: TL_73
                    self.inside(None, rule.rule_id.as_ref(), |e| {
                        e.say(rule.common.line, "TLMD_58", item(n, &examples, problem));
                    });
                }
            }
        }
    }
}

/// A problem with one text of an `example` list, with the position of the
/// text in the list where the list holds more than one, so that the author
/// knows which text fails.
fn item(i: usize, examples: &[String], problem: Problem) -> Problem {
    let reason = problem?;
    if examples.len() > 1 {
        Some(format!("item {} of `example`: {reason}", i + 1))
    } else {
        Some(reason)
    }
}

/// The texts a rule's example gives: its own, or else its format's first.
fn examples_of(schema: &Schema, rule: &Unit) -> Vec<String> {
    if !rule.examples.is_empty() {
        return rule.examples.clone();
    }
    rule.format
        .as_ref()
        .and_then(|n| schema.formats_of(rule).get(n))
        .map(|f| vec![f.first_example().to_string()])
        .unwrap_or_default()
}

/// Whether `text`, an example of a unit of `block`, has `rule`'s shape,
/// its own or its format's, read without its label in `words`.
fn has_shape(schema: &Schema, words: &Words, block: &Block, rule: &Unit, text: &str) -> bool {
    let (bare, _) = schema.unlabelled(words, block, Some(rule), &rule.read_text(text));
    rule.fits(schema.formats_of(rule), &bare, |_| None)
}

impl Examples<'_> {
    /// The example `text` of a unit of `block` that `rule` governs, as the
    /// rule reads it: squashed, without the label that it carries, and the
    /// label (`TLMD_257`).
    fn label_of(
        &self,
        block: &Block,
        rule: Option<&Unit>,
        text: &str,
    ) -> (String, Option<crate::grammar::Found>) {
        self.schema.unlabelled(&self.labels, block, rule, text)
    }

    /// The label `found` that the example `text` carries, as `bare` reads
    /// without it, against what `rule` of `block` declares and the words of
    /// the host, as a unit's label is held to them (`TLMD_257`).
    // sokf-implements: TLMD_257
    // sokf-implements: MD-6_AC-10
    fn label_problem(
        &self,
        block: &Block,
        rule: Option<&Unit>,
        text: &str,
        bare: &str,
        found: Option<&crate::grammar::Found>,
    ) -> Problem {
        if self.labels.is_empty() || !carries_labels(block.kind) {
            return None;
        }
        let listed = self.labels.listed();
        match (block.label_of(rule), found) {
            (Some(Label::Required), None) => Some(format!(
                "the example {} carries no label, and its rule requires one: {listed}",
                text::code(text)
            )),
            (None, Some(f)) => Some(format!(
                "the example {} carries the label `[{}]`, and its rule declares no `label`",
                text::code(bare),
                f.words
            )),
            (Some(_), Some(f)) if !self.labels.accepts(&f.words) => Some(format!(
                "the label `[{}]` of the example {} is none of the labels of the schema: {listed}",
                f.words,
                text::code(bare)
            )),
            _ => None,
        }
    }

    /// The example `text` of a unit of `block` against `rule`, as `unit`
    /// holds it, read without its label, and then its label.
    fn unit(&self, block: &Block, rule: &Unit, text: &str, lines: bool) -> Problem {
        if lines {
            return unit(self.schema, rule, text, &squash(text), None, lines);
        }
        // An example reads as its rule reads a unit (TLMD_259).
        let (bare, found) = self.label_of(block, Some(rule), &rule.read_text(text));
        // A failure quotes the text that its column counts in: the example
        // as written, mapped from its reading, or, where it carries a
        // label, the reading without it.
        let shown = if found.is_some() { bare.as_str() } else { text };
        let quote = found
            .is_none()
            .then(|| written(text, &rule.reading_of(text)));
        unit(self.schema, rule, shown, &bare, quote.as_ref(), lines)
            .or_else(|| self.label_problem(block, Some(rule), text, &bare, found.as_ref()))
    }
}

/// A unit rule's example against that rule's text constraints. A guard
/// comes first: a rule governs what it selects, so an example the guard
/// passes over stands for a unit the rule never reads. `lines` says that
/// the example stands for a line of a fence or of an HTML block. `bare` is
/// the example squashed, without its label, which the rule reads.
fn unit(
    schema: &Schema,
    rule: &Unit,
    text: &str,
    bare: &str,
    quote: Option<&label::Read>,
    lines: bool,
) -> Problem {
    let squashed = bare.to_string();
    if let Some(select) = &rule.select
        && !select.is_match(&squashed)
    {
        return Some(format!(
            "the example {} does not match `select-pattern` {}, so this rule would not govern it",
            text::code(text),
            text::code(select.as_str())
        ));
    }
    if let Some(shape) = rule.matcher()
        && !matches!(shape, Shape::Text(Text::From(_)))
        && !shape.matches(&squashed, |_| None)
    {
        // A text quotes the literal as the schema writes it.
        let written = rule.shape.as_ref().filter(|s| matches!(s, Shape::Text(_)));
        let failure = match (written, quote) {
            (Some(text), _) => text.failure(&squashed),
            (None, Some(q)) => q.quoting(|q| shape.failure_quoting(&squashed, q)),
            (None, None) => shape.failure(&squashed),
        };
        return Some(format!("the example {} {failure}", text::code(text)));
    }
    // The example stands for a unit, so the bans of its rule and of its
    // format leave out its code spans, as a unit's do; a line holds none.
    // sokf-implements: TLMD_111
    let format = rule.format_in(schema.formats_of(rule));
    let code = if lines || !rule.bans_in(schema.formats_of(rule)) {
        Vec::new()
    } else if rule.reads_row {
        row_code_spans(&squashed)
    } else {
        markdown::code_spans_in(&squashed)
    };
    let masked = markdown::mask_code(&squashed, &code);
    if let Some(reason) =
        refused_masked(&squashed, &masked, &rule.prohibited_patterns, "line", None)
    {
        return Some(format!(
            "the example {} is refused: {reason}",
            text::code(text)
        ));
    }
    if let Some(name) = &rule.format {
        match format.map(|f| (f, f.check_masked(&squashed, &masked, "line"))) {
            // The failure shows the format's example, which the finding
            // names already where it is the rule's.
            Some((f, FormatMatch::Fail(reason) | FormatMatch::Refused(reason))) => {
                let reason = if f.examples().contains(&text) {
                    without_example(&reason)
                } else {
                    &reason
                };
                return Some(format!("the example {} {reason}", text::code(text)));
            }
            Some((f, FormatMatch::Ok(None))) if f.key.is_some() && !rule.key_optional => {
                return Some(format!(
                    "the example {} carries no key: the format `{name}` captures `{}`",
                    text::code(text),
                    f.key.clone().unwrap_or_default()
                ));
            }
            _ => {}
        }
    }
    if let Some(k) = &rule.key
        && !rule.key_optional
        && let Some(shape) = rule.matcher()
        && captured(shape, k, &squashed).is_none()
    {
        return Some(format!(
            "the example {} captures no `key` `{k}`",
            text::code(text)
        ));
    }
    None
}

/// `reason`, the failure of a format, without the example of the format
/// that it ends with, where a finding names that example already.
fn without_example(reason: &str) -> &str {
    reason
        .rsplit_once("; a line of this kind reads ")
        .or_else(|| reason.rsplit_once(" A line of this kind reads "))
        .map_or(reason, |(why, _)| why)
}

/// The unit rules of `block` that would take the block's own `example`:
/// the ones that state no text of their own, which is the order the
/// derivation reads (`text`, then `example`, then a format's, then the
/// block's).
fn consumers(block: &Block) -> Vec<&Unit> {
    block
        .units()
        .filter(|r| {
            !matches!(r.shape, Some(Shape::Text(_))) && r.examples.is_empty() && r.format.is_none()
        })
        .collect()
}

/// `text`, an example as written, and `reading`, its reading, as a check
/// reads a unit: squashed, the reading mapped to the text, by which a
/// failure quotes the example as written.
fn written(text: &str, reading: &Reading) -> label::Read {
    label::Read {
        squashed: Squashed::mapped(&reading.text, &reading.map),
        label: None,
        shown: Squashed::new(text),
    }
}

/// A section's example against its heading's shape.
fn heading(section: &Section) -> Problem {
    let text = section.heading.example.as_ref()?;
    let shape = &section.heading.read;
    let quote = written(text, &markdown::reading(text));
    let read = quote.squashed.text.as_str();
    if matches!(shape, Shape::Text(_)) || shape.matches(read, |_| None) {
        return None;
    }
    let failure = quote.quoting(|q| shape.failure_quoting(read, q));
    Some(format!("the example {} {failure}", text::code(text)))
}

/// A frontmatter constraint's example against that constraint. The two
/// repository-dependent formats are left alone, as they are in a scaffold.
fn frontmatter(key: &str, constraint: &FrontmatterConstraint) -> Problem {
    let text = constraint.example.as_ref()?;
    let said = |reason: String| Some(format!("the example of `{key}` {reason}"));
    if let Some(c) = &constraint.const_
        && c != text
    {
        return said(format!("is not the `const` {c}", c = text::code(c)));
    }
    if let Some(shape) = &constraint.shape
        && constraint.id_template.is_none()
        && !shape.matches(&squash(text), |_| None)
    {
        return said(shape.failure(&squash(text)));
    }
    if let Some(values) = &constraint.enum_
        && !values.contains(text)
    {
        return said(format!("is not one of {}", text::code_list(values, ", ")));
    }
    match constraint.format {
        Some(FmFormat::Slug) if !is_slug(text) => return said("is not a slug".into()),
        Some(FmFormat::Url) if !is_url(text) => return said("is not a URL".into()),
        _ => {}
    }
    if let Some(template) = &constraint.id_template
        && !template.matches(text)
    {
        return said("does not match the id template".into());
    }
    None
}
