//! Parses a body with comrak into `Body`, grouping per grammar `block` and
//! `unit`. Region interiors and the definition block are masked first, so
//! generated content never reaches block matching and every reference label
//! in the host's scheme reaches the broken-link callback.

use std::{ops::Range, sync::Arc};

use comrak::{
    Arena, Options, ResolvedReference,
    nodes::{AstNode, ListType, NodeValue, Sourcepos},
    options::{BrokenLinkCallback, BrokenLinkReference},
};

use super::{
    Block, BlockDetail, BlockKind, Body, Dialect, Heading, Link, Marker, Segments, Unit, anchor,
    body, code, dialect,
    link::{REF, target},
    segments as seg, slug,
};

mod columns;
mod definitions;
mod text;

pub(super) use columns::written_pos;
use columns::{line_at, precedes_table};
pub(super) use definitions::Defined;
pub(super) use text::strip_prefix;

struct KeepLabel;

impl BrokenLinkCallback for KeepLabel {
    fn resolve(&self, r: BrokenLinkReference<'_>) -> Option<ResolvedReference> {
        // A footnote label is not a link label.
        if r.original.starts_with('^') {
            return None;
        }
        Some(ResolvedReference::new(
            format!("{REF}{}", r.original),
            String::new(),
        ))
    }
}

/// The options that the engine gives comrak.
pub(super) fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.footnotes = true;
    options.extension.tasklist = true;
    // A footnote's definition stays where it is written, and one that no
    // reference names stays too, so that the code blocks it holds are read.
    options.parse.leave_footnote_definitions = true;
    options.parse.broken_link_callback = Some(Arc::new(KeepLabel));
    options
}

/// Parse a body whose first line is file line `first_line`, its markers as
/// `dialect` spells them.
// sokf-implements: TLMD_4
// sokf-implements: TLMD_5
// sokf-implements: TLMD_6
pub fn parse(body: &str, first_line: usize, segments: &Segments, dialect: &Dialect) -> Body {
    let (masked, written) = mask(body, first_line, segments);
    let options = options();
    // Every rule reads the text without its generated anchors (TLMD_107).
    let stripped = anchor::strip(&masked, first_line, &options);
    let masked = stripped.text;
    // Each line as comrak ends it, so a line that comrak numbers reads the
    // text that comrak read there.
    // sokf-implements: MD-3_AC-14
    let mut lines: Vec<&str> = code::each_line(&masked).map(|l| &masked[l]).collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let arena = Arena::new();
    // comrak reads a stand-in on the line of an item that holds only
    // anchors, so that it reads the blocks that the document holds with
    // them, and the rules read the line without it.
    let parsed = stripped.parsed.as_deref().unwrap_or(&masked);
    let root = comrak::parse_document(&arena, parsed, &options);

    let mut p = Parser {
        dialect,
        lines: &lines,
        defined: Defined::of(root, &lines),
        offset: first_line.saturating_sub(1),
        body: Body {
            first_line,
            anchors: stripped.anchors,
            ..Body::default()
        },
        covered: Vec::new(),
        depth: 0,
    };
    let blocks = p.group(root.children());
    p.body.blocks = blocks;
    // The removal of the anchors keeps every line, so `written` holds the
    // bytes of each line of the tree.
    let found = code::code_blocks(root, &written);
    p.body.fenced = found
        .iter()
        .map(|&(first, last, _)| (first + p.offset, last + p.offset))
        .collect();
    p.body.code_blocks = found.into_iter().map(|(_, _, b)| b).collect();
    p.collect_inlines(root);
    p.reference_blocks();
    p.body.headings.sort_by_key(|h| h.line);
    p.body.links.sort_by_key(|l| l.line);
    p.body.blocks.sort_by_key(|b| b.line);
    p.body
}

