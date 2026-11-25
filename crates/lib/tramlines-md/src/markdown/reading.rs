//! The reading of a text (`TLMD_258` to `TLMD_260`): the text with each
//! piece of its markup in one fixed spelling, the one that oxfmt,
//! Prettier, and dprint write outside a word, and each byte mapped to the
//! text as written; and the plain text, from which a slug is formed. An
//! emphasis inside a word, which a formatter writes with `*`, reads `_x_`
//! too, so `foo*bar*baz` reads as `foo_bar_baz` does.

use std::ops::Range;

use comrak::{Arena, nodes::NodeValue};
use serde::{Deserialize, Serialize};

use super::parse;

/// What a text is read behind: a private-use character, which is neither
/// a space nor punctuation, and a space. So no text starts a block, as
/// `1. two` or `# x` alone would, and the first character of the text
/// keeps the flanking that it has after a space.
const LEAD: &str = "\u{E000} ";

// sokf:begin reading
/// A text in the fixed spelling of its markup: each emphasis as `_x_`,
/// each strong emphasis as `**x**`, each code span in the shortest run of
/// backticks that its content allows, and each escape that changes nothing
/// as the plain character, every other character as written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading {
    /// The text in the fixed spelling.
    pub text: String,
    /// `map[i]` is the offset in the text as written of byte `i` of
    /// `text`, in order, and `map[text.len()]` the length of the text as
    /// written.
    pub map: Vec<usize>,
}
// sokf:end reading

impl Reading {
    /// The reading of a text that reads as written, such as a line of a
    /// fence: the text itself, each byte mapped to itself.
    pub fn written(text: &str) -> Reading {
        Reading {
            text: text.to_string(),
            map: (0..=text.len()).collect(),
        }
    }

    /// The reading, or `text` as written where the reading is empty, as a
    /// `Reading` left at its default by a host, or by the data of an older
    /// version, is.
    pub fn or_written<'a>(&'a self, text: &str) -> std::borrow::Cow<'a, Reading> {
        if self.map.is_empty() {
            std::borrow::Cow::Owned(Reading::written(text))
        } else {
            std::borrow::Cow::Borrowed(self)
        }
    }

    /// The offset in the text as written of offset `i` of the reading.
    pub fn original(&self, i: usize) -> usize {
        self.map
            .get(i)
            .or(self.map.last())
            .copied()
            .unwrap_or_default()
    }

    /// Whether byte `at` of `text`, the text as written, is a backslash that
    /// the reading drops, which belongs to the character after it.
    pub(crate) fn drops_backslash(&self, text: &str, at: usize) -> bool {
        text.as_bytes().get(at) == Some(&b'\\') && self.map.binary_search(&at).is_err()
    }

    /// The bytes of `text`, the text as written, that a part of the reading
    /// stands for, from `start` to `end`, the offsets of `text` that the
    /// part's first byte and the byte after it map to. A backslash that the
    /// reading drops belongs to the character after it, and a backtick that
    /// the reading of a code span drops to the code span, as the map gives
    /// it.
    pub(crate) fn span_of(&self, text: &str, start: usize, end: usize) -> Range<usize> {
        let start = if start > 0 && self.drops_backslash(text, start - 1) {
            start - 1
        } else {
            start
        };
        let end = if end > start && self.drops_backslash(text, end - 1) {
            end - 1
        } else {
            end
        };
        start..end.max(start)
    }
}

/// One change that a reading makes to a text: the bytes `range` of the
/// text as written, and what the reading writes in their place, which is
/// no longer than the range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Change {
    /// The bytes of the text as written.
    pub range: Range<usize>,
    /// What the reading writes in their place.
    pub with: String,
}

// sokf:begin reading-fns
/// The reading of `text`, read as a paragraph alone (`TLMD_259`).
// sokf-implements: TLMD_258
// sokf-implements: TLMD_259
// sokf-implements: MD-14_AC-1
pub fn reading(text: &str) -> Reading {
    apply(text, &changes(text))
}

/// The plain text of `text`, read as a paragraph alone: the text without
/// the markers of emphasis, code, links, and inline HTML, from which
/// `slug` forms a slug.
// sokf-implements: MD-14_AC-2
pub fn plain_text(text: &str) -> String {
    // sokf:end reading-fns
    if !text.bytes().any(|b| b"*_`\\[]<>&!".contains(&b)) {
        return text.to_string();
    }
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &lead(text), &parse::options());
    let collected = root.collect_text();
    let plain = collected.strip_prefix('\u{E000}').unwrap_or(&collected);
    plain.strip_prefix(' ').unwrap_or(plain).to_string()
}

