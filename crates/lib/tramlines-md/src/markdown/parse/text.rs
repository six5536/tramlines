//! A leaf's text as written, and where each piece of it lies in the file,
//! so a position in a unit's text maps back to a line and a column.

use comrak::nodes::Sourcepos;

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
}

/// Strip list indentation and block-quote markers from a continuation line.
pub(super) fn strip_prefix(line: &str) -> &str {
    let mut t = line.trim_start();
    while let Some(rest) = t.strip_prefix('>') {
        t = rest.trim_start();
    }
    t
}