/// The byte ranges of `text`, one line parsed alone, such as a squashed
/// example, that lie in a code span, in order: what a format's bans leave
/// out of a unit.
// sokf-implements: TLMD_111
pub(crate) fn code_spans_in(text: &str) -> Vec<Range<usize>> {
    if !text.contains('`') {
        return Vec::new();
    }
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, &options());
    let mut out: Vec<Range<usize>> = root
        .descendants()
        .filter_map(|n| {
            let ast = n.data();
            let sp = ast.sourcepos;
            matches!(ast.value, NodeValue::Code(_))
                .then(|| sp.start.column.saturating_sub(1)..sp.end.column.min(text.len()))
        })
        .filter(|r| r.start < r.end)
        .collect();
    out.sort_by_key(|r| r.start);
    out
}

/// Whether `cell`, a table cell, holds no inline node. Comrak adds such a
/// cell for each one missing from a body row, with no width, and GFM reads
/// it as empty. A written cell holds no node only where it is empty.
// sokf-implements: TLMD_4
fn empty_cell<'b>(cell: &'b AstNode<'b>) -> bool {
    cell.first_child().is_none()
}

/// Blank every include interior and the definition block, keeping line
/// numbers, with the bytes in `body` of each line of the result, as comrak
/// ends a line. A blanked line stands for every line that comrak reads in
/// the line of `body` that it blanks.
fn mask(body: &str, first_line: usize, segments: &Segments) -> (String, Vec<Range<usize>>) {
    let mut out = String::with_capacity(body.len());
    let lines = code::lines(body);
    let mut written = Vec::with_capacity(lines.len());
    let mut next = 0;
    let mut at = 0;
    for (i, line) in body.split_inclusive('\n').enumerate() {
        let file_line = first_line + i;
        let end = at + line.len();
        // sokf-implements: TLMD_41
        let hidden = segments.in_region(file_line) || segments.in_definition(file_line);
        if hidden {
            if line.ends_with('\n') {
                out.push('\n');
            }
            written.push(at..at + line.trim_end_matches(['\n', '\r']).len());
            while lines.get(next).is_some_and(|l| l.start < end) {
                next += 1;
            }
        } else {
            out.push_str(line);
            while let Some(l) = lines.get(next).filter(|l| l.start < end) {
                written.push(l.clone());
                next += 1;
            }
        }
        at = end;
    }
    written.extend(lines[next..].iter().cloned());
    (out, written)
}

/// The facts of a node, copied out so the node's borrow ends before
/// recursion.
enum Facts {
    Heading {
        level: u8,
        setext: bool,
    },
    Paragraph,
    List(ListType),
    Table,
    Code {
        lang: String,
        fence: usize,
        literal: String,
    },
    BlockQuote,
    Html(String),
    ThematicBreak,
    Footnote(String),
    Other,
}

/// The text of an HTML block that opens a list item, other than a comment:
/// what the author wrote as the item's text.
fn leading_html<'b>(node: &'b AstNode<'b>) -> Option<String> {
    let NodeValue::HtmlBlock(h) = &node.data().value else {
        return None;
    };
    let opens_item = node.previous_sibling().is_none()
        && node
            .parent()
            .is_some_and(|p| matches!(p.data().value, NodeValue::Item(_)));
    let text = h.literal.trim();
    (opens_item && !text.starts_with("<!--")).then(|| text.to_string())
}

fn facts(value: &NodeValue) -> Facts {
    match value {
        NodeValue::Heading(h) => Facts::Heading {
            level: h.level,
            setext: h.setext,
        },
        NodeValue::Paragraph => Facts::Paragraph,
        NodeValue::List(l) => Facts::List(l.list_type),
        NodeValue::Table(_) => Facts::Table,
        NodeValue::CodeBlock(c) => Facts::Code {
            lang: if c.fenced {
                c.info.split_whitespace().next().unwrap_or("").to_string()
            } else {
                String::new()
            },
            fence: if c.fenced { c.fence_length } else { 0 },
            literal: c.literal.clone(),
        },
        NodeValue::BlockQuote => Facts::BlockQuote,
        NodeValue::HtmlBlock(h) => Facts::Html(h.literal.trim().to_string()),
        NodeValue::ThematicBreak(_) => Facts::ThematicBreak,
        NodeValue::FootnoteDefinition(d) => Facts::Footnote(d.name.clone()),
        _ => Facts::Other,
    }
}