/// `text` behind the lead, each line break read as a space, as the text of
/// a unit joins its lines. Every byte of `text` keeps its offset after the
/// lead.
fn lead(text: &str) -> String {
    format!("{LEAD}{}", text.replace(['\n', '\r'], " "))
}

/// The changes that the reading of `text` makes, in order: the respelling
/// of each emphasis, strong emphasis, and code span, and the drop of each
/// backslash that changes nothing, each kept only where the text with it,
/// and with every change kept before it, renders the same HTML as the text
/// as written (`TLMD_258`, `TLMD_260`). An emphasis inside a word is the
/// one exception, which reads `_x_` though a `_` there opens no emphasis:
/// the other changes are judged beside it as written, and its respelling
/// is added after them.
// sokf-implements: TLMD_260
// sokf-implements: TLMD_266
pub(crate) fn changes(text: &str) -> Vec<Change> {
    if !text
        .bytes()
        .any(|b| matches!(b, b'*' | b'_' | b'`' | b'\\'))
    {
        return Vec::new();
    }
    let source = lead(text);
    let (in_word, respellings, escapes) = proposed(text, &source);
    if respellings.is_empty() && escapes.is_empty() {
        return in_word;
    }
    let options = parse::options();
    let html = |changes: &[Change]| {
        let mut sorted = changes.to_vec();
        sorted.sort_by_key(|c| c.range.start);
        comrak::markdown_to_html(&lead(&apply(text, &sorted).text), &options)
    };
    let meaning = comrak::markdown_to_html(&source, &options);
    let mut kept: Vec<Change> = Vec::new();
    // Most texts keep every respelling, which one rendering shows; a text
    // that doesn't keeps each one that keeps its meaning alone, in order.
    let all: Vec<Change> = respellings.iter().flatten().cloned().collect();
    if !respellings.is_empty() && html(&all) == meaning {
        kept = all;
    } else {
        for node in respellings {
            let with: Vec<Change> = kept.iter().chain(&node).cloned().collect();
            if html(&with) == meaning {
                kept = with;
            }
        }
    }
    for at in escapes {
        let mut with = kept.clone();
        with.push(Change {
            range: at..at + 1,
            with: String::new(),
        });
        if html(&with) == meaning {
            kept = with;
        }
    }
    // An emphasis inside a word has one spelling, `*`, so the changes
    // judged beside it as written are judged alike in every twin.
    kept.extend(in_word);
    kept.sort_by_key(|c| c.range.start);
    kept
}

/// The changes that the reading of `text`, behind the lead in `source`,
/// proposes: the respelling of each emphasis inside a word, the respelling
/// of each other emphasis, strong emphasis, and code span, one list for
/// each node, and the offset of the backslash of each escape that may
/// change nothing, in order.
fn proposed(text: &str, source: &str) -> (Vec<Change>, Vec<Vec<Change>>, Vec<usize>) {
    // The text as comrak reads it behind the lead, each line break a space,
    // at the same offsets, which the changes read.
    let flat = &source[LEAD.len()..];
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, source, &parse::options());
    let mut in_word = Vec::new();
    let mut respellings = Vec::new();
    let mut escapes = Vec::new();
    for node in root.descendants() {
        let ast = node.data();
        let sp = ast.sourcepos;
        // The source is one line, so a node of the text lies on it, after
        // the lead.
        if sp.start.line != 1 || sp.end.line != 1 || sp.start.column <= LEAD.len() {
            continue;
        }
        let span = sp.start.column - 1 - LEAD.len()..sp.end.column.saturating_sub(LEAD.len());
        if span.end > text.len() || span.start >= span.end {
            continue;
        }
        let mut node = Vec::new();
        match &ast.value {
            NodeValue::Emph(_) => delimiters(flat, &span, 1, "_", &mut node),
            NodeValue::Strong(_) => delimiters(flat, &span, 2, "**", &mut node),
            NodeValue::Code(_) => code(flat, &span, &mut node),
            // A backslash before a pipe stays, because a table cell needs it,
            // and the cell's text holds it as written.
            NodeValue::Escaped
                if text.as_bytes()[span.start] == b'\\'
                    && text.as_bytes().get(span.start + 1) != Some(&b'|') =>
            {
                escapes.push(span.start);
            }
            _ => {}
        }
        if node.is_empty() {
            continue;
        }
        if matches!(ast.value, NodeValue::Emph(_)) && inside_word(flat, &span) {
            in_word.extend(node);
        } else {
            respellings.push(node);
        }
    }
    // A node inside another comes after it in the walk, so its changes lie
    // between the outer node's.
    in_word.sort_by_key(|c: &Change| c.range.start);
    escapes.sort_unstable();
    (in_word, respellings, escapes)
}

