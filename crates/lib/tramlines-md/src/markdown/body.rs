//! The parsed body: blocks with units as grammar `block` and `unit` read
//! them, plus positioned headings, links, footnotes and definitions.

use serde::{Deserialize, Serialize};

use super::{Anchor, Dialect, slug};
use crate::{RepoPath, text};

/// One variant per grammar `block.kind` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    /// Paragraphs.
    Prose,
    /// A bullet list.
    BulletList,
    /// An ordered list.
    NumberedList,
    /// A GFM table.
    Table,
    /// A code block.
    Code,
    /// A block quote.
    Blockquote,
    /// A paragraph of images only.
    Image,
    /// The host's region: a block its open and close markers bound.
    Region,
    /// An HTML block.
    Html,
    /// A horizontal rule.
    ThematicBreak,
    /// Link reference definitions outside the generated block.
    References,
    /// A repeating sequence of blocks; matched by the checker, never parsed.
    Group,
    /// A heading inside an item or a quote, which is no heading of the
    /// document; parsed, and named by no schema.
    Heading,
}

impl BlockKind {
    /// The grammar's name for the kind, the host's region as `region`.
    pub fn name(self) -> &'static str {
        match self {
            BlockKind::Prose => "prose",
            BlockKind::BulletList => "bullet-list",
            BlockKind::NumberedList => "numbered-list",
            BlockKind::Table => "table",
            BlockKind::Code => "code",
            BlockKind::Blockquote => "blockquote",
            BlockKind::Image => "image",
            BlockKind::Region => "region",
            BlockKind::Html => "html",
            BlockKind::ThematicBreak => "thematic-break",
            BlockKind::References => "references",
            BlockKind::Group => "group",
            BlockKind::Heading => "heading",
        }
    }

    /// The grammar's name for the kind, the host's region by the name its
    /// dialect gives it.
    pub fn name_in(self, dialect: &Dialect) -> &'static str {
        match (self, dialect.region) {
            (BlockKind::Region, Some(region)) => region,
            _ => self.name(),
        }
    }

    /// The kind's name in `dialect` after its article, as a sentence names
    /// a block: "a bullet-list".
    pub(crate) fn spoken_in(self, dialect: &Dialect) -> String {
        let name = self.name_in(dialect);
        format!("{} {name}", text::article(name))
    }

    /// The kind the grammar names `s`, as `dialect` names the region.
    pub fn parse_in(s: &str, dialect: &Dialect) -> Option<Self> {
        if dialect.region == Some(s) {
            return Some(BlockKind::Region);
        }
        Self::parse(s).filter(|k| *k != BlockKind::Region)
    }

    /// The kind the grammar names `s`.
    pub fn parse(s: &str) -> Option<Self> {
        [
            BlockKind::Prose,
            BlockKind::BulletList,
            BlockKind::NumberedList,
            BlockKind::Table,
            BlockKind::Code,
            BlockKind::Blockquote,
            BlockKind::Image,
            BlockKind::Region,
            BlockKind::Html,
            BlockKind::ThematicBreak,
            BlockKind::References,
            BlockKind::Group,
        ]
        .into_iter()
        .find(|k| k.name() == s)
    }
}

/// The per-kind facts grammar `block` keys test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockDetail {
    /// Nothing beyond the kind.
    None,
    /// A code block's tag and fence length; an indented block has an empty
    /// tag and fence 0.
    Code {
        /// The first word of the info string.
        lang: String,
        /// The number of backticks or tildes.
        fence: usize,
    },
    /// A table's header row.
    Table {
        /// The header cells.
        columns: Vec<String>,
    },
    /// A region's open-marker target, as written.
    Region {
        /// The target text.
        target: String,
    },
    /// The headings inside a blockquote, which the quote does not read as
    /// its paragraphs: the line of each, and the heading as written.
    Quote {
        /// Each heading.
        headings: Vec<(usize, String)>,
    },
}

