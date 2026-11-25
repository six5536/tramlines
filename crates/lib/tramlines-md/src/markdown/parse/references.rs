//! The link reference definitions of a body, which comrak consumes: an
//! entry of `ref_defs` for each one, and the `references` blocks of those
//! at the top level.

use std::collections::HashSet;

use comrak::{Arena, nodes::NodeValue};

use super::{Parser, options};
use crate::markdown::{
    Block, BlockKind, Unit, code,
    link_definitions::{self, LinkDef},
};

impl Parser<'_> {
    /// Reference definitions comrak consumed, as comrak reads them: those
    /// on the lines outside every leaf block, of which each at the top level
    /// of the body is a unit of a `references` block, and those that open a
    /// paragraph inside an item or a quote. Each one is an entry of
    /// `ref_defs`, at the line of its label.
    // sokf-implements: TLMD_268
    // sokf-implements: TLMD_278
    // sokf-implements: MD-3_AC-17
    pub(super) fn reference_blocks(&mut self) {
        let covered = |file_line: usize| {
            self.covered
                .iter()
                .any(|&(s, e)| s <= file_line && file_line <= e)
        };
        // The runs of lines that no block covers, each read alone, as file
        // lines.
        let mut free: Vec<LinkDef> = Vec::new();
        let mut i = 0;
        while i < self.lines.len() {
            if covered(self.offset + 1 + i) {
                i += 1;
                continue;
            }
            let end = (i..self.lines.len())
                .find(|&k| covered(self.offset + 1 + k))
                .unwrap_or(self.lines.len());
            let base = self.offset + 1 + i;
            free.extend(
                link_definitions::read(&self.lines[i..end])
                    .defs
                    .into_iter()
                    .map(|d| LinkDef {
                        first: base + d.first,
                        last: base + d.last,
                        ..d
                    }),
            );
            i = end;
        }
        let nested = self.nested_lines(&free);
        let mut current: Option<Block> = None;
        let mut free = free.into_iter().peekable();
        // The last line of the definition that the walk is in.
        let mut within = 0;
        for (i, line) in self.lines.iter().enumerate() {
            let file_line = self.offset + 1 + i;
            if let Some(d) = free.next_if(|d| d.first == file_line) {
                within = d.last;
                self.body.ref_defs.push((d.label, d.dest, file_line));
                // A definition inside an item, a quote, or a footnote is
                // part of it, and forms no block of the top level.
                if line.trim_start().starts_with('[') && !nested.contains(&file_line) {
                    let text = self.lines[i..=i + d.last - d.first]
                        .iter()
                        .map(|l| l.trim())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let unit = Unit::leaf(text, file_line, d.last);
                    match current.as_mut() {
                        Some(b) => {
                            b.units.push(unit);
                            b.end_line = d.last;
                        }
                        None => {
                            current = Some(Block::new(
                                BlockKind::References,
                                file_line,
                                d.last,
                                vec![unit],
                            ));
                        }
                    }
                    continue;
                }
            } else if file_line <= within {
                continue;
            }
            if (covered(file_line) || !line.trim().is_empty())
                && let Some(b) = current.take()
            {
                self.body.blocks.push(b);
            }
        }
        if let Some(b) = current.take() {
            self.body.blocks.push(b);
        }
        // The definitions that open a paragraph inside an item or a quote.
        for &(first, last) in &self.opening {
            let (Some(from), Some(to)) = (
                first.checked_sub(self.offset + 1),
                last.checked_sub(self.offset + 1),
            ) else {
                continue;
            };
            let Some(lines) = self.lines.get(from..=to) else {
                continue;
            };
            for d in link_definitions::read(lines).defs {
                self.body.ref_defs.push((d.label, d.dest, first + d.first));
            }
        }
        self.body.ref_defs.sort_by_key(|d| d.2);
    }

    /// The first line of each definition of `free` whose label opens its
    /// line and that stands inside a list item, a quote, or a footnote's
    /// definition, as an indented or a lazy line does. comrak keeps no node
    /// for a definition, but it reads the lines of definitions as a
    /// paragraph before it takes them out, so the parser writes a letter in
    /// place of the `[` of each such label, which keeps every line in the
    /// containers that held it, though the paragraph can read as another
    /// leaf, such as a setext heading, and reads the containers of the leaf
    /// that holds the line. A label at the first column of the body's first
    /// line, of a line after a blank line, or of a line after such a
    /// definition stands at the top level, so it needs no second parse.
    fn nested_lines(&self, free: &[LinkDef]) -> HashSet<usize> {
        let line_at = |n: usize| n.checked_sub(1).and_then(|i| self.lines.get(i)).copied();
        let mut top_until = 0;
        let mut candidates = HashSet::new();
        for d in free {
            let n = d.first - self.offset;
            let Some(line) = line_at(n).filter(|l| l.trim_start().starts_with('[')) else {
                continue;
            };
            let after_top =
                n == 1 || n - 1 <= top_until || line_at(n - 1).is_some_and(|l| l.trim().is_empty());
            if line.starts_with('[') && after_top {
                top_until = d.last - self.offset;
            } else {
                candidates.insert(n);
            }
        }
        if candidates.is_empty() {
            return HashSet::new();
        }
        // Each line as comrak ends it, so that a carriage return alone ends
        // a line here as it does in `lines`.
        let mut probe = self.source.as_bytes().to_vec();
        for (i, line) in code::each_line(self.source).enumerate() {
            if candidates.contains(&(i + 1))
                && let Some(at) = self.source[line.clone()].find('[')
            {
                probe[line.start + at] = b'x';
            }
        }
        let probe = String::from_utf8(probe)
            .expect("an ASCII `[` replaced with an ASCII `x` keeps the text UTF-8");
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &probe, &options());
        let mut inside: Vec<(usize, usize)> = root
            .descendants()
            .filter(|n| {
                matches!(n.data().value, NodeValue::Paragraph | NodeValue::Heading(_))
                    && n.ancestors().skip(1).any(|a| {
                        matches!(
                            a.data().value,
                            NodeValue::Item(_)
                                | NodeValue::TaskItem(_)
                                | NodeValue::BlockQuote
                                | NodeValue::FootnoteDefinition(_)
                        )
                    })
            })
            .map(|n| (n.data().sourcepos.start.line, n.data().sourcepos.end.line))
            .collect();
        // Leaves don't overlap, so the one that can hold a line is the last
        // that starts at it or before it.
        inside.sort_unstable();
        candidates
            .into_iter()
            .filter(|&l| {
                let k = inside.partition_point(|&(s, _)| s <= l);
                k > 0 && inside[k - 1].1 >= l
            })
            .map(|l| l + self.offset)
            .collect()
    }
}
