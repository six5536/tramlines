//! The link reference definitions that open a paragraph or a setext
//! heading. Comrak consumes them, as CommonMark does. Where it keeps the
//! block's start on their first line, as comrak 0.55 did, it counts the
//! lines of the block's inline nodes from that start, and their columns
//! from where the text starts on the line that it counted, so the parser
//! moves each position past the definitions. comrak 1.1 starts the block
//! past them, so the parser finds no definitions to move past.

use std::collections::HashMap;

use comrak::nodes::{AstNode, NodeValue, Sourcepos};

use crate::markdown::link_definitions::only_definitions;

/// How many lines of link reference definitions open each paragraph and
/// setext heading of a parse, by the block's start.
#[derive(Debug)]
pub(in crate::markdown) struct Defined(HashMap<(usize, usize), usize>);

impl Defined {
    /// The definitions that open each block under `root`, the parse of
    /// `lines`.
    pub(in crate::markdown) fn of<'b>(root: &'b AstNode<'b>, lines: &[&str]) -> Self {
        let mut found = HashMap::new();
        for node in root.descendants() {
            let k = count(node, lines);
            if k > 0 {
                let start = node.data().sourcepos.start;
                found.insert((start.line, start.column), k);
            }
        }
        Self(found)
    }

    /// The lines of definitions that open `block`.
    fn lines_of<'b>(&self, block: &'b AstNode<'b>) -> usize {
        let start = block.data().sourcepos.start;
        self.0
            .get(&(start.line, start.column))
            .copied()
            .unwrap_or(0)
    }
}

/// How many lines past the last run of definitions the count looks for
/// more. A definition reaches further only through a title that wraps.
const REACH: usize = 4;

/// How many lines of link reference definitions that comrak consumed open
/// `block`, a node of the parse of `lines`: the longest run of its first
/// lines that comrak, reading them alone as written, reads as definitions
/// and nothing else, read from the lines rather than from comrak's
/// positions of the inline nodes. 0 for a block of another kind, for a paragraph that
/// comrak takes off a table, which keeps its definitions as text, and for
/// a block whose first line opens no definition.
fn count<'b>(block: &'b AstNode<'b>, lines: &[&str]) -> usize {
    let ast = block.data();
    let sp = ast.sourcepos;
    let last = match &ast.value {
        NodeValue::Paragraph if !super::precedes_table(block) => sp.end.line,
        // The underline ends a setext heading.
        NodeValue::Heading(h) if h.setext => sp.end.line.saturating_sub(1),
        _ => return 0,
    };
    // A definition opens with its label, which can wrap.
    let first = super::line_at(lines, sp.start.line)
        .get(sp.start.column.saturating_sub(1)..)
        .unwrap_or("");
    if !(first.starts_with('[') && (first.contains("]:") || !first.contains(']'))) {
        return 0;
    }
    // The lines as written keep the markers of the containers, so comrak
    // reads a lazy line as it does in the document. They lose the indent
    // of the first line, at most, which alone would make them code.
    let lead = |l: &str| l.len() - l.trim_start_matches(' ').len();
    let first_lead = lead(super::line_at(lines, sp.start.line));
    let written: Vec<&str> = (sp.start.line..=last)
        .map(|n| {
            let src = super::line_at(lines, n);
            &src[lead(src).min(first_lead)..]
        })
        .collect();
    // Definitions alone make no block, so the block's last line is text.
    let mut found = 0;
    for j in 1..written.len() {
        if only_definitions(&written[..j].join("\n")) {
            found = j;
        } else if j > found + REACH {
            break;
        }
    }
    found
}

/// A container of a block, as comrak reads it at the start of each line.
#[derive(Debug, Clone, Copy)]
enum Container {
    /// A quote: a line that isn't lazy opens with `>` and one space.
    Quote,
    /// A list item or a footnote definition: a line that isn't lazy opens
    /// with this many spaces.
    Indent(usize),
    /// A task item. Comrak keeps no indent for it, so each line counts as
    /// one that isn't lazy.
    Task,
}

/// The containers that hold `block`, the outermost first.
fn containers<'b>(block: &'b AstNode<'b>) -> Vec<Container> {
    let mut out: Vec<Container> = block
        .ancestors()
        .skip(1)
        .filter_map(|a| match &a.data().value {
            NodeValue::BlockQuote => Some(Container::Quote),
            NodeValue::Item(list) => Some(Container::Indent(list.marker_offset + list.padding)),
            NodeValue::FootnoteDefinition(_) => Some(Container::Indent(4)),
            NodeValue::TaskItem(_) => Some(Container::Task),
            _ => None,
        })
        .collect();
    out.reverse();
    out
}

/// The byte of `src`, line `n`, from which comrak counts the columns of
/// the block at `sp` inside `containers`: the block's start on its first
/// line, and elsewhere the start of the text after the markers of the
/// containers. Comrak skips the spaces before the text of a line that
/// isn't lazy, and keeps them on a lazy line, which lacks the marker of a
/// container.
fn indent(src: &str, sp: Sourcepos, n: usize, containers: &[Container]) -> usize {
    if n == sp.start.line {
        return sp.start.column.saturating_sub(1);
    }
    let bytes = src.as_bytes();
    let spaces = |at: usize| {
        bytes[at.min(bytes.len())..]
            .iter()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count()
    };
    let mut at = 0;
    for &c in containers {
        let ws = spaces(at);
        match c {
            Container::Quote if ws <= 3 && bytes.get(at + ws) == Some(&b'>') => {
                at += ws + 1;
                if matches!(bytes.get(at), Some(b' ' | b'\t')) {
                    at += 1;
                }
            }
            Container::Indent(need) if ws >= need => at += need,
            Container::Task => at += ws,
            _ => return at,
        }
    }
    at + spaces(at)
}

/// The position of `block`, a node of the parse of `lines`, its start
/// moved past the definitions that open it.
pub(super) fn block_pos<'b>(
    block: &'b AstNode<'b>,
    lines: &[&str],
    defined: &Defined,
) -> Sourcepos {
    let mut sp = block.data().sourcepos;
    let k = defined.lines_of(block);
    if k > 0 {
        let start = sp.start.line + k;
        let src = super::line_at(lines, start);
        sp.start.column = indent(src, sp, start, &containers(block)) + 1;
        sp.start.line = start;
    }
    sp
}

/// The position of `node`, an inline node of the parse of `lines`, moved
/// past the definitions that open its paragraph or its heading: down by
/// their lines, and from the indent of the line that comrak counted to the
/// indent of the line that holds it.
pub(super) fn inline_pos<'b>(
    node: &'b AstNode<'b>,
    lines: &[&str],
    defined: &Defined,
) -> Sourcepos {
    let mut sp = node.data().sourcepos;
    let Some(block) = node
        .ancestors()
        .find(|a| matches!(a.data().value, NodeValue::Paragraph | NodeValue::Heading(_)))
    else {
        return sp;
    };
    let k = defined.lines_of(block);
    if k == 0 {
        return sp;
    }
    let at = block.data().sourcepos;
    let around = containers(block);
    let indent_of = |n: usize| indent(super::line_at(lines, n), at, n, &around);
    // A column of 0 is no position.
    let moved = |line: usize, col: usize| {
        if col == 0 {
            0
        } else {
            (col + indent_of(line + k)).saturating_sub(indent_of(line))
        }
    };
    sp.start.column = moved(sp.start.line, sp.start.column);
    sp.end.column = moved(sp.end.line, sp.end.column);
    sp.start.line += k;
    sp.end.line += k;
    sp
}
