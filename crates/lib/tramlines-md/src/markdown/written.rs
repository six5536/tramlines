//! The text that a repair or a scaffold writes, so that the parser reads it
//! back as what it means: the text of a heading, and the text of a unit
//! bound to a value.

use std::borrow::Cow;

use comrak::{Arena, nodes::NodeValue};

use super::{parse, reading};

/// `text`, the text of an ATX heading, as the line writes it after its
/// hashes, so that the parser reads it back as `text`: a run of `#` that
/// ends the text, after a space or a tab or alone, would be the heading's
/// closing sequence, so its first `#` is escaped (`TLMD_274`).
// sokf-implements: TLMD_274
pub(crate) fn atx_written(text: &str) -> Cow<'_, str> {
    let run = text.trim_end_matches('#').len();
    if run < text.len() && (run == 0 || text[..run].ends_with([' ', '\t'])) {
        Cow::Owned(format!("{}\\{}", &text[..run], &text[run..]))
    } else {
        Cow::Borrowed(text)
    }
}

/// The text that a heading bound to `value` holds: the value without the
/// whitespace at its ends, and each run of whitespace that holds a line
/// break as one space, because a heading holds neither, so the heading
/// that a repair writes reads as that text (`TLMD_282`, `TLMD_283`).
// sokf-implements: TLMD_282
// sokf-implements: TLMD_283
pub(crate) fn heading_text_of(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for (i, line) in value.trim().split(['\n', '\r']).enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if i > 0 && !out.is_empty() {
            out.push(' ');
        }
        out.push_str(line);
    }
    out
}

/// Why no text of a unit's kind reads as a text (`TLMD_276`, `TLMD_277`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unwritable {
    /// The text is empty, and a unit holds some text.
    Empty,
    /// A line that holds the text starts the block that the field names,
    /// and no escape keeps its reading.
    Starts(&'static str),
}

impl Unwritable {
    /// The finding on `subject`, such as ``the frontmatter `description` ``,
    /// whose text is `text`, which no unit that `noun` names can hold.
    pub(crate) fn said(self, subject: &str, text: &str, noun: &str) -> String {
        match self {
            Unwritable::Empty => format!("{subject} is empty, which no {noun} can hold"),
            Unwritable::Starts(block) => format!(
                "{subject} holds {}, which no {noun} can hold: on a line of its own it starts {block}",
                crate::text::code(text)
            ),
        }
    }
}

/// `text`, the text of a paragraph or of a list item's first line, as a
/// repair or a scaffold writes it after `prefix`, the marker of the item,
/// the quote, or the indent that stands before it, so that the parser
/// reads it back as a unit of its kind whose reading is the reading of
/// `text`: a text that would start another block there, such as `# x`,
/// `- x`, `> x`, `1. x`, a fence, a thematic break, or a link reference
/// definition, or that would make a thematic break of its item's marker,
/// as `--` does after `- `, has a backslash written before the first
/// punctuation character at which the escape gives the unit the blocks of
/// a unit of its kind and leaves its reading (`TLMD_275`). An empty text,
/// and a text that no such escape settles, such as one that starts an
/// HTML block, is `Unwritable`.
// sokf-implements: TLMD_275
pub(crate) fn unit_written<'a>(prefix: &str, text: &'a str) -> Result<Cow<'a, str>, Unwritable> {
    // The indent of a nested unit would make its probe a code block, and the
    // marker after it is what can join the text.
    let prefix = prefix.trim_start();
    if text.is_empty() {
        // An item can be empty, and a paragraph can't.
        let item = prefix
            .trim_end()
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .ends_with(['-', '*', '+', '.', ')']);
        return if item {
            Ok(Cow::Borrowed(text))
        } else {
            Err(Unwritable::Empty)
        };
    }
    let kind = blocks(prefix, "x");
    let found = blocks(prefix, text);
    if kinds(&found) == kinds(&kind) {
        return Ok(Cow::Borrowed(text));
    }
    escapes(text)
        .find(|escaped| kinds(&blocks(prefix, escaped)) == kinds(&kind))
        .map(Cow::Owned)
        .ok_or_else(|| {
            let at = found.iter().zip(&kind).position(|(f, k)| f.0 != k.0);
            // A text that makes fewer blocks is a definition, which makes none.
            let block = match at {
                Some(i) => found.get(i).map_or("another block", |b| b.1),
                None if found.len() < kind.len() => "a link reference definition",
                None => found.last().map_or("another block", |b| b.1),
            };
            Unwritable::Starts(block)
        })
}

