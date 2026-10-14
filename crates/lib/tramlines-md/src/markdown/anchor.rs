//! The generated anchors (TLMD_104 to TLMD_109): `<a id="<slug>"></a>` before each key.
//! They are found by form, outside code, and removed from the text before
//! it is parsed, so no rule ever reads one.

use comrak::{Arena, Options, nodes::NodeValue};
use serde::{Deserialize, Serialize};

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

/// `text`, whose first line is file line `first_line`, without its
/// generated anchors, and the anchors found. An anchor is the generated
/// form as inline HTML: inside code it is text, and is left.
// sokf-implements: TLMD_107
pub(super) fn strip(text: &str, first_line: usize, options: &Options) -> (String, Vec<Anchor>) {
    if !text.contains("<a id=\"") {
        return (text.to_string(), Vec::new());
    }
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, options);
    // (body line, start byte, end byte, id)
    let mut found: Vec<(usize, usize, usize, String)> = Vec::new();
    for node in root.descendants() {
        let ast = node.data();
        let NodeValue::HtmlInline(html) = &ast.value else {
            continue;
        };
        let Some(id) = open_id(html) else {
            continue;
        };
        let open = ast.sourcepos;
        let Some(next) = node.next_sibling() else {
            continue;
        };
        let close = next.data();
        let closes = matches!(&close.value, NodeValue::HtmlInline(h) if h == "</a>")
            && close.sourcepos.start.line == open.start.line
            && close.sourcepos.start.column == open.end.column + 1;
        if closes {
            found.push((
                open.start.line,
                open.start.column - 1,
                close.sourcepos.end.column,
                id.to_string(),
            ));
        }
    }
    if found.is_empty() {
        return (text.to_string(), Vec::new());
    }
    found.sort();
    let mut anchors = Vec::new();
    let mut out = String::with_capacity(text.len());
    // The anchors are sorted by line, so each line takes the run of them
    // that starts where the last line's ended.
    let mut next = 0;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let body_line = i + 1;
        let start = next;
        while found.get(next).is_some_and(|f| f.0 == body_line) {
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
        for (_, start, end, id) in here {
            clean.push_str(&line[at..*start]);
            anchors.push(Anchor {
                id: id.clone(),
                line: first_line + i,
                col: clean.len(),
                raw_col: *start,
                alone: false,
            });
            at = *end;
        }
        clean.push_str(&line[at..]);
        let alone = clean.trim().is_empty();
        for a in &mut anchors[first..] {
            a.alone = alone;
        }
        if alone {
            // The line is left blank, so every line keeps its number.
            if line.ends_with('\n') {
                out.push('\n');
            }
        } else {
            out.push_str(&clean);
        }
    }
    (out, anchors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> (String, Vec<Anchor>) {
        let mut options = Options::default();
        options.extension.table = true;
        strip(text, 1, &options)
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

    #[test]
    fn text_without_anchors_is_untouched() {
        let (clean, anchors) = run("Plain <a href=\"x\">link</a>.\n");
        assert_eq!(clean, "Plain <a href=\"x\">link</a>.\n");
        assert!(anchors.is_empty());
        assert!(open_id("<a id=\"x y\">").is_none());
        assert_eq!(open_id("<a id=\"cli-1_ac-2\">"), Some("cli-1_ac-2"));
    }
}
