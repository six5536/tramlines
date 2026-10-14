//! tramlines-core: the part of tramlines that every format of document
//! shares.
//!
//! The core grammar, the vocabulary a schema is written in, and the
//! layers a format grammar and a host extension add to it; the shapes,
//! types and formats a rule matches text with; the frontmatter and the
//! lifecycle of a document; and the [`Finding`] a failure is reported as.
//! A format crate, such as tramlines-md, adds what its documents hold.
//!
//! A host program uses a format crate, which re-exports what it needs from
//! here. This crate alone reads a formats file and matches lines against
//! its formats:
//!
//! ```
//! use tramlines_core::grammar::{FormatMatch, FormatRegistry, RegexCache, load_core_grammar};
//!
//! let grammar = load_core_grammar()?;
//! let formats = "registry:\n  item:\n    template: '${name:word} x${n:int}'\n    example: Milk x2\n";
//! let registry = FormatRegistry::from_text(formats, &[], &grammar, &RegexCache::new())?;
//! let item = registry.get("item").expect("the file defines `item`");
//! assert!(matches!(item.check("Bread x1"), FormatMatch::Ok(_)));
//! assert!(matches!(item.check("Bread"), FormatMatch::Fail(_)));
//! # Ok::<(), tramlines_core::Error>(())
//! ```

#![warn(missing_docs)]

mod error;
mod finding;
pub mod frontmatter;
pub mod grammar;
mod path;
pub mod text;
pub mod yaml;

pub use error::{Error, Result};
pub use finding::{Finding, Locator, Problem, Severity};
pub use path::{RepoPath, RepoRoot};
pub use yaml::{Spanned, Yaml, YamlError};
