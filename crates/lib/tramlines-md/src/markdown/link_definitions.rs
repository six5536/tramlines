//! The link reference definitions of a run of lines, as comrak reads them.
//! comrak consumes a definition and keeps no node for it, so the parser
//! asks comrak which of the lines read as definitions and nothing else,
//! one definition at a time, and reads the label and the destination of
//! each from its lines.

use comrak::{Arena, nodes::NodeValue};

use super::parse::{options, strip_prefix};

/// One link reference definition of a run of lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::markdown) struct LinkDef {
    /// The index in the run of the line of its label.
    pub first: usize,
    /// The index in the run of its last line.
    pub last: usize,
    /// The label, its lines joined by one space.
    pub label: String,
    /// The destination as written.
    pub dest: String,
}

/// What a run of lines holds: its definitions, in order, and the index of
/// each line that holds text and lies in no definition.
#[derive(Debug, Default)]
pub(in crate::markdown) struct Read {
    pub defs: Vec<LinkDef>,
    pub other: Vec<usize>,
}

/// How many lines past the last line that reads as its end the search for
/// the end of a definition looks, and a quarter of how many lines past its
/// label. A definition reaches further only through a title that wraps
/// onto more lines.
const REACH: usize = 8;

/// Whether every line of `lines` that holds text lies in a definition.
pub(crate) fn all_definitions(lines: &[&str]) -> bool {
    let read = read(lines);
    read.other.is_empty() && !read.defs.is_empty()
}

/// The definitions of `lines`, as written with the markers of the quotes
/// and the items that hold them. A definition never holds a blank line,
/// so each run between blank lines reads alone.
// sokf-implements: TLMD_268
pub(in crate::markdown) fn read(lines: &[&str]) -> Read {
    let mut out = Read::default();
    let blank = |l: &str| strip_prefix(l).trim().is_empty();
    let mut s = 0;
    while s < lines.len() {
        if blank(lines[s]) {
            s += 1;
            continue;
        }
        let end = (s..lines.len())
            .find(|&i| blank(lines[i]))
            .unwrap_or(lines.len());
        read_run(&lines[s..end], s, &mut out);
        s = end;
    }
    out
}

/// The definitions of `run`, lines with no blank line among them, whose
/// first line is line `base` of the lines that `read` reads.
fn read_run(run: &[&str], base: usize, out: &mut Read) {
    let mut s = 0;
    while s < run.len() {
        match definition_at(run, s) {
            Some((end, label, dest)) => {
                // A line of the markers of an item alone ends no definition.
                let last = (s..end)
                    .rev()
                    .find(|&i| !opening(run[i]).is_empty())
                    .unwrap_or(s);
                out.defs.push(LinkDef {
                    first: base + s,
                    last: base + last,
                    label,
                    dest,
                });
                out.other.extend(base + last + 1..base + end);
                s = end;
            }
            // Text that is no definition and that the next line can carry
            // on, such as a paragraph, leaves no definition in the lines
            // after it. A line of the markers of an item alone, a heading,
            // or a comment carries on into no later line.
            None if !opening(run[s]).is_empty() && !ends_its_block(run[s]) => {
                out.other.extend(base + s..base + run.len());
                return;
            }
            None => {
                out.other.push(base + s);
                s += 1;
            }
        }
    }
}

/// The end, exclusive, the label, and the destination of the definition
/// that starts at line `s` of `run`, or `None` where none starts there.
/// The definition ends at the last line that keeps the lines from `s`
/// definitions and nothing else. The search stops at a line that opens
/// with `[` once the lines before it read so, because such a line carries
/// on no definition that already reads whole: a title opens with a quote
/// or a parenthesis.
fn definition_at(run: &[&str], s: usize) -> Option<(usize, String, String)> {
    if !opening(run[s]).starts_with('[') {
        return None;
    }
    let mut found = None;
    for j in s + 1..=run.len() {
        if only_definitions(&as_written(&run[s..j])) {
            found = Some(j);
            if j == run.len() || opening(run[j]).starts_with('[') {
                break;
            }
        } else if found.is_some_and(|f| j >= f + REACH) {
            break;
        }
        // Lines of the markers of items alone keep a definition read, so
        // the search stops past every title that wraps.
        if j >= s + 4 * REACH {
            break;
        }
    }
    let end = found?;
    let (label, dest) = label_and_destination(&run[s..end])?;
    Some((end, label, dest))
}

/// The label of the definition that tells whether a line ends its block.
const PROBE: &str = "tramlines-probe";

/// Whether comrak reads a definition on the line after `line`, so that
/// `line` leaves a definition after it to be read.
fn ends_its_block(line: &str) -> bool {
    let text = format!("{}[{PROBE}]: /{PROBE}\n\n[{PROBE}]\n", as_written(&[line]));
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &text, &options());
    root.descendants().any(
        |n| matches!(&n.data().value, NodeValue::Link(link) if link.url == format!("/{PROBE}")),
    )
}

