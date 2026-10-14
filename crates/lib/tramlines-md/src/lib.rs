//! tramlines-md: the markdown format of tramlines. It checks a markdown
//! document against a schema, repairs it, and derives the scaffold a
//! schema implies.
//!
//! A schema is YAML in the vocabulary of the grammar: the core grammar of
//! tramlines-core, with the markdown format grammar laid on it, which adds
//! the sections, blocks and units a document of one kind must carry. The
//! engine parses the document, holds it to the schema and reports each
//! failure as a [`Finding`]. The items of tramlines-core are re-exported,
//! so a host needs this crate alone.
//!
//! Compile a schema and check a document against it:
//!
//! ```
//! use tramlines_md::{
//!     Locator, RepoPath,
//!     check::{CheckOptions, NullHost, check_against},
//!     document::Document,
//!     grammar::{FormatRegistry, RegexCache, compile, load_grammar},
//!     markdown::Dialect,
//!     yaml,
//! };
//!
//! let schema = "line-limit: 0\nsections:\n  - heading: Groceries\n    level: 1\n    blocks:\n      - kind: bullet-list\n        each: { template: '${item:word} x${n:int}', example: Milk x2 }\n";
//! let schema = compile(
//!     &yaml::parse(schema, 1)?,
//!     &load_grammar()?,
//!     &FormatRegistry::default(),
//!     &RegexCache::new(),
//!     &RepoPath::parse("/list.yaml").expect("a path"),
//!     "list",
//!     &Dialect::default(),
//! )
//! .expect("the schema compiles");
//!
//! let text = "# Groceries\n\n- Milk x2\n- Bread\n";
//! let path = RepoPath::parse("/groceries.md").expect("a path");
//! let document = Document::parse(path, text, &Dialect::default());
//! let states = schema.states(document.frontmatter.as_ref());
//! let out = check_against(
//!     document.view(),
//!     &schema,
//!     &states,
//!     &NullHost,
//!     CheckOptions::default(),
//!     Locator::schema("list"),
//! );
//! assert_eq!(out.findings.len(), 1);
//! assert_eq!(out.findings[0].criterion, "TL_74");
//! assert_eq!(out.findings[0].line, Some(4));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs)]

pub mod check;
pub mod derive;
pub mod document;
pub mod edit;
pub mod grammar;
pub mod markdown;

// sokf-implements: MD-9_AC-1
// sokf-implements: MD-9_AC-3
// sokf:begin reexports
pub use tramlines_core::{
    Error, Finding, Locator, RepoPath, RepoRoot, Result, Severity, Spanned, Yaml, YamlError, text,
    yaml,
};
// sokf:end reexports