struct Parser<'a> {
    dialect: &'a Dialect,
    lines: &'a [&'a str],
    /// The lines of the definitions that open each paragraph and heading.
    defined: Defined,
    offset: usize,
    body: Body,
    /// Line ranges of leaf blocks, for finding reference definitions.
    covered: Vec<(usize, usize)>,
    /// How many items and quotes the nodes being grouped lie inside: a
    /// heading is a heading of the document only at none.
    depth: usize,
}

impl<'a> Parser<'a> {
    fn line(&self, sp: Sourcepos) -> usize {
        sp.start.line + self.offset
    }

    fn end_line(&self, sp: Sourcepos) -> usize {
        sp.end.line + self.offset
    }

    /// The source line at file line `n`, or empty.
    fn src(&self, file_line: usize) -> &'a str {
        file_line
            .checked_sub(self.offset)
            .map_or("", |n| line_at(self.lines, n))
    }

    fn cover(&mut self, sp: Sourcepos) {
        let range = (self.line(sp), self.end_line(sp));
        self.covered.push(range);
    }

    /// Cover the lines of `node`, a paragraph or a heading that starts at
    /// `sp`, past its opening definitions. Inside an item or a quote, the
    /// definitions that open a paragraph or a setext heading stay covered,
    /// because `reference_blocks` reads each line that no block covers as a
    /// line of the top level. comrak starts the block past them, so they are
    /// the lines just above its start, up to a blank line, a line that a
    /// block covers, or the start of the item or the quote that holds it.
    fn cover_text<'b>(&mut self, node: &'b AstNode<'b>, sp: Sourcepos) {
        let mut sp = if self.depth > 0 {
            node.data().sourcepos
        } else {
            sp
        };
        // Only a paragraph or a setext heading opens with definitions.
        let opens = match &node.data().value {
            NodeValue::Paragraph => true,
            NodeValue::Heading(h) => h.setext,
            _ => false,
        };
        if self.depth > 0 && opens {
            let floor = node.parent().map_or(1, |p| p.data().sourcepos.start.line);
            while sp.start.line > floor {
                let above = self.offset + sp.start.line - 1;
                let covered = self.covered.iter().any(|&(s, e)| s <= above && above <= e);
                if covered || strip_prefix(self.src(above)).trim().is_empty() {
                    break;
                }
                sp.start.line -= 1;
            }
        }
        self.cover(sp);
    }

    /// Group the nodes inside an item or a quote into blocks, one level
    /// deeper.
    fn nested<'b>(&mut self, nodes: impl Iterator<Item = &'b AstNode<'b>>) -> Vec<Block> {
        self.depth += 1;
        let blocks = self.group(nodes);
        self.depth -= 1;
        blocks
    }

    /// Group sibling nodes into blocks.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass of the parser over sibling nodes, a split of which is a refactor of its own"
    )]
    fn group<'b>(&mut self, nodes: impl Iterator<Item = &'b AstNode<'b>>) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut last_prose = false;
        for node in nodes {
            // Only a paragraph that follows a paragraph joins its block.
            let after_prose = std::mem::take(&mut last_prose);
            let facts = facts(&node.data().value);
            // A paragraph or a heading starts past its opening definitions.
            let sp = definitions::block_pos(node, self.lines, &self.defined);
            let line = self.line(sp);
            let end_line = self.end_line(sp);
            let block = match facts {
                Facts::Heading { level, setext } => {
                    self.cover_text(node, sp);
                    let (text, pieces) = self.heading_text(sp, setext);
                    // A heading inside an item or a quote is part of that
                    // block, not a heading of the document. Its unit's text
                    // starts with its hashes, which map to no column.
                    // sokf-implements: TLMD_173
                    if self.depth > 0 {
                        let hashes = format!("{} ", "#".repeat(usize::from(level)));
                        let pieces = pieces
                            .into_iter()
                            .map(|p| super::Piece {
                                offset: p.offset + hashes.len(),
                                ..p
                            })
                            .collect();
                        let unit = Unit {
                            pieces,
                            ..Unit::leaf(format!("{hashes}{text}"), line, end_line)
                        };
                        blocks.push(Block::new(BlockKind::Heading, line, end_line, vec![unit]));
                        continue;
                    }
                    self.body.headings.push(Heading {
                        level,
                        slug: slug::slug(&text),
                        text,
                        line,
                        end_line,
                        setext,
                    });
                    continue;
                }
                Facts::Paragraph => {
                    self.cover_text(node, sp);
                    if let Some(images) = images_of(node, self.lines, &self.defined) {
                        let units = images
                            .into_iter()
                            .map(|(url, l)| Unit::leaf(url, l + self.offset, l + self.offset))
                            .collect();
                        Block::new(BlockKind::Image, line, end_line, units)
                    } else {
                        let (text, pieces) = self.mapped_text(sp);
                        let unit = Unit {
                            pieces,
                            ..Unit::leaf(text, line, end_line)
                        };
                        let html = self.html_lines(node);
                        last_prose = true;
                        if after_prose {
                            let prev = blocks.last_mut().expect("last prose block");
                            prev.units.push(unit);
                            prev.end_line = end_line;
                            prev.html_lines.extend(html);
                            continue;
                        }
                        Block {
                            html_lines: html,
                            ..Block::new(BlockKind::Prose, line, end_line, vec![unit])
                        }
                    }
                }
                Facts::List(list_type) => {
                    let kind = match list_type {
                        ListType::Bullet => BlockKind::BulletList,
                        ListType::Ordered => BlockKind::NumberedList,
                    };
                    let html = self.html_lines(node);
                    let units = node.children().map(|item| self.item(item)).collect();
                    Block {
                        html_lines: html,
                        ..Block::new(kind, line, end_line, units)
                    }
                }
                Facts::Table => {
                    self.cover(sp);
                    let html = self.html_lines(node);
                    self.table(node, line, end_line, html)
                }
                Facts::Code {
                    lang,
                    fence,
                    literal,
                } => {
                    // The block's own lines, which comrak's span can miss.
                    let (first, last) = code::span(sp.start.line, sp.end.line, fence > 0, &literal);
                    let end_line = last + self.offset;
                    self.covered.push((first + self.offset, end_line));
                    let unit = Unit::leaf(literal, line, end_line);
                    Block {
                        detail: BlockDetail::Code { lang, fence },
                        ..Block::new(BlockKind::Code, line, end_line, vec![unit])
                    }
                }
                Facts::BlockQuote => {
                    let html = self.html_lines(node);
                    let inner = self.nested(node.children());
                    let headings = body::headings_in(&inner);
                    let units = inner
                        .into_iter()
                        .filter(|b| b.kind == BlockKind::Prose)
                        .flat_map(|b| b.units)
                        .collect();
                    Block {
                        kind: BlockKind::Blockquote,
                        line,
                        end_line,
                        units,
                        detail: BlockDetail::Quote { headings },
                        html_lines: html,
                    }
                }
                Facts::Html(literal) => {
                    self.cover(sp);
                    match self.dialect.marker(&literal) {
                        Some(Marker::Open(target)) => Block {
                            detail: BlockDetail::Region { target },
                            ..Block::new(BlockKind::Region, line, end_line, Vec::new())
                        },
                        Some(_) => continue,
                        None if dialect::comment(&literal).is_some() => continue,
                        None => {
                            let unit = Unit::leaf(literal, line, end_line);
                            Block::new(BlockKind::Html, line, end_line, vec![unit])
                        }
                    }
                }
                Facts::ThematicBreak => {
                    self.cover(sp);
                    Block::new(BlockKind::ThematicBreak, line, end_line, Vec::new())
                }
                Facts::Footnote(name) => {
                    self.cover(sp);
                    self.body.footnote_defs.push((name, line));
                    // A definition is no block, so the paragraphs around it
                    // stay one prose block.
                    last_prose = after_prose;
                    continue;
                }
                Facts::Other => continue,
            };
            blocks.push(block);
        }
        blocks
    }

    /// A list item: its first paragraph is its text; the rest is nested.
    /// An item that opens with an HTML tag, which markdown reads as an
    /// HTML block, has the tag as its text, and the tag is its inline HTML.
    fn item<'b>(&mut self, item: &'b AstNode<'b>) -> Unit {
        let line = self.line(item.data().sourcepos);
        let mut text = String::new();
        let mut pieces = Vec::new();
        let mut end_line = line;
        let mut children = item.children().peekable();
        if let Some(first) = children.peek() {
            let is_para = matches!(first.data().value, NodeValue::Paragraph);
            let sp = definitions::block_pos(first, self.lines, &self.defined);
            if is_para && images_of(first, self.lines, &self.defined).is_none() {
                // Its opening definitions stay part of the item: comrak
                // starts the paragraph past them, and the item at its marker.
                let mut covers = first.data().sourcepos;
                covers.start = item.data().sourcepos.start;
                self.cover(covers);
                (text, pieces) = self.mapped_text(sp);
                end_line = self.end_line(sp);
                children.next();
            } else if let Some(html) = leading_html(first) {
                self.cover(sp);
                text = html;
                end_line = self.end_line(sp);
                children.next();
            }
        }
        let nested = self.nested(children);
        Unit {
            text,
            line,
            end_line,
            nested,
            pieces,
        }
    }

    fn table<'b>(
        &mut self,
        table: &'b AstNode<'b>,
        line: usize,
        end_line: usize,
        html_lines: Vec<usize>,
    ) -> Block {
        let mut columns = Vec::new();
        let mut units = Vec::new();
        for row in table.children() {
            let (header, sp) = {
                let ast = row.data();
                let NodeValue::TableRow(row) = &ast.value else {
                    continue;
                };
                (row.header, ast.sourcepos)
            };
            let row_line = self.line(sp);
            let cells: Vec<(String, Vec<super::Piece>)> = row
                .children()
                .map(|cell| {
                    if empty_cell(cell) {
                        (String::new(), Vec::new())
                    } else {
                        self.mapped_text(cell.data().sourcepos)
                    }
                })
                .collect();
            if header {
                columns = cells.into_iter().map(|(t, _)| t).collect();
            } else {
                // The cells joined by ` | `, each piece moved by where its
                // cell starts in the row's text.
                let mut text = String::new();
                let mut pieces = Vec::new();
                for (i, (cell, cell_pieces)) in cells.into_iter().enumerate() {
                    if i > 0 {
                        text.push_str(" | ");
                    }
                    let base = text.len();
                    pieces.extend(cell_pieces.into_iter().map(|p| super::Piece {
                        offset: base + p.offset,
                        ..p
                    }));
                    text.push_str(&cell);
                }
                units.push(Unit {
                    pieces,
                    ..Unit::leaf(text, row_line, row_line)
                });
            }
        }
        Block {
            detail: BlockDetail::Table { columns },
            html_lines,
            ..Block::new(BlockKind::Table, line, end_line, units)
        }
    }

    /// Lines of inline HTML other than comments under `node`, with the
    /// HTML an item opens with.
    fn html_lines<'b>(&self, node: &'b AstNode<'b>) -> Vec<usize> {
        let mut lines = Vec::new();
        for n in node.descendants() {
            let ast = n.data();
            let inline = matches!(&ast.value, NodeValue::HtmlInline(html) if !html.trim_start().starts_with("<!--"));
            if inline {
                lines.push(self.line(written_pos(n, self.lines, &self.defined)));
            } else if leading_html(n).is_some() {
                lines.push(self.line(ast.sourcepos));
            }
        }
        lines.sort_unstable();
        lines.dedup();
        lines
    }

    /// Links, images, footnote references and inline HTML of the whole body.
    fn collect_inlines<'b>(&mut self, root: &'b AstNode<'b>) {
        for n in root.descendants() {
            let ast = n.data();
            let line = || self.line(written_pos(n, self.lines, &self.defined));
            match &ast.value {
                NodeValue::Link(link) | NodeValue::Image(link) => {
                    let is_image = matches!(ast.value, NodeValue::Image(_));
                    let sp = written_pos(n, self.lines, &self.defined);
                    let source = self.source_text(sp);
                    if let Some(target) = target(&link.url, &source, self.dialect.scheme) {
                        self.body.links.push(Link {
                            target,
                            line: self.line(sp),
                            is_image,
                            end_line: self.end_line(sp),
                            inline: inline_destination(&source),
                        });
                    }
                    let spans = self.spans(sp);
                    self.body.link_spans.extend(spans);
                }
                NodeValue::FootnoteReference(r) => {
                    self.body.footnote_refs.push((r.name.clone(), line()));
                }
                NodeValue::HtmlInline(html) if !html.trim_start().starts_with("<!--") => {
                    self.body.inline_html.push(line());
                }
                // A code span may open on one line and close on another,
                // so each line it covers records its own range.
                // sokf-implements: TLMD_8
                // sokf-implements: TLMD_112
                NodeValue::Code(_) => {
                    let spans = self.spans(written_pos(n, self.lines, &self.defined));
                    self.body.code_spans.extend(spans);
                }
                _ => {}
            }
        }
        self.body.code_spans.sort_unstable();
        self.body.link_spans.sort_unstable();
        self.body.inline_html.sort_unstable();
        self.body.inline_html.dedup();
    }

    /// The byte range an inline node covers on each of its lines.
    fn spans(&self, sp: Sourcepos) -> Vec<(usize, usize, usize)> {
        let line = self.line(sp);
        let last = self.end_line(sp);
        (line..=last)
            .map(|file_line| {
                let start = if file_line == line {
                    sp.start.column.saturating_sub(1)
                } else {
                    0
                };
                let end = if file_line == last {
                    sp.end.column
                } else {
                    usize::MAX
                };
                (file_line, start, end)
            })
            .collect()
    }

    /// Reference definitions comrak consumed: lines outside every leaf block
    /// that read `[label]: destination`, as `references` blocks.
    fn reference_blocks(&mut self) {
        let mut current: Option<Block> = None;
        for (i, line) in self.lines.iter().enumerate() {
            let file_line = self.offset + 1 + i;
            let covered = self
                .covered
                .iter()
                .any(|&(s, e)| s <= file_line && file_line <= e);
            let def = if covered { None } else { seg::ref_def(line) };
            match def {
                Some((label, dest)) => {
                    self.body.ref_defs.push((label, dest, file_line));
                    let unit = Unit::leaf(line.trim().to_string(), file_line, file_line);
                    match current.as_mut() {
                        Some(b) => {
                            b.units.push(unit);
                            b.end_line = file_line;
                        }
                        None => {
                            current = Some(Block::new(
                                BlockKind::References,
                                file_line,
                                file_line,
                                vec![unit],
                            ));
                        }
                    }
                }
                None => {
                    if (covered || !line.trim().is_empty())
                        && let Some(b) = current.take()
                    {
                        self.body.blocks.push(b);
                    }
                }
            }
        }
        if let Some(b) = current.take() {
            self.body.blocks.push(b);
        }
    }
}

