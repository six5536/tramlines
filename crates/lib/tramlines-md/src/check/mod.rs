//! `check_against`: a document against a schema, every grammar rule key
//! applied. What lies outside the document, the host answers through
//! [`Host`].

mod anchors;
mod blocks;
mod checker;
mod claim;
mod divide;
pub mod example;
mod frontmatter;
mod host;
mod keys;
mod limits;
mod markers;
mod output;
mod rules;
mod schema;
mod sections;

pub use anchors::{heading_col, placements};
pub(crate) use checker::{At, Checker, Occurrence, short};
pub use checker::{CheckOptions, check_against};
pub use host::{DocView, Host, NullHost, ScaffoldFront, Subject};
pub(crate) use output::rules_named;
pub use output::{Capture, CheckOutput, Gate, Paired, Placement, Repair, Spot};
pub use schema::check_schema;