/// `lines` joined, each ending with a line break, and each without the
/// spaces and tabs after its quote markers that the first line's indent
/// there spans at most, which alone would make them code. A tab reaches to
/// the next multiple of four columns.
fn as_written(lines: &[&str]) -> String {
    // The bytes of the indent of `rest` that end within `width` columns,
    // and the columns of the whole indent.
    let indent = |rest: &str, width: usize| {
        let (mut col, mut cut) = (0, 0);
        for (i, b) in rest.bytes().enumerate() {
            col = match b {
                b' ' => col + 1,
                b'\t' => col + 4 - col % 4,
                _ => break,
            };
            if col <= width {
                cut = i + 1;
            }
        }
        (cut, col)
    };
    let first = lines
        .first()
        .map_or(0, |l| indent(&l[quote_markers(l)..], usize::MAX).1);
    let mut out = String::new();
    for l in lines {
        let quotes = quote_markers(l);
        let (cut, _) = indent(&l[quotes..], first);
        out.push_str(&l[..quotes]);
        out.push_str(&l[quotes + cut..]);
        out.push('\n');
    }
    out
}

/// The length of the quote markers that open `line`: each up to three
/// spaces, a `>`, and the one space after it.
fn quote_markers(line: &str) -> usize {
    let mut at = 0;
    loop {
        let rest = &line[at..];
        let spaces = rest.len() - rest.trim_start_matches(' ').len();
        if spaces > 3 || !rest[spaces..].starts_with('>') {
            return at;
        }
        at += spaces + 1;
        if line[at..].starts_with(' ') {
            at += 1;
        }
    }
}

/// Whether comrak reads `text` alone as link reference definitions inside
/// containers, and nothing else.
pub(in crate::markdown) fn only_definitions(text: &str) -> bool {
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, &options());
    root.descendants().all(|n| {
        matches!(
            n.data().value,
            NodeValue::Document
                | NodeValue::BlockQuote
                | NodeValue::List(_)
                | NodeValue::Item(_)
                | NodeValue::TaskItem(_)
                | NodeValue::FootnoteDefinition(_)
        )
    })
}

/// `line` past the markers of the quotes, the items, and the task boxes
/// that open it, and the spaces after them.
fn opening(line: &str) -> &str {
    let mut t = line.trim_start();
    loop {
        let rest = t.strip_prefix('>').or_else(|| list_marker(t)).or_else(|| {
            ["[ ]", "[x]", "[X]"]
                .iter()
                .find_map(|b| t.strip_prefix(b))
                .filter(|r| r.starts_with([' ', '\t']))
        });
        match rest {
            Some(rest) => t = rest.trim_start(),
            None => return t,
        }
    }
}

/// `text` past the marker of a list item that opens it: `-`, `*`, or `+`,
/// or one to nine digits and `.` or `)`, then a space, a tab, or the end.
fn list_marker(text: &str) -> Option<&str> {
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let rest = match digits {
        0 => text.strip_prefix(['-', '*', '+'])?,
        1..=9 => text[digits..].strip_prefix(['.', ')'])?,
        _ => return None,
    };
    (rest.is_empty() || rest.starts_with([' ', '\t'])).then_some(rest)
}

