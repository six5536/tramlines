//! Markdown parsing: blocks and units as the grammar reads them, the
//! markers a host's dialect spells, and heading slugs.

mod anchor;
mod body;
mod code;
mod dialect;
mod link;
mod parse;
mod segments;
mod slug;
mod split;

pub(crate) use anchor::gap;
pub use anchor::{Anchor, render as render_anchor};
pub(crate) use body::mask_code;
pub use body::{
    Block, BlockDetail, BlockKind, Body, CodeBlock, Heading, Link, LinkTarget, Piece, Unit,
};
pub use dialect::{Dialect, Marker};
pub use link::host_ref;
pub(crate) use parse::code_spans_in;
pub use parse::parse;
pub use segments::{DefinitionBlock, Region, Segments, ref_def, segments};
pub use slug::slug;
pub use split::{Split, split_frontmatter};
pub use tramlines_core::text::{is_slug, is_url};