/// One part of a body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// The kind.
    pub kind: BlockKind,
    /// The line the block starts on.
    pub line: usize,
    /// The line the block ends on.
    pub end_line: usize,
    /// Paragraphs, items, rows, or the one unit of a code or HTML block.
    pub units: Vec<Unit>,
    /// The per-kind facts.
    pub detail: BlockDetail,
    /// Lines carrying inline HTML other than a comment.
    pub html_lines: Vec<usize>,
}

impl Block {
    /// A block of `kind` holding `units`, with no per-kind facts and no
    /// inline HTML.
    pub(crate) fn new(kind: BlockKind, line: usize, end_line: usize, units: Vec<Unit>) -> Self {
        Block {
            kind,
            line,
            end_line,
            units,
            detail: BlockDetail::None,
            html_lines: Vec::new(),
        }
    }
}

/// A paragraph, an item, a row, or an image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// The text as written, lines joined by one space.
    pub text: String,
    /// The line the unit starts on.
    pub line: usize,
    /// The line the unit's own text ends on: a paragraph's last line, an
    /// item's first paragraph's last line.
    #[serde(default)]
    pub end_line: usize,
    /// What a list item carries beyond its own text.
    pub nested: Vec<Block>,
    /// Where each piece of `text` lies, in order: empty for a unit that is
    /// not a paragraph, an item or a table row.
    #[serde(default)]
    pub pieces: Vec<Piece>,
}

/// The start of one piece of a unit's text: a line of a paragraph, or a
/// table cell. The piece runs on unbroken in its line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    /// The byte offset in the unit's text.
    pub offset: usize,
    /// The file line.
    pub line: usize,
    /// The byte column in the line, anchors removed.
    pub col: usize,
}

impl Unit {
    /// A unit of `text` from `line` to `end_line`, with nothing nested and
    /// no pieces mapped.
    pub(crate) fn leaf(text: String, line: usize, end_line: usize) -> Self {
        Unit {
            text,
            line,
            end_line,
            nested: Vec::new(),
            pieces: Vec::new(),
        }
    }

    /// The file line and byte column of `offset` in the text, anchors
    /// removed; `None` for a unit with no pieces.
    pub fn position(&self, offset: usize) -> Option<(usize, usize)> {
        let piece = self.pieces.iter().rev().find(|p| p.offset <= offset)?;
        Some((piece.line, piece.col + (offset - piece.offset)))
    }
}

/// A heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    /// The text as written.
    pub text: String,
    /// The slug of the heading (TLMD_101).
    pub slug: String,
    /// The line of the heading, or of a setext heading's first line.
    pub line: usize,
    /// The last line: the underline of a setext heading.
    pub end_line: usize,
    /// Whether the heading is setext.
    pub setext: bool,
}

/// The forms a body link takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkTarget {
    /// `[text][<scheme>:<rest>]`, a reference in the host's scheme: the
    /// label after `<scheme>:`, which the host resolves.
    Host(String),
    /// A reference-style link whose label is in no host scheme and matches
    /// no definition.
    Reference(String),
    /// A `/`-rooted path.
    Repo {
        /// The path.
        path: RepoPath,
        /// The text after `#`.
        fragment: Option<String>,
    },
    /// A path resolved from the containing file.
    Relative {
        /// The path as written, without its fragment.
        target: String,
        /// The text after `#`.
        fragment: Option<String>,
    },
    /// An absolute URL.
    Url(String),
    /// A `#fragment` alone.
    Anchor(String),
    /// A `/`-rooted path that leaves the repository.
    Invalid(String),
}

/// A body link or image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// Where it points.
    pub target: LinkTarget,
    /// The line.
    pub line: usize,
    /// Whether it is an image.
    pub is_image: bool,
    /// The last line the link covers: a link may wrap.
    #[serde(default)]
    pub end_line: usize,
    /// An inline link's destination as written, from `](` to the closing
    /// `)`, when it lies on one line: what a repair replaces to respell it.
    /// `None` for a reference-style link.
    #[serde(default)]
    pub inline: Option<String>,
}

