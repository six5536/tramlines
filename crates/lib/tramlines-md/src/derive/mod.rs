//! The scaffold a schema implies, which is the only template of a
//! document it has.
//!
//! The scaffold is checked, never trusted: it goes through the
//! path that checks an authored one, so a generator that writes what the
//! schema refuses fails the schema in the store that owns the schema.
//! The failure mode is a false alarm and never a silent hole.
//!
//! No text is invented. A part that no `text`, no `example` and no bound
//! format supplies is a refusal naming the rule that demands it: text that
//! satisfies the checker and teaches nothing is worse than an error, and
//! the agent that reads a scaffold imitates what it is given.

mod bans;
mod blocks;
mod body;
mod deriver;
mod frontmatter;
mod scaffold;
mod text;
mod units;

use deriver::Deriver;

pub use scaffold::{Derived, Refusal, derive, derive_one, derived_title, no_template};
