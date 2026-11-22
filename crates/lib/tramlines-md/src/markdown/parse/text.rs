//! A leaf's text as written, and where each piece of it lies in the file,
//! so a position in a unit's text maps back to a line and a column.

use comrak::nodes::{LineColumn, Sourcepos};

use super::Parser;
use crate::markdown::Piece;

impl Parser<'_> {
    /// A leaf node's text as written: the first line from its start column,
    /// continuation lines stripped of indentation and quote markers, joined
    /// by one space.
    pub(super) fn source_text(&self, sp: Sourcepos) -> String {
        self.mapped_text(sp).0
    }

    /// `source_text`, and the start of each of its pieces: the offset in
    /// the text, the file line and the byte column in that line.
    pub(super) fn mapped_text(&self, sp: Sourcepos) -> (String, Vec<Piece>) {
        let start = self.line(sp);
        let end = self.end_line(sp);
        let mut text = String::new();
        let mut pieces = Vec::new();
        for n in start..=end {
            let line = self.src(n);
            let to = if n == end {
                line.ceil_char_boundary(sp.end.column.min(line.len()))
            } else {
                line.len()
            };
            let from = if n == start {
                line.floor_char_boundary(sp.start.column.saturating_sub(1).min(line.len()))
            } else {
                let head = &line[..to];
                head.len() - strip_prefix(head).len()
            };
            let slice = &line[from..to.max(from)];
            let piece = slice.trim();
            if piece.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            let col = from + (slice.len() - slice.trim_start().len());
            pieces.push(Piece {
                offset: text.len(),
                line: n,
                col,
            });
            text.push_str(piece);
        }
        (text, pieces)
    }

    /// A heading's text, and the start of each of its pieces: an ATX
    /// heading's from its line without its hashes, and a setext heading's
    /// from its lines without its underline, read from where comrak starts
    /// it, so past the marker of an item or a quote that holds it
    /// (`TLMD_173`). An ATX heading with no text keeps a piece where its
    /// text would start, so that it holds the anchors on its line.
    // sokf-implements: TLMD_173
    pub(super) fn heading_text(&self, sp: Sourcepos, setext: bool) -> (String, Vec<Piece>) {
        if setext {
            let mut lines = sp;
            lines.end = LineColumn {
                line: sp.end.line.saturating_sub(1).max(sp.start.line),
                column: usize::MAX,
            };
            return self.mapped_text(lines);
        }
        let (written, pieces) = self.mapped_text(sp);
        let text = super::atx_text(&written);
        let lead = written.len() - written.trim_start_matches('#').trim_start().len();
        let pieces = pieces
            .into_iter()
            .take(1)
            .map(|p| Piece {
                col: p.col + lead,
                ..p
            })
            .collect();
        (text, pieces)
    }
}

/// Strip list indentation and block-quote markers from a continuation line.
pub(in crate::markdown) fn strip_prefix(line: &str) -> &str {
    let mut t = line.trim_start();
    while let Some(rest) = t.strip_prefix('>') {
        t = rest.trim_start();
    }
    t
}
