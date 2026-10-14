//! A rule's `example` against the rule that carries it, and a format's
//! against the format (grammar `section`, `block`, `unit`,
//! `frontmatter-constraint`, `format`).
//!
//! The instance voice: text of the kind a rule demands. A scaffold
//! is built from these, and checking each one where it stands catches the
//! example on an optional rule that no scaffold reaches, and the example a
//! later edit to its rule left behind.

use super::At;
use crate::{
    Finding, Locator, RepoPath,
    grammar::{
        Block, FmFormat, Format, FormatMatch, FormatRegistry, FormatSource, FrontmatterConstraint,
        Schema, Section, Shape, Text, Unit, captured, refused, squash,
    },
    markdown::{is_slug, is_url},
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
}

/// Every `example` a schema carries, against the rule it sits on. The
/// finding of a rule's example names the schema `id` and the rule.
pub fn check(schema: &Schema, path: &RepoPath, id: &str) -> Vec<Finding> {
    check_examples(schema, path, id).0
}

/// The examples that failed their rules: the text of each, and the line
/// of each rule whose example failed.
#[derive(Debug, Default)]
pub(super) struct Failed {
    pub texts: Vec<String>,
    pub rules: Vec<usize>,
}

/// The findings of `check`, and the examples that failed.
// sokf-implements: TL_64
// sokf-implements: TL_65
// sokf-implements: TL_73
pub(super) fn check_examples(schema: &Schema, path: &RepoPath, id: &str) -> (Vec<Finding>, Failed) {
    let mut e = Examples {
        schema,
        path,
        out: Vec::new(),
        failed: Failed::default(),
        locator: Locator::schema(id),
        at: At::default(),
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
    // formats file that the caller names is checked on that file, with
    // `check_formats`; its failed examples still stand for the scaffold.
    // sokf-implements: MD-6_AC-8
    for (name, (format, source)) in &schema.core.formats.entries {
        let own = matches!(source, FormatSource::Schema(_));
        let layer = if own {
            String::new()
        } else {
            format!(" ({})", source.label())
        };
        for (criterion, message, text, line) in format_examples(name, format, &layer) {
            let at = if own { line } else { schema.core.line };
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                e.failed.texts.push(text);
            }
            if !matches!(source, FormatSource::File(_)) {
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
            format_examples(name, format, "")
                .into_iter()
                .map(|(criterion, message, _, line)| {
                    Finding::error(path.clone(), Some(line), criterion, message)
                })
        })
        .collect()
}

/// The failures of the examples of the format `name`: each criterion, its
/// message, the text of an example that failed its shape, and the line of
/// the format, or of the variant whose example it is. `layer` names a
/// farther layer in the message.
fn format_examples(
    name: &str,
    format: &Format,
    layer: &str,
) -> Vec<(&'static str, String, Option<String>, usize)> {
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
        let text = squash(line);
        let problem = match (own, format.check(&text)) {
            (Some((vi, shape)), _) if !shape.matches(&text, |_| None) => Some(format!(
                "the example `{line}` of variant `{vi}` of format `{name}`{layer} does not match its own variant: it {}",
                shape.failure(&text)
            )),
            // The failure of a format shows its examples, which here are
            // the one that failed, so the example check leaves them out.
            (_, FormatMatch::Fail(reason) | FormatMatch::Refused(reason)) => {
                let reason = reason
                    .rsplit_once("; a line of this kind reads ")
                    .map_or(reason.as_str(), |(why, _)| why);
                Some(format!("the example `{line}` of format `{name}`{layer} {reason}"))
            }
            // The example speaks for the key as well as the shape, so one
            // that captures no key fails its format.
            // sokf-implements: TL_65
            (_, FormatMatch::Ok(None)) => format.key.as_ref().map(|key| {
                format!(
                    "the example `{line}` of format `{name}`{layer} carries no key: the format captures `{key}`"
                )
            }),
            (_, FormatMatch::Ok(Some(_))) => None,
        };
        if let Some(message) = problem {
            out.push(("TL_65", message, Some(text), line_of(own.map(|(vi, _)| vi))));
        }
    }
    // Each variant's example tells it from the others, so a line is the
    // variant it reads as, and a failure names the one it meant. An example
    // that fails its own variant is reported above.
    // sokf-implements: TL_43
    // sokf-implements: TLMD_58
    for (i, (vi, variant)) in format.variants.iter().enumerate() {
        let text = squash(&variant.example);
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
    /// `pos`, as the check names a block's rule, and put it back after.
    fn inside(&mut self, pos: Option<usize>, rule_id: Option<&String>, f: impl FnOnce(&mut Self)) {
        let mut at = self.at.clone();
        at.blocks.extend(pos);
        if rule_id.is_some() {
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
        // A block's example stands for one of its units, so it is held
        // to the rules that would take it: those that state no text of
        // their own. A block whose rules each carry an example, and one
        // that declares no unit rule at all, answer to nothing here.
        for (i, text) in block.examples.iter().enumerate() {
            for rule in consumers(block) {
                let problem = item(i, &block.examples, unit(self.schema, rule, text));
                self.said_of(text, block.common.line, "TL_64", problem);
            }
        }
        for rule in block.units() {
            self.units(rule);
        }
        if let Some(rules) = &block.rules {
            self.grammar(rules);
        }
        self.blocks(block.blocks.as_deref());
    }

    /// One unit rule, the rules under its `lines`, and an item's blocks.
    fn units(&mut self, rule: &Unit) {
        for (i, text) in rule.examples.iter().enumerate() {
            let problem = item(i, &rule.examples, unit(self.schema, rule, text));
            self.inside(None, rule.rule_id.as_ref(), |e| {
                e.said_of(text, rule.common.line, "TL_64", problem);
            });
        }
        for line in rule.lines.iter().flatten() {
            self.units(line);
        }
        if let Some(lines) = &rule.lines {
            self.grammar(lines);
        }
        self.blocks(rule.blocks.as_deref());
    }

    /// A rule list is a grammar, so each production's example is taken by
    /// its own rule and by no earlier one.
    // sokf-implements: TLMD_58
    // sokf-implements: TL_43
    // sokf-implements: TLMD_58
    fn grammar(&mut self, rules: &[Unit]) {
        for (i, rule) in rules.iter().enumerate() {
            if !rule.is_production() {
                continue;
            }
            let examples = examples_of(self.schema, rule);
            for (n, text) in examples.iter().enumerate() {
                if let Some(earlier) = rules[..i]
                    .iter()
                    .position(|r| r.is_production() && has_shape(self.schema, r, text))
                {
                    let problem = Some(format!(
                        "the example `{text}` is taken by the rule at line {} first, so this rule takes nothing: order the rules from the narrowest",
                        rules[earlier].common.line
                    ));
                    self.say(rule.common.line, "TLMD_58", item(n, &examples, problem));
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
        .and_then(|n| schema.core.formats.get(n))
        .map(|f| vec![f.first_example().to_string()])
        .unwrap_or_default()
}

/// Whether `text` has `rule`'s shape, its own or its format's.
fn has_shape(schema: &Schema, rule: &Unit, text: &str) -> bool {
    rule.fits(&schema.core.formats, &squash(text), |_| None)
}

/// A unit rule's example against that rule's text constraints. A guard
/// comes first: a rule governs what it selects, so an example the guard
/// passes over stands for a unit the rule never reads.
fn unit(schema: &Schema, rule: &Unit, text: &str) -> Problem {
    let squashed = squash(text);
    if let Some(select) = &rule.select
        && !select.is_match(&squashed)
    {
        return Some(format!(
            "the example `{text}` does not match `select-pattern` `{}`, so this rule would not govern it",
            select.as_str()
        ));
    }
    if let Some(shape) = &rule.shape
        && !matches!(shape, Shape::Text(Text::From(_)))
        && !shape.matches(&squashed, |_| None)
    {
        return Some(format!("the example `{text}` {}", shape.failure(&squashed)));
    }
    if let Some(reason) = refused(&squashed, &rule.prohibited_patterns, None) {
        return Some(format!("the example `{text}` is refused: {reason}"));
    }
    if let Some(name) = &rule.format {
        match rule
            .format_in(&schema.core.formats)
            .map(|f| (f, f.check(&squashed)))
        {
            Some((_, FormatMatch::Fail(reason) | FormatMatch::Refused(reason))) => {
                return Some(format!("the example `{text}` {reason}"));
            }
            Some((f, FormatMatch::Ok(None))) if f.key.is_some() && !rule.key_optional => {
                return Some(format!(
                    "the example `{text}` carries no key: the format `{name}` captures `{}`",
                    f.key.clone().unwrap_or_default()
                ));
            }
            _ => {}
        }
    }
    if let Some(k) = &rule.key
        && !rule.key_optional
        && let Some(shape) = &rule.shape
        && captured(shape, k, &squashed).is_none()
    {
        return Some(format!("the example `{text}` captures no `key` `{k}`"));
    }
    None
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

/// A section's example against its heading's shape.
fn heading(section: &Section) -> Problem {
    let text = section.heading.example.as_ref()?;
    let shape = &section.heading.shape;
    if matches!(shape, Shape::Text(_)) || shape.matches(&squash(text), |_| None) {
        return None;
    }
    Some(format!(
        "the example `{text}` {}",
        shape.failure(&squash(text))
    ))
}

/// A frontmatter constraint's example against that constraint. The two
/// repository-dependent formats are left alone, as they are in a scaffold.
fn frontmatter(key: &str, constraint: &FrontmatterConstraint) -> Problem {
    let text = constraint.example.as_ref()?;
    let said = |reason: String| Some(format!("the example of `{key}` {reason}"));
    if let Some(c) = &constraint.const_
        && c != text
    {
        return said(format!("is not the `const` `{c}`"));
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
        return said(format!("is not one of `{}`", values.join("`, `")));
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
