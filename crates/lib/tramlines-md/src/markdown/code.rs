//! The code blocks of a body as comrak reads them, at any depth of quotes,
//! lists, and footnotes, each with the bytes of its lines in the text as
//! written.

use std::ops::Range;

use comrak::{
    Arena,
    nodes::{AstNode, NodeValue},
};

use super::{CodeBlock, parse};

/// The bytes of each line of `text`, without its ending, as comrak ends a
/// line: at `\n`, at `\r\n`, or at `\r` alone. The last entry is the text
/// after the last ending, empty when the text ends with one.
pub(super) fn lines(text: &str) -> Vec<Range<usize>> {
    each_line(text).collect()
}

/// The lines of `lines`, one at a time.
pub(super) fn each_line(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    let mut done = false;
    std::iter::from_fn(move || {
        while i < bytes.len() {
            let ending = match bytes[i] {
                b'\r' if bytes.get(i + 1) == Some(&b'\n') => 2,
                b'\n' | b'\r' => 1,
                _ => 0,
            };
            i += ending.max(1);
            if ending > 0 {
                let line = start..i - ending;
                start = i;
                return Some(line);
            }
        }
        (!std::mem::replace(&mut done, true)).then_some(start..bytes.len())
    })
}

/// The number of lines of `content`, whose last line can lack its ending.
fn content_lines(content: &str) -> usize {
    each_line(content).count() - usize::from(content.is_empty() || content.ends_with(['\n', '\r']))
}

/// The first and the last line of a code block that comrak reads from
/// `first` to `end`, with `content` between its fences. The last line is
/// the closing fence where the block has one, and otherwise its last line
/// that is not blank. comrak counts the blank lines after an indented
/// block in its span, and ends a fence that a list item leaves open before
/// blank lines on its opening line, while its content keeps its lines.
pub(super) fn span(first: usize, end: usize, fenced: bool, content: &str) -> (usize, usize) {
    // CommonMark counts a line of spaces and tabs alone as blank.
    let kept = content_lines(content.trim_end_matches([' ', '\t', '\n', '\r']));
    if !fenced {
        return (first, first + kept.saturating_sub(1));
    }
    let closed = end > first + content_lines(content);
    (first, if closed { end } else { first + kept })
}

/// Each code block under `root`, in the order of the text: its first and
/// its last line in the tree, and the block with the bytes of those lines.
/// `lines` holds the bytes of each line of the tree in the text as
/// written, in order.
// sokf-implements: TLMD_8
// sokf-implements: MD-3_AC-13
// sokf-implements: MD-3_AC-14
pub(super) fn code_blocks<'a>(
    root: &'a AstNode<'a>,
    lines: &[Range<usize>],
) -> Vec<(usize, usize, CodeBlock)> {
    let past = lines.last().map_or(0, |l| l.end);
    let line = |n: usize| n.checked_sub(1).and_then(|i| lines.get(i));
    root.descendants()
        .filter_map(|node| {
            let ast = node.data();
            let NodeValue::CodeBlock(code) = &ast.value else {
                return None;
            };
            let sp = ast.sourcepos;
            let (first, last) = span(sp.start.line, sp.end.line, code.fenced, &code.literal);
            let start = line(first).map_or(past, |l| l.start);
            let end = line(last).map_or(past, |l| l.end);
            let block = CodeBlock {
                bytes: start..end,
                fenced: code.fenced,
            };
            Some((first, last, block))
        })
        .collect()
}

/// The bytes of each fenced code block of `text`, parsed as written with
/// the parser's options. A text without three backticks or three tildes
/// holds no fence, and costs no parse.
pub(super) fn fenced(text: &str) -> Vec<Range<usize>> {
    if !text.contains("```") && !text.contains("~~~") {
        return Vec::new();
    }
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, &parse::options());
    code_blocks(root, &lines(text))
        .into_iter()
        .filter(|(_, _, b)| b.fenced)
        .map(|(_, _, b)| b.bytes)
        .collect()
}

#[cfg(test)]
mod tests;