/// The text of an ATX heading line: leading hashes and a closing sequence
/// removed.
fn atx_text(line: &str) -> String {
    let t = line.trim_start_matches('#').trim();
    let t = t.trim_end_matches('#').trim_end();
    t.to_string()
}

/// The images of a paragraph of images only, of the parse of `lines`,
/// with their body lines.
fn images_of<'b>(
    paragraph: &'b AstNode<'b>,
    lines: &[&str],
    defined: &Defined,
) -> Option<Vec<(String, usize)>> {
    let mut images = Vec::new();
    for child in paragraph.children() {
        let ast = child.data();
        match &ast.value {
            NodeValue::Image(link) => {
                let line = written_pos(child, lines, defined).start.line;
                images.push((link.url.clone(), line));
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {}
            NodeValue::Text(t) if t.trim().is_empty() => {}
            _ => return None,
        }
    }
    if images.is_empty() {
        None
    } else {
        Some(images)
    }
}

/// An inline link's destination as written, `](…)`, when the link ends
/// with it on one line.
fn inline_destination(source: &str) -> Option<String> {
    let source = source.trim_end();
    if !source.ends_with(')') {
        return None;
    }
    let tail = &source[source.rfind("](")?..];
    (!tail.contains('\n')).then(|| tail.to_string())
}

#[cfg(test)]
mod tests;