/// Whether the emphasis at `span` of `text` stands inside a word: a letter,
/// a digit, or a private-use character touches it from outside, where only
/// `*` opens or closes an emphasis, so that its reading as `_x_` reads as
/// no emphasis (`TLMD_258`). The placeholder of a hole of a template is a
/// private-use character, so an emphasis beside a hole reads as one inside
/// a word does.
fn inside_word(text: &str, span: &Range<usize>) -> bool {
    // A letter, a digit, or a private-use character, such as the
    // placeholder; every other character is taken for punctuation, so an
    // emphasis that touches only such characters is judged by its HTML.
    let word = |c: char| c.is_alphanumeric() || ('\u{E000}'..='\u{F8FF}').contains(&c);
    let before = text[..span.start].chars().next_back();
    let after = text[span.end..].chars().next();
    before.is_some_and(word) || after.is_some_and(word)
}

/// The delimiters, `n` bytes at each end of `span`, of an emphasis or a
/// strong emphasis, written as `with`.
fn delimiters(text: &str, span: &Range<usize>, n: usize, with: &str, out: &mut Vec<Change>) {
    let bytes = text.as_bytes();
    let delimiter = |r: Range<usize>| bytes[r].iter().all(|b| matches!(b, b'*' | b'_'));
    if span.len() < 2 * n
        || !delimiter(span.start..span.start + n)
        || !delimiter(span.end - n..span.end)
    {
        return;
    }
    for range in [span.start..span.start + n, span.end - n..span.end] {
        if &text[range.clone()] != with {
            out.push(Change {
                range,
                with: with.to_string(),
            });
        }
    }
}

/// The run of backticks and the padding of the code span at `span`,
/// written again: its content as the text holds it, without the one space
/// that CommonMark strips at each end, in the shortest run of backticks
/// that the content allows, with one space inside each end only where the
/// content starts or ends with a backtick, or starts and ends with a space
/// and holds another character (`TLMD_258`).
fn code(text: &str, span: &Range<usize>, out: &mut Vec<Change>) {
    let written = &text[span.clone()];
    let n = written.bytes().take_while(|&b| b == b'`').count();
    if n == 0
        || written.len() < 2 * n + 1
        || !written[written.len() - n..].bytes().all(|b| b == b'`')
    {
        return;
    }
    let content = &written[n..written.len() - n];
    let stripped = content.len() >= 2
        && content.starts_with(' ')
        && content.ends_with(' ')
        && content.bytes().any(|b| b != b' ');
    let inner = if stripped {
        &content[1..content.len() - 1]
    } else {
        content
    };
    let (m, pad) = crate::text::code_fence(inner);
    let ticks = "`".repeat(m);
    let space = if pad { " " } else { "" };
    let strip = usize::from(stripped);
    let open = span.start..span.start + n + strip;
    let close = span.end - n - strip..span.end;
    for (range, with) in [
        (open, format!("{ticks}{space}")),
        (close, format!("{space}{ticks}")),
    ] {
        if text[range.clone()] != with {
            out.push(Change { range, with });
        }
    }
}

