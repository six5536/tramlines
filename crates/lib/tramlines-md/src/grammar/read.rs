//! The text of a schema in the reading of markdown (`TLMD_259`), and the
//! formats as read, which a rule of prose binds.

use std::ops::Range;

use super::FormatRegistry;
use crate::markdown::{self, BlockKind};

/// `text`, a text of the schema, in the reading of markdown, read as a
/// paragraph alone (`TLMD_259`).
// sokf-implements: TLMD_259
pub(crate) fn read(text: &str) -> String {
    markdown::reading(text).text
}

/// Whether the units of a block of `kind` read prose, in their reading
/// (`TLMD_258`): a paragraph, an item, a row, or a paragraph of a quote.
pub(crate) fn reads_prose(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::Prose
            | BlockKind::BulletList
            | BlockKind::NumberedList
            | BlockKind::Table
            | BlockKind::Blockquote
    )
}

/// `formats` as read through `read`: each format's shapes in the reading
/// of markdown as a paragraph alone, or cell by cell as a row reads.
pub(crate) fn read_formats(formats: &FormatRegistry, read: fn(&str) -> String) -> FormatRegistry {
    FormatRegistry {
        entries: formats
            .entries
            .iter()
            .map(|(name, (f, source))| (name.clone(), (f.read_with(&read), source.clone())))
            .collect(),
        types: formats.types.clone(),
    }
}

/// `text`, a text of the schema for a table row, in the reading of a row:
/// each cell, between ` | `, read alone, as the parser reads a row.
pub(crate) fn read_row(text: &str) -> String {
    row_reading(text).text
}

/// The code spans of `text`, a text of the schema for a table row, such as
/// a squashed example, each cell parsed alone, as the parser reads a row,
/// so that no span crosses ` | `.
pub(crate) fn row_code_spans(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut base = 0;
    for (i, cell) in text.split(" | ").enumerate() {
        if i > 0 {
            base += 3;
        }
        out.extend(
            markdown::code_spans_in(cell)
                .into_iter()
                .map(|r| r.start + base..r.end + base),
        );
        base += cell.len();
    }
    out
}

/// The reading of `text`, a text of the schema for a table row, cell by
/// cell, each byte mapped to the byte of `text` that it stands for.
pub(crate) fn row_reading(text: &str) -> markdown::Reading {
    let mut out = markdown::Reading::default();
    let mut base = 0;
    for (i, cell) in text.split(" | ").enumerate() {
        if i > 0 {
            out.map.extend(base..base + 3);
            out.text.push_str(" | ");
            base += 3;
        }
        let r = markdown::reading(cell);
        out.map
            .extend(r.map[..r.text.len()].iter().map(|o| base + o));
        out.text.push_str(&r.text);
        base += cell.len();
    }
    out.map.push(text.len());
    out
}