/// The label and the destination as written of the definition that
/// `lines` hold, which comrak reads as one definition. The label runs from
/// the first `[` to the first `]` that no backslash escapes, over several
/// lines too, and the destination is the first word after the `]:`, on its
/// line or the next.
fn label_and_destination(lines: &[&str]) -> Option<(String, String)> {
    let mut texts = lines
        .iter()
        .enumerate()
        .map(|(i, l)| if i == 0 { opening(l) } else { strip_prefix(l) });
    let mut text = texts.next()?.strip_prefix('[')?;
    let mut label = String::new();
    let rest = loop {
        let mut escaped = false;
        let end = text.char_indices().find_map(|(i, c)| {
            let close = c == ']' && !escaped;
            escaped = c == '\\' && !escaped;
            close.then_some(i)
        });
        if let Some(end) = end {
            label.push_str(&text[..end]);
            break &text[end + 1..];
        }
        label.push_str(text);
        label.push(' ');
        text = texts.next()?;
    };
    // CommonMark ends a destination at a space, a tab, or a line break.
    let blank: &[char] = &[' ', '\t'];
    let rest = rest.strip_prefix(':')?.trim_matches(blank);
    let dest = if rest.is_empty() {
        texts.next()?.trim_matches(blank)
    } else {
        rest
    };
    // A destination in angle brackets runs to the first `>` that no
    // backslash escapes, spaces and all.
    let dest = match dest.strip_prefix('<') {
        Some(inner) => {
            let mut escaped = false;
            let end = inner.char_indices().find_map(|(i, c)| {
                let close = c == '>' && !escaped;
                escaped = c == '\\' && !escaped;
                close.then_some(i)
            })?;
            &dest[..end + 2]
        }
        None => dest.split(blank).next().filter(|d| !d.is_empty())?,
    };
    // A label that opens with `^` is a footnote's, whose definition holds
    // the definitions after it.
    let footnote = label.starts_with('^');
    let label = label.trim();
    (!label.is_empty() && !footnote).then(|| (label.to_string(), dest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first line, the last line, the label, and the destination of a
    /// definition.
    type Found = (usize, usize, String, String);

    /// Each definition of `text`, with the lines that hold other text.
    fn defs(text: &str) -> (Vec<Found>, Vec<usize>) {
        let lines: Vec<&str> = text.lines().collect();
        let read = read(&lines);
        let defs = read
            .defs
            .into_iter()
            .map(|d| (d.first, d.last, d.label, d.dest))
            .collect();
        (defs, read.other)
    }

    fn def(first: usize, last: usize, label: &str, dest: &str) -> Found {
        (first, last, label.to_string(), dest.to_string())
    }

    // sokf-tests: TLMD_268
    #[test]
    fn a_definition_takes_every_line_of_its_destination_and_its_title() {
        for (text, last) in [
            ("[a]: /x", 0),
            ("[a]:\n/x", 1),
            ("[a]:\n  /x", 1),
            ("[a]: /x\n\"t\"", 1),
            ("[a]: /x\n  'a\n  long t'", 2),
            ("[a]:\n  /x\n  \"t\"", 2),
            ("[a]: /x \"a\nlong\nt\"", 2),
            ("> [a]:\n> /x", 1),
            ("> [a]:\n/x", 1),
            ("- [a]:\n  /x", 1),
        ] {
            assert_eq!(
                defs(text),
                (vec![def(0, last, "a", "/x")], vec![]),
                "{text:?}"
            );
        }
    }

    // sokf-tests: TLMD_268
    #[test]
    fn each_definition_of_a_run_reads_apart() {
        assert_eq!(
            defs("[a]: /x\n[b]:\n  /y\n  \"t\"\n\n[c d\ne]: /z"),
            (
                vec![
                    def(0, 0, "a", "/x"),
                    def(1, 3, "b", "/y"),
                    def(5, 6, "c d e", "/z")
                ],
                vec![]
            )
        );
        // A destination that opens with `[` belongs to the label before it.
        assert_eq!(
            defs("[a]:\n[x]\n[b]: /y"),
            (vec![def(0, 1, "a", "[x]"), def(2, 2, "b", "/y")], vec![])
        );
        assert_eq!(
            defs("- [a]: /x\n-\n- [b]: /y"),
            (vec![def(0, 0, "a", "/x"), def(2, 2, "b", "/y")], vec![1])
        );
        assert_eq!(
            defs("[a]: <k.md>\n[b]: <k b.md> \"t\"\n>     [c]: /z"),
            (
                vec![
                    def(0, 0, "a", "<k.md>"),
                    def(1, 1, "b", "<k b.md>"),
                    def(2, 2, "c", "/z")
                ],
                vec![]
            )
        );
        assert_eq!(
            defs("\t[a]: /x\n\n- b\n\n\t[c]: /z\u{a0}w"),
            (
                vec![def(0, 0, "a", "/x"), def(4, 4, "c", "/z\u{a0}w")],
                vec![2]
            )
        );
        assert_eq!(
            defs("> 1. - [a]: /x\n>      [b]: /y"),
            (vec![def(0, 0, "a", "/x"), def(1, 1, "b", "/y")], vec![])
        );
        assert_eq!(
            defs("[a\\]b]: /x"),
            (vec![def(0, 0, "a\\]b", "/x")], vec![])
        );
    }

    // sokf-tests: TLMD_268
    #[test]
    fn text_that_comrak_reads_as_no_definition_is_other_text() {
        assert_eq!(
            defs("[a]: /x\nnot a def"),
            (vec![def(0, 0, "a", "/x")], vec![1])
        );
        assert_eq!(defs("[a]: /x \"t"), (vec![], vec![0]));
        assert_eq!(defs("[a]:"), (vec![], vec![0]));
        assert_eq!(defs("[a][b]: /x"), (vec![], vec![0]));
        assert_eq!(defs("text"), (vec![], vec![0]));
        assert_eq!(defs("[^n]: [a]: /x"), (vec![], vec![0]));
        assert_eq!(defs("[ ^a]: /x"), (vec![def(0, 0, "^a", "/x")], vec![]));
        // A heading, a comment, or a rule ends its block, so a definition
        // after it reads.
        for between in ["# H", "<!-- note -->", "***"] {
            assert_eq!(
                defs(&format!("[a]: /x\n{between}\n[b]: /y")),
                (vec![def(0, 0, "a", "/x"), def(2, 2, "b", "/y")], vec![1]),
                "{between}"
            );
        }
        assert_eq!(
            defs("[a]: /x\n<div>\n[b]: /y"),
            (vec![def(0, 0, "a", "/x")], vec![1, 2])
        );
        assert_eq!(defs("[a]: /x \"t\n[b]: /y"), (vec![], vec![0, 1]));
        // A title on the line after the definition belongs to it, a later
        // line of other text doesn't.
        assert_eq!(
            defs("[a]: /x\n\"t\"\nmore"),
            (vec![def(0, 1, "a", "/x")], vec![2])
        );
    }
}