/// `text` with `changes`, in order and apart, applied, each byte of the
/// result mapped to the byte of `text` that it stands for.
pub(crate) fn apply(text: &str, changes: &[Change]) -> Reading {
    let mut out = String::with_capacity(text.len());
    let mut map = Vec::with_capacity(text.len() + 1);
    let mut at = 0;
    for c in changes {
        if c.range.start < at || c.range.end > text.len() {
            continue;
        }
        out.push_str(&text[at..c.range.start]);
        map.extend(at..c.range.start);
        out.push_str(&c.with);
        let last = c.range.end.saturating_sub(1).max(c.range.start);
        map.extend((0..c.with.len()).map(|k| (c.range.start + k).min(last)));
        at = c.range.end;
    }
    out.push_str(&text[at..]);
    map.extend(at..=text.len());
    Reading { text: out, map }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// The reading of `text`, as a string.
    fn read(text: &str) -> String {
        reading(text).text
    }

    // sokf-tests: TLMD_258
    #[test]
    fn emphasis_reads_in_the_fixed_spelling() {
        assert_eq!(read("*x* and _x_"), "_x_ and _x_");
        assert_eq!(read("__x__ and **x**"), "**x** and **x**");
        assert_eq!(read("foo*bar*baz"), "foo_bar_baz");
        assert_eq!(read("***x*** and **_y_**"), "_**x**_ and **_y_**");
        assert_eq!(read("a * b and 2*3"), "a * b and 2*3");
        assert_eq!(read("snake_case_name"), "snake_case_name");
        assert_eq!(read("Use *the* tool"), "Use _the_ tool");
        assert_eq!(read("Ünï *x* é"), "Ünï _x_ é");
        // An emphasis inside a word inside another reads alike whatever else
        // the text holds, and judges no strong emphasis beside it.
        assert_eq!(read("a*b *c*d*"), "a_b _c_d_");
        assert_eq!(read("a*b *c*d* snake\\_case"), "a_b _c_d_ snake_case");
        assert_eq!(read("a*x*__y__"), read("a*x***y**"));
        // Beside punctuation that isn't ASCII, an emphasis is judged by its
        // HTML.
        assert_eq!(read("“*_init_*”"), "“*_init_*”");
        assert_eq!(read("“*x*” and —*y*—"), "“_x_” and —_y_—");
    }

    // sokf-tests: TLMD_258
    // sokf-tests: TLMD_261
    #[test]
    fn a_code_span_reads_in_its_shortest_backticks() {
        assert_eq!(read("``#user``"), "`#user`");
        assert_eq!(read("` | `"), "`|`");
        assert_eq!(read("`` ` ``"), "`` ` ``");
        assert_eq!(read("`` `x` ``"), "`` `x` ``");
        assert_eq!(read("`  two  `"), "`  two  `");
        assert_eq!(read("``x`y``"), "``x`y``");
        assert_eq!(read("`*x*` and `snake\\_case`"), "`*x*` and `snake\\_case`");
        assert_eq!(read("`  `"), "`  `");
        // A line break inside a code span reads as a space, as a unit joins
        // its lines.
        assert_eq!(read("``\nfoo\n``"), "`foo`");
    }

    // sokf-tests: TLMD_260
    #[test]
    fn an_escape_reads_as_the_character_where_it_changes_nothing() {
        assert_eq!(read("snake\\_case"), "snake_case");
        assert_eq!(read("a \\* b"), "a * b");
        assert_eq!(read("\\*x\\*"), "*x\\*");
        assert_eq!(read("*x\\*"), "*x\\*");
        assert_eq!(read("\\[x]"), "\\[x]");
        assert_eq!(read("1\\. two"), "1. two");
        assert_eq!(read("a \\\\ b"), "a \\ b");
        assert_eq!(read("a\\|b"), "a\\|b");
    }

    // sokf-tests: TLMD_258
    #[test]
    fn a_link_reads_its_text_and_keeps_its_destination() {
        assert_eq!(read("[*the* guide](u_v)"), "[_the_ guide](u_v)");
        assert_eq!(read("[*a*](u) and ![*alt*](i)"), "[_a_](u) and ![_alt_](i)");
        assert_eq!(read("<b>*h*</b>"), "<b>_h_</b>");
        assert_eq!(read("[x](<a b> 'T')"), "[x](<a b> 'T')");
    }

    #[test]
    fn a_reading_maps_each_byte_to_the_text() {
        let r = reading("a \\_ ``b``");
        assert_eq!(r.text, "a _ `b`");
        assert_eq!(r.map, vec![0, 1, 3, 4, 5, 7, 8, 10]);
        assert_eq!(reading("plain").map, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn the_plain_text_drops_the_markers() {
        assert_eq!(plain_text("Title *x*"), "Title x");
        assert_eq!(plain_text("See [the guide](x)"), "See the guide");
        assert_eq!(plain_text("What's `this`? (v2)"), "What's this? (v2)");
        assert_eq!(plain_text("_Short_"), "Short");
        assert_eq!(plain_text("1. Intro"), "1. Intro");
        assert_eq!(plain_text("CLI-1_AC-2"), "CLI-1_AC-2");
        assert_eq!(plain_text(""), "");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A reading is never longer than its text, and maps in order
        /// into it.
        #[test]
        fn a_reading_maps_in_order(text in "[ a-c*_`\\\\\\[\\]]{0,16}") {
            let r = reading(&text);
            prop_assert!(r.text.len() <= text.len());
            prop_assert_eq!(r.map.len(), r.text.len() + 1);
            prop_assert!(r.map.windows(2).all(|w| w[0] < w[1] || w[1] == text.len()));
            prop_assert_eq!(*r.map.last().unwrap(), text.len());
        }
    }
}
