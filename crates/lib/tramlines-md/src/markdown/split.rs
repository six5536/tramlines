//! A document's frontmatter, split from its body.

/// Frontmatter and body of a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split<'a> {
    /// The YAML text between the `---` lines and the line of its first line.
    pub frontmatter: Option<(&'a str, usize)>,
    /// Everything after the frontmatter.
    pub body: &'a str,
    /// The line of the body's first line.
    pub body_line: usize,
}

/// Split frontmatter from body. Frontmatter opens with `---` on the first
/// line and closes with the next `---` line. An unclosed block runs to the
/// end of the text, with an empty body.
// sokf-implements: TLMD_1
// sokf-implements: TLMD_2
pub fn split_frontmatter(text: &str) -> Split<'_> {
    let opens = text == "---" || text.starts_with("---\n");
    if !opens {
        return Split {
            frontmatter: None,
            body: text,
            body_line: 1,
        };
    }
    let after_open = if text == "---" { "" } else { &text[4..] };
    let mut offset = 0;
    let mut line = 2;
    for raw_line in after_open.split_inclusive('\n') {
        let content = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        if content == "---" {
            let yaml = &after_open[..offset];
            let body = &after_open[offset + raw_line.len()..];
            return Split {
                frontmatter: Some((yaml, 2)),
                body,
                body_line: line + 1,
            };
        }
        offset += raw_line.len();
        line += 1;
    }
    Split {
        frontmatter: Some((after_open, 2)),
        body: "",
        body_line: line,
    }
}
