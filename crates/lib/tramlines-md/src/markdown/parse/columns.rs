//! Where comrak's positions of an inline node lie in the lines as
//! written: past the link reference definitions that open a paragraph, in
//! a table row indented apart from its table, and past the backslashes that
//! comrak drops from each escaped pipe.

use comrak::nodes::{AstNode, NodeValue, Sourcepos};

use super::definitions::{self, Defined};

/// The byte of `src`, a line as written, that comrak's column `col` stands
/// for after comrak drops backslashes from byte `from` of the line on.
/// Comrak drops the backslash of each `\|` whose backslash no backslash
/// before it escapes, and `as_written` skips the same backslashes.
fn as_written(src: &[u8], from: usize, col: usize) -> usize {
    let (mut at, mut read) = (from, from);
    let mut after_backslash = false;
    while at < src.len() {
        let byte = src[at];
        let dropped = !after_backslash && byte == b'\\' && src.get(at + 1) == Some(&b'|');
        after_backslash = !after_backslash && byte == b'\\';
        if !dropped {
            if read == col {
                return at;
            }
            read += 1;
        }
        at += 1;
    }
    at + col.saturating_sub(read)
}

/// `sp`, the position of `node`, a node of the parse of `lines`, with the
/// columns that comrak gives a body row of a table moved to the row's line
/// as written. Comrak reads a body row from the first byte of its line that
/// is no space, no tab, and no `>`, in its function `try_opening_row`, but
/// counts the row's columns from the column where the table starts, so a
/// row indented apart from the table's first line lies off by the
/// difference. A row holds no list marker, since one opens a list, and a
/// `>` there opens a quote. The header row keeps its columns, because
/// comrak counts them from the start of the header's own text, which can
/// follow the marker of a list item. The row's own end column is the true
/// one, so only the nodes inside the row move.
pub(super) fn row_pos<'b>(node: &'b AstNode<'b>, mut sp: Sourcepos, lines: &[&str]) -> Sourcepos {
    let row = node
        .ancestors()
        .skip(1)
        .find(|a| matches!(a.data().value, NodeValue::TableRow(false)));
    let Some((row, table)) = row.zip(row.and_then(|r| r.parent())) else {
        return sp;
    };
    let src = line_at(lines, row.data().sourcepos.start.line);
    let to = src.len() - src.trim_start_matches([' ', '\t', '>']).len() + 1;
    let from = table.data().sourcepos.start.column;
    // A column of 0 is no position.
    let moved = |col: usize| {
        if col == 0 {
            0
        } else {
            (col + to).saturating_sub(from)
        }
    };
    sp.start.column = moved(sp.start.column);
    sp.end.column = moved(sp.end.column);
    sp
}

/// The byte from which comrak drops a backslash before each `\|` on each
/// line of `node`, an inline node of the parse of `lines`: the start of its
/// table cell, or the start of the line in a paragraph that directly
/// precedes a table. Comrak takes that paragraph off the header of the
/// table, in its function `try_inserting_table_header_paragraph`, and
/// unescapes it as a cell. GFM doesn't specify this, so the tests of the
/// parser pin it. The markers of an item or a quote before the paragraph's
/// text hold no backslash. `None` where comrak drops none.
fn unescaped_from<'b>(node: &'b AstNode<'b>, lines: &[&str]) -> Option<usize> {
    for a in node.ancestors() {
        let ast = a.data();
        match ast.value {
            NodeValue::TableCell => {
                return Some(
                    row_pos(a, ast.sourcepos, lines)
                        .start
                        .column
                        .saturating_sub(1),
                );
            }
            NodeValue::Paragraph => return precedes_table(a).then_some(0),
            _ => {}
        }
    }
    None
}

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
/// lines and the columns as written. Comrak counts the lines of the inline
/// nodes of a paragraph that opens with link reference definitions from
/// the first line of the definitions, which `definitions` corrects. Its
/// columns in a body row of a table move by the row's indent, as `row_pos`
/// gives them; no paragraph lies in a row, so at most one of the two moves
/// a node. Comrak reads a table cell, and a paragraph that directly
/// precedes a table, with the backslash of each `\|` dropped, so a column
/// it gives there then lies one byte to the left for each such backslash
/// before it on its line.
// sokf-implements: TLMD_4
// sokf-implements: TLMD_8
// sokf-implements: TLMD_111
pub(in crate::markdown) fn written_pos<'b>(
    node: &'b AstNode<'b>,
    lines: &[&str],
    defined: &Defined,
) -> Sourcepos {
    let sp = definitions::inline_pos(node, lines, defined);
    let mut sp = row_pos(node, sp, lines);
    if let Some(from) = unescaped_from(node, lines) {
        // A column of 0 is no position, and a line without `\|` loses
        // no backslash.
        let moved = |n: usize, col: usize| {
            let src = line_at(lines, n);
            if src.contains("\\|") {
                as_written(src.as_bytes(), from, col)
            } else {
                col
            }
        };
        if sp.start.column > 0 {
            sp.start.column = moved(sp.start.line, sp.start.column - 1) + 1;
        }
        if sp.end.column > from {
            sp.end.column = moved(sp.end.line, sp.end.column - 1) + 1;
        }
    }
    sp
}

/// Line `n` of `lines`, counted from 1, or empty.
pub(super) fn line_at<'a>(lines: &[&'a str], n: usize) -> &'a str {
    n.checked_sub(1)
        .and_then(|i| lines.get(i))
        .copied()
        .unwrap_or("")
}
