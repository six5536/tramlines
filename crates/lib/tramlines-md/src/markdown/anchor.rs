//! The generated anchors (`TLMD_104` to `TLMD_109`): `<a id="<slug>"></a>` before each key.
//! They are found by form, outside code, and removed from the text before
//! it is parsed, so no rule ever reads one.

use std::collections::HashSet;

use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
};
use serde::{Deserialize, Serialize};

use super::{code, parse};

/// One generated anchor as found in a body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    /// The `id`, as written.
    pub id: String,
    /// The file line.
    pub line: usize,
    /// The byte column in the line with every anchor removed.
    pub col: usize,
    /// The byte column in the line as written.
    pub raw_col: usize,
    /// Whether the line holds nothing but anchors.
    pub alone: bool,
    /// Whether the line holds only the anchors of the first item of a list
    /// whose marker alone, without them, Markdown reads as no item, such as
    /// a marker that joins the paragraph before it.
    #[serde(default)]
    pub joins_text: bool,
}

impl Anchor {
    /// The anchor as written.
    pub fn rendered(&self) -> String {
        render(&self.id)
    }
}

/// The generated form of an anchor.
pub fn render(id: &str) -> String {
    format!("<a id=\"{id}\"></a>")
}

/// The id of an open tag of the generated form, `<a id="…">`.
fn open_id(html: &str) -> Option<&str> {
    let id = html.strip_prefix("<a id=\"")?.strip_suffix("\">")?;
    let valid = !id.is_empty()
        && !id
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '<' | '>'));
    valid.then_some(id)
}

/// The stand-in that comrak reads as the text of an item whose own line
/// holds only anchors, so that the item stays an item.
const STAND_IN: &str = "x";

/// A text without its generated anchors.
#[derive(Debug)]
pub(super) struct Stripped {
    /// The text without its anchors, which every rule reads.
    pub(super) text: String,
    /// The text that comrak parses, where it differs from `text`: a stand-in
    /// on each line of a list item that holds only anchors after its
    /// marker, where its first anchor stood.
    pub(super) parsed: Option<String>,
    /// The anchors found.
    pub(super) anchors: Vec<Anchor>,
}

impl Stripped {
    fn unchanged(text: &str) -> Self {
        Stripped {
            text: text.to_string(),
            parsed: None,
            anchors: Vec::new(),
        }
    }
}

/// An anchor that comrak found, before its line is cut.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Found {
    /// The body line.
    line: usize,
    /// The start byte in the line.
    start: usize,
    /// The end byte in the line.
    end: usize,
    /// The id.
    id: String,
    /// What `opens_item` says of the anchor.
    opens: Option<Opens>,
}

/// The item whose own line an anchor opens.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Opens {
    /// Whether the item is the first of its list.
    first: bool,
    /// The column where comrak starts the item.
    column: usize,
}

/// Where `node`, an anchor's open tag on body line `line`, is the first
/// content of a list item on the item's own line, that item.
fn opens_item<'b>(node: &'b AstNode<'b>, line: usize) -> Option<Opens> {
    let mut at = node;
    let item = loop {
        if at.previous_sibling().is_some() {
            return None;
        }
        at = at.parent()?;
        if matches!(at.data().value, NodeValue::Item(_) | NodeValue::TaskItem(_)) {
            break at;
        }
    };
    let start = item.data().sourcepos.start;
    (start.line == line).then(|| Opens {
        first: item.previous_sibling().is_none(),
        column: start.column,
    })
}

/// The blank line that keeps a line `raw` apart from the block before it:
/// the indentation and the quote markers that open `raw`, without the
/// spaces after them.
pub(crate) fn gap(raw: &str) -> String {
    let markers: String = raw
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t' | '>'))
        .collect();
    markers.trim_end().to_string()
}

