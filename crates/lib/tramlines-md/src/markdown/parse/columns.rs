//! Where comrak's positions of an inline node lie in the lines as
//! written: past the link reference definitions that open a paragraph.
//! comrak 1.1 places the nodes of a table row, and those after an escaped
//! pipe, where the document holds them, so they need no correction.

use comrak::nodes::{AstNode, NodeValue, Sourcepos};

use super::definitions::{self, Defined};

/// Whether `paragraph` directly precedes a table: the table is its next
/// block and starts on the line after it.
pub(super) fn precedes_table<'b>(paragraph: &'b AstNode<'b>) -> bool {
    let end = paragraph.data().sourcepos.end.line;
    paragraph.next_sibling().is_some_and(|t| {
        let t = t.data();
        matches!(t.value, NodeValue::Table(_)) && t.sourcepos.start.line == end + 1
    })
}

/// The position of `node`, an inline node of the parse of `lines`, in the
/// lines and the columns as written: past the link reference definitions
/// that open its paragraph, where comrak counts its lines from the first
/// line of the definitions, as `definitions` corrects.
// sokf-implements: TLMD_4
// sokf-implements: TLMD_8
// sokf-implements: TLMD_111
pub(in crate::markdown) fn written_pos<'b>(
    node: &'b AstNode<'b>,
    lines: &[&str],
    defined: &Defined,
) -> Sourcepos {
    definitions::inline_pos(node, lines, defined)
}

/// Line `n` of `lines`, counted from 1, or empty.
pub(super) fn line_at<'a>(lines: &[&'a str], n: usize) -> &'a str {
    n.checked_sub(1)
        .and_then(|i| lines.get(i))
        .copied()
        .unwrap_or("")
}