impl Link {
    /// The first and the last line the link covers.
    pub fn lines(&self) -> (usize, usize) {
        (self.line, self.end_line.max(self.line))
    }
}

/// A parsed body.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    /// The blocks, in order, headings excluded.
    pub blocks: Vec<Block>,
    /// The headings, in order.
    pub headings: Vec<Heading>,
    /// Every link and image, include interiors and the definition block
    /// excluded.
    pub links: Vec<Link>,
    /// Footnote references: label and line.
    pub footnote_refs: Vec<(String, usize)>,
    /// Footnote definitions: label and line.
    pub footnote_defs: Vec<(String, usize)>,
    /// Link reference definitions outside the generated block: label,
    /// destination and line.
    pub ref_defs: Vec<(String, String, usize)>,
    /// Lines carrying inline HTML other than a comment.
    pub inline_html: Vec<usize>,
    /// Line ranges of fenced and indented code, inclusive.
    pub fenced: Vec<(usize, usize)>,
    /// Byte ranges a code span covers: line, start, end. A span crossing
    /// lines records one entry per line.
    #[serde(default)]
    pub code_spans: Vec<(usize, usize, usize)>,
    /// Byte ranges a link's text and destination cover: line, start, end,
    /// as `code_spans`.
    #[serde(default)]
    pub link_spans: Vec<(usize, usize, usize)>,
    /// The generated anchors, removed from the text before it was parsed
    /// (TLMD_107).
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    /// The line the body starts on.
    pub first_line: usize,
}

impl Body {
    /// Each heading inside a list item or a quote, at any depth, in the
    /// order of the document, with the slug of TLMD_101. Such a heading is
    /// part of its block, not a heading of the document (TLMD_173), but a
    /// host that resolves a link to it needs it.
    // sokf-implements: MD-3_AC-8
    pub fn nested_headings(&self) -> Vec<Heading> {
        let mut out: Vec<Heading> = headings_in(&self.blocks)
            .into_iter()
            .map(|(line, written)| {
                let hashes = written.chars().take_while(|&c| c == '#').count();
                let text = written[hashes..].trim().to_string();
                Heading {
                    level: u8::try_from(hashes).unwrap_or(6),
                    slug: slug(&text),
                    text,
                    line,
                    end_line: line,
                    setext: false,
                }
            })
            .collect();
        out.sort_by_key(|h| h.line);
        out
    }

    /// The blocks between `from` (exclusive) and `to` (exclusive), as a
    /// section's body.
    pub fn blocks_between(&self, from: usize, to: usize) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| b.line > from && b.line < to)
            .collect()
    }

    /// Whether `line` lies inside fenced or indented code.
    pub fn in_code(&self, line: usize) -> bool {
        self.fenced.iter().any(|&(s, e)| s <= line && line <= e)
    }

    /// `text`, the line numbered `line` as written, without the generated
    /// anchors the parse removed from it.
    pub fn clean_line<'a>(&self, line: usize, text: &'a str) -> std::borrow::Cow<'a, str> {
        let mut on_line: Vec<&Anchor> = self.anchors.iter().filter(|a| a.line == line).collect();
        if on_line.is_empty() {
            return std::borrow::Cow::Borrowed(text);
        }
        on_line.sort_by_key(|a| std::cmp::Reverse(a.raw_col));
        let mut out = text.to_string();
        for a in on_line {
            let end = a.raw_col + a.rendered().len();
            if end <= out.len() && out.is_char_boundary(a.raw_col) && out.is_char_boundary(end) {
                out.replace_range(a.raw_col..end, "");
            }
        }
        std::borrow::Cow::Owned(out)
    }

    /// Whether byte `col` of `line` lies inside a code span.
    pub fn in_code_span(&self, line: usize, col: usize) -> bool {
        self.code_spans
            .iter()
            .any(|&(l, s, e)| l == line && s <= col && col < e)
    }

    /// Whether byte `col` of `line` lies strictly inside a code span or a
    /// link, and where that span starts.
    pub fn span_around(&self, line: usize, col: usize) -> Option<usize> {
        self.code_spans
            .iter()
            .chain(&self.link_spans)
            .filter(|&&(l, s, e)| l == line && s < col && col < e)
            .map(|&(_, s, _)| s)
            .min()
    }

    /// `text`, the line numbered `line`, with every code span blanked.
    /// Lengths, byte offsets and line numbers are kept, so a finding still
    /// points where the text is.
    // sokf-implements: TLMD_111
    pub fn mask_spans<'a>(&self, line: usize, text: &'a str) -> std::borrow::Cow<'a, str> {
        let spans: Vec<(usize, usize)> = self
            .code_spans
            .iter()
            .filter(|&&(l, _, _)| l == line)
            .map(|&(_, s, e)| (s.min(text.len()), e.min(text.len())))
            .filter(|&(s, e)| s < e && text.is_char_boundary(s) && text.is_char_boundary(e))
            .collect();
        if spans.is_empty() {
            return std::borrow::Cow::Borrowed(text);
        }
        let mut out = text.to_string();
        for (start, end) in spans {
            // One byte of space per byte blanked, so every later offset
            // and the line's own length stay as they were.
            out.replace_range(start..end, &" ".repeat(end - start));
        }
        std::borrow::Cow::Owned(out)
    }
}