/// `text` with a gap before each of the body lines `before`, as a repair
/// writes it.
fn with_gaps(text: &str, before: &[usize]) -> String {
    let lines = code::lines(text);
    let mut out = String::with_capacity(text.len() + before.len());
    for (i, range) in lines.iter().enumerate() {
        let end = lines.get(i + 1).map_or(text.len(), |l| l.start);
        if before.contains(&(i + 1)) {
            out.push_str(&gap(&text[range.clone()]));
            out.push('\n');
        }
        out.push_str(&text[range.start..end]);
    }
    out
}

/// The line and the column where each list item of `text` starts.
fn item_starts(text: &str, options: &Options) -> HashSet<(usize, usize)> {
    let arena = Arena::new();
    comrak::parse_document(&arena, text, options)
        .descendants()
        .filter(|n| matches!(n.data().value, NodeValue::Item(_) | NodeValue::TaskItem(_)))
        .map(|n| {
            let start = n.data().sourcepos.start;
            (start.line, start.column)
        })
        .collect()
}

/// `text`, whose first line is file line `first_line`, without its
/// generated anchors, and the anchors found. An anchor is the generated
/// form as inline HTML: inside code it is text, and is left.
// sokf-implements: TLMD_107
// sokf-implements: MD-3_AC-14
#[expect(
    clippy::too_many_lines,
    reason = "one pass of the removal of the anchors, a split of which is a refactor of its own"
)]
pub(super) fn strip(text: &str, first_line: usize, options: &Options) -> Stripped {
    if !text.contains("<a id=\"") {
        return Stripped::unchanged(text);
    }
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, options);
    // Each line as comrak ends it, at a carriage return alone too, so a
    // line that comrak numbers reads the text that comrak read there.
    let lines: Vec<&str> = code::each_line(text).map(|l| &text[l]).collect();
    let defined = parse::Defined::of(root, &lines);
    let mut found: Vec<Found> = Vec::new();
    for node in root.descendants() {
        let NodeValue::HtmlInline(html) = &node.data().value else {
            continue;
        };
        let Some(id) = open_id(html) else {
            continue;
        };
        // The position of the anchor as written, past the definitions that
        // open its paragraph.
        let open = parse::written_pos(node, &lines, &defined);
        let Some(next) = node.next_sibling() else {
            continue;
        };
        if !matches!(&next.data().value, NodeValue::HtmlInline(h) if h == "</a>") {
            continue;
        }
        let close = parse::written_pos(next, &lines, &defined);
        // An anchor goes only where its text stands, so a position that
        // comrak gives wrong never cuts other text.
        let (line, start, end) = (
            open.start.line,
            open.start.column.saturating_sub(1),
            close.end.column,
        );
        let written = line
            .checked_sub(1)
            .and_then(|i| lines.get(i))
            .and_then(|l| l.get(start..end));
        if close.start.line == line
            && close.start.column == open.end.column + 1
            && written == Some(render(id).as_str())
        {
            found.push(Found {
                line,
                start,
                end,
                id: id.to_string(),
                opens: opens_item(node, line),
            });
        }
    }
    if found.is_empty() {
        return Stripped::unchanged(text);
    }
    found.sort();
    let mut anchors = Vec::new();
    let mut out = String::with_capacity(text.len());
    // Where `out` takes a stand-in, in order, and the first item of a list
    // that each opens: its body line, its start, and its first anchor.
    let mut stand_ins = Vec::new();
    let mut firsts: Vec<(usize, usize, usize)> = Vec::new();
    // The anchors are sorted by line, so each line takes the run of them
    // that starts where the last line's ended. A line ends where comrak
    // ends it, so it has the number that comrak gives it.
    let lines = code::lines(text);
    let mut next = 0;
    for (i, range) in lines.iter().enumerate() {
        let body_line = i + 1;
        // The line with its ending, up to the start of the next line.
        let end = lines.get(i + 1).map_or(text.len(), |l| l.start);
        let line = &text[range.start..end];
        let start = next;
        while found.get(next).is_some_and(|f| f.line == body_line) {
            next += 1;
        }
        let here = &found[start..next];
        if here.is_empty() {
            out.push_str(line);
            continue;
        }
        let mut clean = String::with_capacity(line.len());
        let mut at = 0;
        let first = anchors.len();
        for f in here {
            // Two nodes can't give one anchor.
            if f.start < at {
                continue;
            }
            clean.push_str(&line[at..f.start]);
            anchors.push(Anchor {
                id: f.id.clone(),
                line: first_line + i,
                col: clean.len(),
                raw_col: f.start,
                alone: false,
                joins_text: false,
            });
            at = f.end;
        }
        clean.push_str(&line[at..]);
        let alone = clean.trim().is_empty();
        // The anchors that open an item are all that its line holds, so
        // the item's marker would stand alone.
        let col = anchors[first].col;
        let opens = here[0]
            .opens
            .filter(|_| !alone && clean[col..].trim().is_empty());
        for a in &mut anchors[first..] {
            a.alone = alone;
        }
        if let Some(item) = opens {
            stand_ins.push(out.len() + col);
            if item.first {
                firsts.push((body_line, item.column, first));
            }
        }
        // A line of anchors alone is left blank, with its line ending, so
        // every line keeps its number.
        if alone {
            let text_end = clean.trim_end_matches(['\r', '\n']).len();
            out.push_str(&clean[text_end..]);
        } else {
            out.push_str(&clean);
        }
    }
    // Where the text without its stand-ins opens no item, the marker of the
    // first item of a list joins the block before it, so a repair that
    // removes the anchors writes a blank line before it. Each item is
    // judged with the blank lines before the items above it, as the repair
    // writes them.
    let mut gaps: Vec<usize> = Vec::new();
    for (line, column, first) in firsts {
        let read = with_gaps(&out, &gaps);
        if !item_starts(&read, options).contains(&(line + gaps.len(), column)) {
            gaps.push(line);
            for a in anchors[first..]
                .iter_mut()
                .take_while(|a| a.line == first_line + line - 1)
            {
                a.joins_text = true;
            }
        }
    }
    let parsed = (!stand_ins.is_empty()).then(|| {
        let mut parsed = String::with_capacity(out.len() + stand_ins.len());
        let mut from = 0;
        for at in stand_ins {
            parsed.push_str(&out[from..at]);
            parsed.push_str(STAND_IN);
            from = at;
        }
        parsed.push_str(&out[from..]);
        parsed
    });
    Stripped {
        text: out,
        parsed,
        anchors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> (String, Vec<Anchor>) {
        let mut options = Options::default();
        options.extension.table = true;
        let stripped = strip(text, 1, &options);
        (stripped.text, stripped.anchors)
    }

    // An item whose own line holds only anchors keeps a stand-in in the
    // text that comrak parses, and the first of a list straight after a
    // paragraph with text marks its anchors.
    // sokf-tests: TLMD_107
    #[test]
    fn an_item_of_anchors_alone_keeps_a_stand_in() {
        let mut options = Options::default();
        options.extension.tasklist = true;
        let text = "Text\n- <a id=\"a\"></a><a id=\"b\"></a> \n  - <a id=\"c\"></a>\n- [ ] <a id=\"d\"></a>\n- <a id=\"e\"></a>E\n> - <a id=\"f\"></a>\n";
        let stripped = strip(text, 1, &options);
        assert_eq!(stripped.text, "Text\n-  \n  - \n- [ ] \n- E\n> - \n");
        assert_eq!(
            stripped.parsed.as_deref(),
            Some("Text\n- x \n  - x\n- [ ] x\n- E\n> - x\n")
        );
        let got: Vec<(&str, bool)> = stripped
            .anchors
            .iter()
            .map(|a| (a.id.as_str(), a.joins_text))
            .collect();
        assert_eq!(
            got,
            vec![
                ("a", true),
                ("b", true),
                ("c", false),
                ("d", false),
                ("e", false),
                ("f", false)
            ]
        );
        let stripped = strip("Text\n- <a id=\"a\"></a>A\n", 1, &options);
        assert_eq!(stripped.parsed, None);
        // A heading that the item holds, and a paragraph of anchors alone
        // before the list.
        let stripped = strip(
            "Text\n- <a id=\"a\"></a>\n  ===\n\n> <a id=\"b\"></a>\n> - <a id=\"c\"></a>\n",
            1,
            &options,
        );
        assert_eq!(
            stripped.parsed.as_deref(),
            Some("Text\n- x\n  ===\n\n> \n> - x\n")
        );
        let got: Vec<(&str, bool)> = stripped
            .anchors
            .iter()
            .map(|a| (a.id.as_str(), a.joins_text))
            .collect();
        assert_eq!(got, vec![("a", true), ("b", false), ("c", false)]);
        // A line of anchors alone after a definition takes no stand-in, and
        // a quote that the item opens keeps it apart from the paragraph.
        let stripped = strip("- [a]: /x\n  <a id=\"a\"></a>\n", 1, &options);
        assert_eq!(stripped.parsed, None);
        let stripped = strip("Text\n- > <a id=\"a\"></a>\n", 1, &options);
        assert_eq!(stripped.parsed.as_deref(), Some("Text\n- > x\n"));
        assert!(!stripped.anchors[0].joins_text);
    }

    // sokf-tests: TLMD_107
    #[test]
    fn anchors_are_removed_and_recorded() {
        let text = "## <a id=\"k\"></a>4.1 Head\n\n- <a id=\"user\"></a>`User` - A person.\n\n<a id=\"a\"></a><a id=\"b\"></a>\n\n```text\n<a id=\"c\"></a>\n```\n\nSee `<a id=\"d\"></a>` and <a id=\"e\">x</a>.\n";
        let (clean, anchors) = run(text);
        assert_eq!(
            clean,
            "## 4.1 Head\n\n- `User` - A person.\n\n\n\n```text\n<a id=\"c\"></a>\n```\n\nSee `<a id=\"d\"></a>` and <a id=\"e\">x</a>.\n"
        );
        let got: Vec<(&str, usize, usize, usize, bool)> = anchors
            .iter()
            .map(|a| (a.id.as_str(), a.line, a.col, a.raw_col, a.alone))
            .collect();
        assert_eq!(
            got,
            vec![
                ("k", 1, 3, 3, false),
                ("user", 3, 2, 2, false),
                ("a", 5, 0, 0, true),
                ("b", 5, 0, 14, true),
            ]
        );
    }

    // sokf-tests: TLMD_107
    // sokf-tests: MD-3_AC-14
    #[test]
    fn an_anchor_after_a_carriage_return_alone_is_removed_from_its_line() {
        let (clean, anchors) = run("a\rb\nc\n<a id=\"k\"></a>x\nyy\n");
        assert_eq!(clean, "a\rb\nc\nx\nyy\n");
        let got: Vec<(&str, usize, usize, bool)> = anchors
            .iter()
            .map(|a| (a.id.as_str(), a.line, a.raw_col, a.alone))
            .collect();
        assert_eq!(got, vec![("k", 4, 0, false)]);
        let (clean, anchors) = run("<a id=\"j\"></a>\r<a id=\"k\"></a>x\r\n");
        assert_eq!(clean, "\rx\r\n");
        let got: Vec<(&str, usize, bool)> = anchors
            .iter()
            .map(|a| (a.id.as_str(), a.line, a.alone))
            .collect();
        assert_eq!(got, vec![("j", 1, true), ("k", 2, false)]);
    }

    // An anchor past an escaped pipe, in a table cell and in a paragraph
    // that directly precedes a table, is removed whole.
    // sokf-tests: TLMD_107
    #[test]
    fn an_anchor_past_an_escaped_pipe_is_removed_whole() {
        for (text, clean) in [
            (
                "a \\| \\| <a id=\"k\"></a>`x`\n| A |\n|---|\n",
                "a \\| \\| `x`\n| A |\n|---|\n",
            ),
            (
                "| A |\n|---|\n| a \\| <a id=\"k\"></a>`x` |\n",
                "| A |\n|---|\n| a \\| `x` |\n",
            ),
        ] {
            let (got, anchors) = run(text);
            assert_eq!(got, clean, "{text}");
            assert_eq!(anchors.len(), 1, "{text}");
            let a = &anchors[0];
            let line = text.lines().nth(a.line - 1).unwrap();
            assert!(line[a.raw_col..].starts_with(&render("k")), "{text}");
        }
    }

    // An anchor in a row indented apart from its table's first line is
    // removed whole.
    // sokf-tests: TLMD_107
    #[test]
    fn an_anchor_in_an_indented_row_is_removed_whole() {
        for text in [
            "| A |\n|---|\n | <a id=\"k\"></a>`x` |\n",
            "- item\n\n    | A |\n    |---|\n  | a \\| <a id=\"k\"></a>`x` |\n",
        ] {
            let (got, anchors) = run(text);
            assert_eq!(got, text.replace(&render("k"), ""), "{text}");
            assert_eq!(anchors.len(), 1, "{text}");
            let a = &anchors[0];
            let line = text.lines().nth(a.line - 1).unwrap();
            assert!(line[a.raw_col..].starts_with(&render("k")), "{text}");
        }
    }

    // The anchors of a row indented more or less than its table, or after
    // an escaped pipe, are found where they are written, and no other text
    // goes.
    // sokf-tests: TLMD_107
    #[test]
    fn the_anchors_of_a_shifted_row_are_found_where_written() {
        let (clean, anchors) =
            run("| A | B |\n| --- | --- |\n  x <a id=\"y\"></a> | <a id=\"z\"></a>2 |\n");
        assert_eq!(clean, "| A | B |\n| --- | --- |\n  x  | 2 |\n");
        let got: Vec<(&str, usize, usize)> = anchors
            .iter()
            .map(|a| (a.id.as_str(), a.col, a.raw_col))
            .collect();
        assert_eq!(got, vec![("y", 4, 4), ("z", 7, 21)]);
        let (clean, anchors) = run("   A | B\n   --- | ---\n x <a id=\"y\"></a> | 2\n");
        assert_eq!(clean, "   A | B\n   --- | ---\n x  | 2\n");
        assert_eq!(anchors[0].raw_col, 3);
        let (clean, anchors) =
            run("| A | B |\n| --- | --- |\n| <a id=\"y\"></a>x | a \\| b <a id=\"z\"></a> |\n");
        assert_eq!(clean, "| A | B |\n| --- | --- |\n| x | a \\| b  |\n");
        let got: Vec<(&str, usize)> = anchors.iter().map(|a| (a.id.as_str(), a.raw_col)).collect();
        assert_eq!(got, vec![("y", 2), ("z", 27)]);
        let (clean, anchors) = run("| A | B |\n| --- | --- |\n| 1 | 2 |\n  <a id=\"y\"></a>\n");
        assert_eq!(clean, "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n");
        assert!(anchors[0].alone);
    }

    #[test]
    fn text_without_anchors_is_untouched() {
        let (clean, anchors) = run("Plain <a href=\"x\">link</a>.\n");
        assert_eq!(clean, "Plain <a href=\"x\">link</a>.\n");
        assert!(anchors.is_empty(), "{anchors:?}");
        assert!(open_id("<a id=\"x y\">").is_none());
        assert_eq!(open_id("<a id=\"cli-1_ac-2\">"), Some("cli-1_ac-2"));
    }
}
