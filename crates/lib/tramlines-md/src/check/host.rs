//! What a check sees of a document, and what the engine asks of its
//! host: everything that lies outside the document.

use serde::Serialize;

use super::{CheckOptions, CheckOutput};
use crate::{
    Locator, RepoPath, Spanned,
    grammar::{LimitKind, States},
    markdown::{BlockKind, Body, Dialect, Segments},
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
// sokf-implements: MD-5_AC-14
// sokf-implements: MD-5_AC-16
// sokf-implements: MD-5_AC-21
// sokf-implements: MD-5_AC-22
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

    /// The words that a guidance line holds before the `when-at-limit` of
    /// the limit that `facts` describe, in the language of the schema, such
    /// as the count and its subject worded for the writer. Where the host
    /// gives none, which it does by default, the line holds the limit in
    /// the syntax of the schema, such as `` `hard: 9`, `when-at-limit`: ``,
    /// before the `when-at-limit`.
    fn limit_lead(&self, _facts: &LimitFacts<'_>) -> Option<String> {
        None
    }

    /// The words of the label that `facts` describe, in the language of
    /// the schema, which a document writes between the label's square
    /// brackets, such as `Mensch` for the key `human` in German. Where the
    /// host gives none, which it does by default, or words that are blank,
    /// hold a square bracket, or begin with `^`, the label's key is its
    /// words.
    fn label_words(&self, _facts: &LabelFacts<'_>) -> Option<String> {
        None
    }

    /// The key of the label that a scaffold writes on each unit that
    /// carries one, such as `agent`, of `labels`, the keys that the schema
    /// declares. Where the host names none, which it does by default, or a
    /// key that `labels` lacks, a unit takes the label of its example, else
    /// the first of `labels`.
    fn scaffold_label(&self, _labels: &[String]) -> Option<String> {
        None
    }

    /// Whether the engine records in `CheckOutput::accepted` each unit that
    /// a unit rule accepts, for the host's rules to read. By default it
    /// records none, which costs a check nothing.
    fn records_accepted(&self) -> bool {
        false
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
/// The facts of one limit that the host words before its `when-at-limit`:
/// its kind, its count, what it is applied to, and the language of the
/// schema.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct LimitFacts<'a> {
    /// `soft` or `hard`.
    pub kind: LimitKind,
    /// The number that `soft` or `hard` states.
    pub count: usize,
    /// What the limit is applied to.
    pub subject: LimitSubject<'a>,
    /// The `language` of the schema, a BCP 47 tag as the schema writes it,
    /// or `None` where the schema declares none. A tag's case has no
    /// meaning, so compare tags ignoring case.
    pub language: Option<&'a str>,
}

impl<'a> LimitFacts<'a> {
    /// The facts of a limit, for a host's own tests of its words.
    pub fn new(
        kind: LimitKind,
        count: usize,
        subject: LimitSubject<'a>,
        language: Option<&'a str>,
    ) -> Self {
        LimitFacts {
            kind,
            count,
            subject,
            language,
        }
    }
}

/// The facts of one label that the host words: its key, and the language
/// of the schema.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct LabelFacts<'a> {
    /// The key of the label, as the schema's `labels` writes it.
    pub key: &'a str,
    /// The `language` of the schema, a BCP 47 tag as the schema writes it,
    /// or `None` where the schema declares none. A tag's case has no
    /// meaning, so compare tags ignoring case.
    pub language: Option<&'a str>,
}

impl<'a> LabelFacts<'a> {
    /// The facts of a label, for a host's own tests of its words.
    pub fn new(key: &'a str, language: Option<&'a str>) -> Self {
        LabelFacts { key, language }
    }
}

/// What a limit is applied to, as the schema states it, for the host to
/// word with the limit's count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitSubject<'a> {
    /// A block's own limit, on the units of a block of this kind: its
    /// paragraphs, items, rows, fences, or repetitions.
    Block(BlockKind),
    /// The limit of a unit rule or a line rule.
    Rule {
        /// The kind of the block that the rule governs.
        block: BlockKind,
        /// Whether the rule counts units of the block or content lines.
        counts: Counts,
        /// The format that the rule binds, or else its `rule-id`, or
        /// `None` where it has neither. Only a `Schema` that a host builds
        /// or changes itself holds such a rule, because `compile` refuses
        /// a rule with a `max` and no name (`TLMD_235`).
        name: Option<RuleName<'a>>,
    },
    /// A frontmatter key's limit, on the elements of the list of this key.
    Key(&'a str),
}

/// What the limit of a rule counts.
// sokf-implements: MD-13_AC-3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Counts {
    /// The units of a block, under a unit rule: its paragraphs, items,
    /// rows, fences, or repetitions.
    Units,
    /// The content lines of a code block or an HTML block, under a line
    /// rule.
    Lines,
}

/// The name by which the schema names a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleName<'a> {
    /// The name of the format that the rule binds.
    Format(&'a str),
    /// The rule's `rule-id`, where it binds no format.
    RuleId(&'a str),
}

/// The host's part of a scaffold's frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScaffoldFront {
    /// The keys written first, in order, each with the value the host
    /// seeds it with. A value the schema fixes, by `const`, by the `id`
    /// template or by `example`, wins over the seed in a scaffold. In a
    /// skeleton, which writes no example, only a `const` and an `id` that
    /// its template renders from the states alone win over it, and a value
    /// that the host gives for the document wins over all but a `const`.
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
