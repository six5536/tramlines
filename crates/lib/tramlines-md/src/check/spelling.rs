//! The warning on a pattern that names a spelling which the reading never
//! holds (`TLMD_264`): a pattern reads the reading of a unit, so a run of
//! literal text that reads otherwise, such as `\*Note\*`, never matches.

use std::collections::HashSet;

use regex_syntax::ast::{self, Ast};

use super::At;
use crate::{
    Finding, Locator, RepoPath,
    grammar::{Block, Format, FormatRegistry, FormatSource, Schema, Shape, Unit, read, read_row},
    text,
};

/// The warnings on the patterns of `schema` that read prose: the patterns,
/// the guards, the bans, and the types of the templates of each rule of
/// prose and of each heading, the bans of the document, and the formats of
/// the schema itself, each at the line of its rule. A rule of the lines of
/// a fence reads each line as written, so it draws none.
// sokf-implements: TLMD_264
// sokf-implements: MD-6_AC-11
pub(super) fn check(schema: &Schema, path: &RepoPath, id: &str) -> Vec<Finding> {
    let mut w = Warnings {
        path,
        locator: Locator::schema(id),
        at: At::default(),
        out: Vec::new(),
        seen: HashSet::new(),
        row: false,
    };
    w.at.named = Some("prohibited-patterns".to_string());
    for ban in &schema.core.prohibited_patterns {
        w.pattern(ban.pattern.as_str(), schema.core.line);
    }
    for section in &schema.sections {
        w.at = At::section(section);
        w.shape(&section.heading.shape, section.common.line);
        w.blocks(section.blocks.as_deref());
    }
    if let Some(preamble) = &schema.preamble {
        w.at = At {
            region: Some("preamble".to_string()),
            ..At::default()
        };
        w.blocks(preamble.blocks.as_deref());
    }
    // A format that only the lines of a fence bind reads each line as
    // written, so it draws none, and one that a rule of a table row binds
    // reads cell by cell.
    w.at = At::default();
    let bound = schema.bound_formats();
    for (name, (format, source)) in &schema.core.formats.entries {
        if !matches!(source, FormatSource::Schema(_)) {
            continue;
        }
        let start = w.out.len();
        if bound.prose.contains(name) || !(bound.rows.contains(name) || bound.lines.contains(name))
        {
            w.format(name, format, format.raw.line);
        }
        if bound.rows.contains(name) {
            let prose = w.out.len();
            w.row = true;
            w.format(name, format, format.raw.line);
            w.row = false;
            // A run that reads alike as a row and as other prose draws one
            // warning.
            let rows = w.out.split_off(prose);
            for f in rows {
                if !w.out[start..].contains(&f) {
                    w.out.push(f);
                }
            }
        }
    }
    w.out
}

/// The warnings on the formats of `registry` from `source`, a formats
/// file, on `path`: a format names no kind of block, so it reads prose.
// sokf-implements: TLMD_264
// sokf-implements: MD-6_AC-11
pub(super) fn check_formats(
    registry: &FormatRegistry,
    source: &FormatSource,
    path: &RepoPath,
) -> Vec<Finding> {
    let mut w = Warnings {
        path,
        locator: Locator::schema(""),
        at: At::default(),
        out: Vec::new(),
        seen: HashSet::new(),
        row: false,
    };
    for (name, (format, s)) in &registry.entries {
        if s == source {
            w.format(name, format, format.raw.line);
        }
    }
    // A formats file is no schema, so its findings name no rule.
    for f in &mut w.out {
        *f = Finding::warning(path.clone(), f.line, "TLMD_264", strip_locator(&f.message));
    }
    w.out
}

/// The message of a finding without the locator that ends it.
fn strip_locator(message: &str) -> String {
    message
        .rsplit_once(" (schema ")
        .map_or(message, |(m, _)| m)
        .to_string()
}

/// The walk of a schema for its warnings.
struct Warnings<'a> {
    path: &'a RepoPath,
    locator: Locator,
    at: At,
    out: Vec<Finding>,
    /// The patterns of the types already read, each after whether it was
    /// read as a row.
    seen: HashSet<(bool, String)>,
    /// Whether the rule or the format being read reads a table row, cell
    /// by cell.
    row: bool,
}

