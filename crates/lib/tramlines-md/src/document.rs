//! A document as the engine reads it on its own: its frontmatter and its
//! body, parsed in a dialect, with nothing of a host.

use std::sync::LazyLock;

use crate::{
    Finding, Locator, RepoPath, Result, Spanned, Yaml,
    check::{self, CheckOptions, CheckOutput, DocView, Host},
    edit::{self, AnchorEdit},
    grammar::Schema,
    markdown::{self, Body, Dialect, Segments, split_frontmatter},
    yaml,
};

// sokf:begin document
/// One document, parsed.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// The file.
    pub path: RepoPath,
    /// The text, LF.
    pub text: String,
    /// The frontmatter, when present and read.
    pub frontmatter: Option<Spanned>,
    /// Why the frontmatter could not be read, when it could not.
    pub problem: Option<Finding>,
    /// The body.
    pub body: Body,
    /// The markers.
    pub segments: Segments,
    /// The line the body starts on.
    pub body_line: usize,
}
// sokf:end document

impl Document {
    /// Parse `text`, the document at `path`, its markers as `dialect`
    /// spells them. A CRLF text is read as LF.
    // sokf:begin document
    pub fn parse(path: RepoPath, text: &str, dialect: &Dialect) -> Self {
        // sokf:end document
        // sokf-implements: MD-3_AC-1
        // sokf-implements: CLI-8_AC-1
        let text = text.replace("\r\n", "\n");
        let split = split_frontmatter(&text);
        let (frontmatter, problem) = match split.frontmatter {
            // sokf-implements: MD-3_AC-4
            None => (None, None),
            Some((written, line)) => match yaml::parse(written, line) {
                Ok(y) => (Some(y), None),
                // YAML that parses and uses what the frontmatter refuses is
                // a fault of its own.
                // sokf-implements: MD-3_AC-3
                // sokf-implements: TL_53
                // sokf-implements: TL_123
                Err(e) => {
                    let criterion = if e.is_refused() { "TL_123" } else { "TL_53" };
                    (
                        None,
                        Some(Finding::error(
                            path.clone(),
                            Some(line - 1),
                            criterion,
                            format!("the frontmatter does not read: {e}"),
                        )),
                    )
                }
            },
        };
        // sokf-implements: MD-3_AC-2
        let segments = markdown::segments(split.body, split.body_line, dialect);
        let body = markdown::parse(split.body, split.body_line, &segments, dialect);
        let body_line = split.body_line;
        Document {
            path,
            frontmatter,
            problem,
            body,
            segments,
            body_line,
            text,
        }
    }

    /// The document checked against `schema` at the states its frontmatter
    /// holds, each finding ending with `locator`. A frontmatter that does
    /// not read is the first finding.
    // sokf:begin document
    pub fn check(&self, schema: &Schema, host: &dyn Host, locator: Locator) -> CheckOutput {
        // sokf:end document
        // sokf-implements: MD-3_AC-9
        let states = schema.states(self.frontmatter.as_ref());
        let mut out = check::check_against(
            self.view(),
            schema,
            &states,
            host,
            CheckOptions::default(),
            locator,
        );
        if let Some(problem) = &self.problem {
            out.findings.insert(0, problem.clone());
        }
        out
    }

    /// The text with the repairs of `out` and the anchors its keys need
    /// applied, or `None` where they change nothing.
    // sokf:begin document
    pub fn repair(&self, out: &CheckOutput) -> Result<Option<String>> {
        // sokf:end document
        // sokf-implements: MD-3_AC-10
        let anchors = AnchorEdit {
            body: &self.body,
            placements: &out.anchors,
            text: &self.text,
            renames: &[],
        };
        let edits = edit::line_edits(&self.text, &self.segments, &out.repairs, Some(&anchors));
        if edits.is_empty() {
            return Ok(None);
        }
        let repaired = edit::apply(&self.text, &edits)?;
        Ok((repaired != self.text).then_some(repaired))
    }

    /// The number of lines.
    pub fn line_count(&self) -> usize {
        self.text.lines().count()
    }

    /// What a check sees of the document. A markdown document always has
    /// frontmatter to check: one that is absent, or that is not a map, is
    /// checked as an empty map, so each key the schema demands is reported.
    /// One that does not read is not checked; its problem is the finding.
    // sokf:begin document
    pub fn view(&self) -> DocView<'_> {
        // sokf:end document
        // sokf-implements: TL_54
        // sokf-implements: TL_94
        // sokf-implements: MD-3_AC-5
        // sokf-implements: MD-3_AC-6
        // sokf-implements: MD-3_AC-7
        static EMPTY: LazyLock<Spanned> = LazyLock::new(|| Spanned::new(Yaml::Map(vec![]), 1));
        let frontmatter = match (&self.frontmatter, &self.problem) {
            (_, Some(_)) => None,
            (Some(f), None) if f.as_map().is_some() => Some(f),
            _ => Some(&*EMPTY),
        };
        DocView {
            path: &self.path,
            frontmatter,
            body: &self.body,
            segments: &self.segments,
            line_count: self.line_count(),
            text: &self.text,
            body_line: self.body_line,
        }
    }
}
