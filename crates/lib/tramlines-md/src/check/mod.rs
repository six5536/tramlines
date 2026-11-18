//! `check_against`: a document against a schema, every grammar rule key
//! applied. What lies outside the document, the host answers through
//! [`Host`].

mod anchors;
mod blocks;
mod bound;
mod checker;
mod claim;
mod divide;
pub mod example;
mod frontmatter;
mod group;
mod host;
mod keys;
mod label;
mod limits;
mod markers;
mod output;
mod rules;
mod schema;
mod sections;
mod trial;

pub use anchors::{heading_col, placements};
pub(crate) use checker::{At, Broken, Checker, Occurrence, short};
pub use checker::{CheckOptions, check_against};
pub(crate) use claim::is_run;
pub use host::{
    Counts, DocView, Host, LabelFacts, LimitFacts, LimitSubject, NullHost, RuleName, ScaffoldFront,
    Subject,
};
pub(crate) use label::words_of;
pub(crate) use output::rules_named;
pub use output::{
    Accepted, Capture, CheckOutput, Gate, LabelRepair, Paired, Placement, Repair, Spot,
};
pub use schema::check_schema;
