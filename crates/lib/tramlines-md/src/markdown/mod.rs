//! Markdown parsing: blocks and units as the grammar reads them, the
//! markers a host's dialect spells, and heading slugs.

mod anchor;
mod body;
mod code;
mod dialect;
mod link;
mod link_definitions;
pub(crate) use link_definitions::all_definitions;
mod parse;
mod reading;
mod segments;
mod slug;
mod split;
mod written;

pub(crate) use anchor::gap;
pub use anchor::{Anchor, render as render_anchor};
pub(crate) use body::mask_code;
pub use body::{
    Block, BlockDetail, BlockKind, Body, CodeBlock, Heading, Link, LinkTarget, Piece, Unit,
};
pub use dialect::{Dialect, Marker};
pub use link::host_ref;
pub use parse::parse;
pub(crate) use parse::{atx_text, code_spans_in};
pub(crate) use reading::{Change, apply, changes};
pub use reading::{Reading, plain_text, reading};
pub use segments::{DefinitionBlock, Region, Segments, ref_def, segments};
pub(crate) use slug::id_of;
pub use slug::slug;
pub use split::{Split, split_frontmatter};
pub use tramlines_core::text::{is_slug, is_url};
pub(crate) use written::{atx_written, escapes, heading_text_of, unit_written};