impl Warnings<'_> {
    fn blocks(&mut self, blocks: Option<&[Block]>) {
        for block in blocks.unwrap_or(&[]) {
            let held = self.at.clone();
            self.at.blocks.push(block.pos);
            if block.rule_id.is_some() {
                self.at.rule_id.clone_from(&block.rule_id);
            }
            for rule in block.units() {
                self.unit(rule);
            }
            self.blocks(block.blocks.as_deref());
            self.at = held;
        }
    }

    /// A unit rule that reads prose: its shape, its guard, and its bans,
    /// and the blocks of its items.
    fn unit(&mut self, rule: &Unit) {
        let held = self.at.clone();
        if rule.rule_id.is_some() {
            self.at.rule_id.clone_from(&rule.rule_id);
        }
        if rule.reads_prose {
            let line = rule.common.line;
            let row = std::mem::replace(&mut self.row, rule.reads_row);
            if let Some(shape) = &rule.shape {
                self.shape(shape, line);
            }
            if let Some(select) = &rule.select {
                self.pattern(select.as_str(), line);
            }
            for ban in &rule.prohibited_patterns {
                self.pattern(ban.pattern.as_str(), line);
            }
            self.row = row;
        }
        self.blocks(rule.blocks.as_deref());
        self.at = held;
    }

    /// A shape: a pattern, or the types of the holes of a template.
    fn shape(&mut self, shape: &Shape, line: usize) {
        match shape {
            Shape::Pattern(p) => self.pattern(p.as_str(), line),
            Shape::Template(t) => {
                for s in &t.segments {
                    // A type that several templates use is warned of once,
                    // and once more only where a row reads it otherwise.
                    if let crate::grammar::Segment::Hole { def, .. } = s
                        && self.seen.insert((
                            self.row
                                && respelt_by(&def.regex(), read_row)
                                    != respelt_by(&def.regex(), read),
                            def.regex(),
                        ))
                    {
                        self.pattern(&def.regex(), line);
                    }
                }
            }
            Shape::Text(_) => {}
        }
    }

    fn format(&mut self, name: &str, format: &Format, line: usize) {
        let held = self.at.clone();
        self.at.named = Some(format!("format `{name}`"));
        if let Some(shape) = &format.shape {
            self.shape(shape, line);
        }
        for (_, v) in &format.variants {
            self.shape(&v.shape, line);
        }
        for ban in &format.prohibited {
            self.pattern(ban.pattern.as_str(), line);
        }
        self.at = held;
    }

    /// The warning on each run of literal text of `pattern` whose reading
    /// differs from the run.
    fn pattern(&mut self, pattern: &str, line: usize) {
        let read = if self.row { read_row } else { read };
        for (run, reading) in respelt_by(pattern, read) {
            let locator = self.at.locate(&self.locator);
            let finding = Finding::warning(
                self.path.clone(),
                Some(line),
                "TLMD_264",
                format!(
                    "the pattern {pattern} names {run}, which reads {reading}; a pattern reads the fixed spelling, so it never matches that text outside a code span", pattern = text::code(pattern), run = text::code(&run), reading = text::code(&reading)),
            )
            .at(&locator);
            self.out.push(finding);
        }
    }
}

// sokf:begin respelt
/// Each run of literal text of the regular expression `pattern`, in order,
/// whose reading as a paragraph alone differs from the run, as the pair of
/// the run and its reading, such as `("*Note*", "_Note_")` for
/// `\*Note\*`: the runs that the warning `TLMD_264` names for a pattern of
/// a rule of prose that reads no table row. A pattern matched against
/// the reading of a unit, such as `Unit::reading`, never matches such a
/// run where the reading respells it, though a code span, a link's
/// destination and title, and inline HTML read as written and can hold
/// it. A
/// run that a class builds, such as `[*_]{2}`, is no run of literal text,
/// and a pattern that doesn't parse has none.
// sokf-implements: MD-16_AC-1
// sokf-implements: MD-16_AC-2
pub fn respelt(pattern: &str) -> Vec<(String, String)> {
    // sokf:end respelt
    respelt_by(pattern, read)
}

/// Each run of literal characters of the regular expression `pattern`,
/// from its syntax tree, whose reading through `read`, as a paragraph
/// alone or as a row, differs from the run, with that reading. A pattern
/// that does not parse has none.
fn respelt_by(pattern: &str, read: fn(&str) -> String) -> Vec<(String, String)> {
    let Ok(tree) = ast::parse::Parser::new().parse(pattern) else {
        return Vec::new();
    };
    let mut runs = Vec::new();
    literal_runs(&tree, &mut runs);
    runs.into_iter()
        .filter(|run| run.bytes().any(|b| matches!(b, b'*' | b'_' | b'`' | b'\\')))
        .filter_map(|run| {
            let reading = read(&run);
            (reading != run).then_some((run, reading))
        })
        .collect()
}

/// The runs of literal characters of `tree`, in order: each run of a
/// concatenation that only literals make, and the runs of every group and
/// every branch inside it.
fn literal_runs(tree: &Ast, out: &mut Vec<String>) {
    match tree {
        Ast::Literal(l) => out.push(l.c.to_string()),
        Ast::Concat(concat) => {
            let mut run = String::new();
            for part in &concat.asts {
                if let Ast::Literal(l) = part {
                    run.push(l.c);
                    continue;
                }
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
                literal_runs(part, out);
            }
            if !run.is_empty() {
                out.push(run);
            }
        }
        Ast::Alternation(alt) => {
            for branch in &alt.asts {
                literal_runs(branch, out);
            }
        }
        Ast::Group(g) => literal_runs(&g.ast, out),
        Ast::Repetition(r) => literal_runs(&r.ast, out),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_that_reads_otherwise_is_respelt() {
        assert_eq!(
            respelt_by(r"\*Note\*", read),
            vec![("*Note*".into(), "_Note_".into())]
        );
        assert_eq!(
            respelt_by(r"__Note__", read),
            vec![("__Note__".into(), "**Note**".into())]
        );
        assert_eq!(respelt_by(r"\\_", read), vec![("\\_".into(), "_".into())]);
        assert_eq!(
            respelt_by(r"``x``", read),
            vec![("``x``".into(), "`x`".into())]
        );
        // A run the reading keeps, and one that a class builds, draw none.
        for kept in [
            r"\*\*[^*]+\*\*([^:]|$)",
            r"^_[a-z]+_ and \*\*x\*\*",
            r"[*_]{2}Note",
            r"(",
        ] {
            let found = respelt_by(kept, read);
            assert!(found.is_empty(), "{kept}: {found:?}");
        }
    }
}
