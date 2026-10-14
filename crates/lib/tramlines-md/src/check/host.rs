//! What a check sees of a document, and what the engine asks of its
//! host: everything that lies outside the document.

use super::{CheckOptions, CheckOutput};
use crate::{
    Locator, RepoPath, Spanned,
    grammar::States,
    markdown::{Body, Dialect, Segments},
};

// sokf:begin doc-view
/// What a check sees of a document, a scaffold or a file with no
/// frontmatter.
#[derive(Debug, Clone, Copy)]
pub struct DocView<'a> {
    /// The file, or the schema for a scaffold or an example.
    pub path: &'a RepoPath,
    /// The frontmatter, when the document has one that is a map;
    /// `None` leaves the frontmatter unchecked.
    pub frontmatter: Option<&'a Spanned>,
    /// The body.
    pub body: &'a Body,
    /// The markers.
    pub segments: &'a Segments,
    /// The number of lines.
    pub line_count: usize,
    /// The whole text, LF.
    pub text: &'a str,
    /// The line the body starts on.
    pub body_line: usize,
}
// sokf:end doc-view

// sokf:begin host-answers
/// What a `source-pattern` is matched against, as the host reads a
/// reference or a region's target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// This text.
    Text(String),
    /// A document, which a `source-pattern` never admits.
    Document,
    /// Nothing the host can tell; the rule does not apply.
    Unknown,
}
// sokf:end host-answers

// sokf-implements: MD-5_AC-9
// sokf-implements: MD-5_AC-13
// sokf:begin host
/// What the engine asks of its host in the middle of a check: everything
/// that lies outside the document.
pub trait Host: tramlines_core::frontmatter::Host {
    /// How the host spells its markers.
    fn dialect(&self) -> Dialect;

    /// What an image in the host's reference scheme shows: `reference` is
    /// the label after `<scheme>:`.
    fn image(&self, reference: &str) -> Subject;

    /// What a region's target names, as written in its open marker.
    fn region(&self, target: &str) -> Subject;

    /// The frontmatter a scaffold at `states` starts and ends with; `None`
    /// when the documents have no frontmatter.
    fn scaffold_front(&self, _states: &States) -> Option<ScaffoldFront> {
        Some(ScaffoldFront::default())
    }

    /// The host's own rules on `doc` at `states`, run after the engine's.
    /// The host pushes each finding into `out`, ending a finding of a rule
    /// of the schema with `locator`, and each gate, which reaches the check
    /// of a scaffold's `only` markers. By default the host adds no rule.
    fn check(
        &self,
        _doc: &DocView<'_>,
        _states: &States,
        _options: CheckOptions,
        _locator: &Locator,
        _out: &mut CheckOutput,
    ) {
    }
}
// sokf:end host

// sokf:begin host-answers
/// The host's part of a scaffold's frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScaffoldFront {
    /// The keys written first, in order, each with the value the host
    /// seeds it with. A value the schema fixes, by `const`, by the `id`
    /// template or by `example`, wins over the seed.
    pub leading: Vec<(String, Option<String>)>,
    /// The lines written after every key, before the closing fence.
    pub trailing: Vec<String>,
}
// sokf:end host-answers

// sokf-implements: MD-5_AC-10
// sokf-implements: MD-5_AC-11
// sokf:begin host-answers
/// A host that knows nothing beyond the document: every question is
/// unknown, and the engine's markers are in the default dialect.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullHost;

impl Host for NullHost {
    fn dialect(&self) -> Dialect {
        Dialect::default()
    }

    fn image(&self, _: &str) -> Subject {
        Subject::Unknown
    }

    fn region(&self, _: &str) -> Subject {
        Subject::Unknown
    }
}
// sokf:end host-answers

// sokf-implements: MD-5_AC-11
impl tramlines_core::frontmatter::Host for NullHost {
    fn scalar_format(&self, _: &str, _: &str) -> Option<Result<(), String>> {
        None
    }

    fn reserved_keys(&self) -> &[&str] {
        &[]
    }
}