/// `text` with a backslash before one of its first punctuation characters,
/// or before the `:` after the label of a link reference definition, in
/// order, each whose reading is the reading of `text`: the escapes that can
/// keep a text from starting a block or from joining the line above it.
pub(crate) fn escapes(text: &str) -> impl Iterator<Item = String> + '_ {
    let meant = reading(text).text;
    // A block starts within the first few punctuation characters of its
    // line, or at the `:` after the label of a link reference definition, so
    // the search stops there.
    let colon = text.find("]:").map(|i| i + 1);
    text.char_indices()
        .filter(|(_, c)| c.is_ascii_punctuation() && *c != '\\')
        .map(|(i, _)| i)
        .take(CANDIDATES)
        .chain(colon)
        .map(|i| format!("{}\\{}", &text[..i], &text[i..]))
        .filter(move |escaped| reading(escaped).text == meant)
}

/// How many punctuation characters at the start of a text the escape is
/// tried before: a block starts at its line's first characters, the
/// delimiter after the digits of an ordered marker, or the `:` after the
/// label of a link reference definition.
const CANDIDATES: usize = 16;

/// A block of a parse: its kind, and how a finding names it.
type Kind = (std::mem::Discriminant<NodeValue>, &'static str);

/// The kinds of `blocks`.
fn kinds(blocks: &[Kind]) -> Vec<std::mem::Discriminant<NodeValue>> {
    blocks.iter().map(|b| b.0).collect()
}

/// The blocks that `prefix` and `text` make, on a line of their own after
/// a paragraph and a blank line, in the order of the parse; a task item
/// counts as an item.
fn blocks(prefix: &str, text: &str) -> Vec<Kind> {
    let arena = Arena::new();
    let source = format!("x\n\n{prefix}{text}\n");
    let root = comrak::parse_document(&arena, &source, &parse::options());
    let item = std::mem::discriminant(&NodeValue::Item(comrak::nodes::NodeList::default()));
    root.descendants()
        .skip(2)
        .filter(|n| n.data().value.block())
        .map(|n| {
            let value = &n.data().value;
            let name = match value {
                NodeValue::HtmlBlock(_) => "an HTML block",
                NodeValue::Heading(_) => "a heading",
                NodeValue::List(_) | NodeValue::Item(_) | NodeValue::TaskItem(_) => "a list",
                NodeValue::BlockQuote => "a quote",
                NodeValue::CodeBlock(_) => "a code block",
                NodeValue::ThematicBreak(_) => "a thematic break",
                NodeValue::Table(_) => "a table",
                _ => "another block",
            };
            match value {
                NodeValue::TaskItem(_) => (item, name),
                v => (std::mem::discriminant(v), name),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_that_starts_a_block_is_escaped_to_a_paragraph() {
        for (text, written) in [
            ("# x", "\\# x"),
            ("1)", "1\\)"),
            ("1. x", "1\\. x"),
            ("[a]: /x", "[a]\\: /x"),
            ("- x", "\\- x"),
            ("a # b", "a # b"),
        ] {
            assert_eq!(unit_written("", text).unwrap(), written, "{text:?}");
        }
        for (prefix, text, written) in [
            ("- ", "--", "\\--"),
            ("* ", "**", "\\**"),
            ("- ", "**", "**"),
            ("- ", "[ ] x", "[ ] x"),
        ] {
            assert_eq!(unit_written(prefix, text).unwrap(), written, "{text:?}");
        }
    }

    #[test]
    fn an_empty_text_and_an_html_block_are_unwritable() {
        assert_eq!(unit_written("", ""), Err(Unwritable::Empty));
        assert_eq!(
            unit_written("", "<div>"),
            Err(Unwritable::Starts("an HTML block"))
        );
        assert_eq!(
            unit_written("- ", "<div>"),
            Err(Unwritable::Starts("an HTML block"))
        );
    }
}