/// The headings inside `blocks` at any depth, through quotes and the
/// blocks inside items, each by its line and as written.
pub(super) fn headings_in(blocks: &[Block]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for b in blocks {
        match (&b.kind, &b.detail) {
            (BlockKind::Heading, _) => {
                out.extend(b.units.iter().map(|u| (u.line, u.text.clone())));
            }
            (_, BlockDetail::Quote { headings }) => out.extend(headings.iter().cloned()),
            _ => {
                for u in &b.units {
                    out.extend(headings_in(&u.nested));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_headings_are_the_headings_inside_an_item_or_a_quote() {
        let text = "# Top\n\n> ## Inner\n>\n> Text.\n\n- ## In item\n";
        let segments = crate::markdown::segments(text, 1, &Dialect::default());
        let body = crate::markdown::parse(text, 1, &segments, &Dialect::default());
        let nested = body.nested_headings();
        let seen: Vec<(usize, u8, &str, &str)> = nested
            .iter()
            .map(|h| (h.line, h.level, h.text.as_str(), h.slug.as_str()))
            .collect();
        assert_eq!(
            seen,
            vec![(3, 2, "Inner", "inner"), (7, 2, "In item", "in-item")]
        );
        assert_eq!(body.headings.len(), 1);
    }

    #[test]
    fn kinds_round_trip_through_their_names() {
        for name in [
            "prose",
            "bullet-list",
            "numbered-list",
            "table",
            "code",
            "blockquote",
            "image",
            "region",
            "html",
            "thematic-break",
            "references",
            "group",
        ] {
            assert_eq!(BlockKind::parse(name).unwrap().name(), name);
        }
        assert!(BlockKind::parse("paragraph").is_none());
        let host = Dialect {
            region: Some("include"),
            ..Dialect::default()
        };
        assert_eq!(
            BlockKind::parse_in("include", &host),
            Some(BlockKind::Region)
        );
        assert_eq!(BlockKind::parse_in("region", &host), None);
        assert_eq!(BlockKind::Region.name_in(&host), "include");
        assert_eq!(BlockKind::parse_in("include", &Dialect::default()), None);
    }

    #[test]
    fn blocks_between_and_in_code() {
        let block = |line| Block {
            kind: BlockKind::Prose,
            line,
            end_line: line,
            units: vec![],
            detail: BlockDetail::None,
            html_lines: vec![],
        };
        let body = Body {
            blocks: vec![block(2), block(5), block(9)],
            fenced: vec![(4, 6)],
            ..Body::default()
        };
        let lines: Vec<usize> = body.blocks_between(1, 9).iter().map(|b| b.line).collect();
        assert_eq!(lines, vec![2, 5]);
        assert!(body.in_code(5));
        assert!(!body.in_code(7));
    }
}
