//! Finds a host's region and trailer markers outside fenced code, before
//! block grouping, so generated content never reaches block matching.

use serde::{Deserialize, Serialize};

use super::{Dialect, Marker};

/// A region as found on disk: the host's marker pair and what lies
/// between.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// The open marker's target, as written.
    pub target: String,
    /// The line of the open marker.
    pub open_line: usize,
    /// The line of the close marker.
    pub close_line: usize,
    /// The lines between the markers, each ending with `\n`.
    pub content: String,
}

/// The generated block of reference definitions that ends a document, as
/// found on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionBlock {
    /// The line of the marker.
    pub line: usize,
    /// The reference definitions after it: label and destination.
    pub entries: Vec<(String, String)>,
    /// Whether anything but a definition follows the marker.
    pub trailing: bool,
    /// The text from the marker line to the end of the body.
    pub text: String,
}

/// The markers of one body.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segments {
    /// The regions, in order.
    pub regions: Vec<Region>,
    /// The definition block.
    pub definition: Option<DefinitionBlock>,
    /// The line of an open marker with no close marker.
    pub unclosed: Option<usize>,
    /// Lines of close markers with no open marker.
    pub stray_closes: Vec<usize>,
}

impl Segments {
    /// Whether `line` lies strictly inside a region.
    pub fn in_region(&self, line: usize) -> bool {
        self.regions
            .iter()
            .any(|i| i.open_line < line && line < i.close_line)
            || self.unclosed.is_some_and(|open| line > open)
    }

    /// Whether `line` lies in the definition block.
    pub fn in_definition(&self, line: usize) -> bool {
        self.definition.as_ref().is_some_and(|d| line >= d.line)
    }

    /// Whether `line` is generated content or a marker.
    pub fn is_generated(&self, line: usize) -> bool {
        self.in_region(line)
            || self.in_definition(line)
            || self
                .regions
                .iter()
                .any(|i| i.open_line == line || i.close_line == line)
    }
}

/// Strip block-quote and list markers, as a fence inside them is still a
/// fence.
fn strip_container_markers(line: &str) -> &str {
    let mut t = line.trim_start();
    loop {
        if let Some(rest) = t.strip_prefix('>') {
            t = rest.trim_start();
            continue;
        }
        if let Some(rest) = t
            .strip_prefix("- ")
            .or_else(|| t.strip_prefix("* "))
            .or_else(|| t.strip_prefix("+ "))
        {
            t = rest.trim_start();
            continue;
        }
        let digits = t.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && digits <= 9 {
            let after = &t[digits..];
            if let Some(rest) = after
                .strip_prefix(". ")
                .or_else(|| after.strip_prefix(") "))
            {
                t = rest.trim_start();
                continue;
            }
        }
        return t;
    }
}

/// A fence line: the fence character and its length.
fn fence(line: &str) -> Option<(char, usize)> {
    let t = strip_container_markers(line);
    let c = t.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let len = t.chars().take_while(|&x| x == c).count();
    if len < 3 {
        return None;
    }
    // A backtick fence's info string may not contain a backtick.
    if c == '`' && t[len..].contains('`') {
        return None;
    }
    Some((c, len))
}

fn closes(line: &str, open: (char, usize)) -> bool {
    let t = strip_container_markers(line).trim_end();
    let n = t.chars().take_while(|&x| x == open.0).count();
    n >= open.1 && t.chars().all(|x| x == open.0)
}

/// Find the markers of `body`, whose first line is file line `first_line`,
/// as `dialect` spells them.
// sokf-implements: TLMD_13
// sokf-implements: TLMD_14
// sokf-implements: TLMD_16
pub fn segments(body: &str, first_line: usize, dialect: &Dialect) -> Segments {
    let mut out = Segments::default();
    let lines: Vec<&str> = body.lines().collect();
    let mut open_fence: Option<(char, usize)> = None;
    let mut open: Option<(String, usize, usize)> = None; // target, line, index
    for (i, line) in lines.iter().copied().enumerate() {
        let file_line = first_line + i;
        if let Some(f) = open_fence {
            if closes(line, f) {
                open_fence = None;
            }
            continue;
        }
        match dialect.marker(line) {
            Some(Marker::Open(target)) if open.is_none() => {
                open = Some((target, file_line, i));
            }
            Some(Marker::Open(_)) => {}
            Some(Marker::Close) => match open.take() {
                Some((target, open_line, open_index)) => {
                    let content: String = lines[open_index + 1..i]
                        .iter()
                        .map(|l| format!("{l}\n"))
                        .collect();
                    out.regions.push(Region {
                        target,
                        open_line,
                        close_line: file_line,
                        content,
                    });
                }
                None => out.stray_closes.push(file_line),
            },
            Some(Marker::Trailer) if open.is_none() => {
                out.definition = Some(definition(&lines[i..], file_line));
                return finish(out, open);
            }
            Some(Marker::Trailer) => {}
            None => open_fence = fence(line),
        }
    }
    finish(out, open)
}

fn finish(mut out: Segments, open: Option<(String, usize, usize)>) -> Segments {
    if let Some((_, line, _)) = open {
        out.unclosed = Some(line);
    }
    out
}

fn definition(lines: &[&str], line: usize) -> DefinitionBlock {
    let mut entries = Vec::new();
    let mut trailing = false;
    for l in &lines[1..] {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        match ref_def(t) {
            Some((label, dest)) => entries.push((label, dest)),
            None => trailing = true,
        }
    }
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    DefinitionBlock {
        line,
        entries,
        trailing,
        text,
    }
}

/// Parse `[label]: destination`.
pub fn ref_def(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    let rest = t.strip_prefix('[')?;
    let end = rest.find("]:")?;
    let label = &rest[..end];
    if label.is_empty() || label.contains('[') {
        return None;
    }
    let dest = rest[end + 2..].trim();
    let dest = dest.split_whitespace().next()?;
    Some((label.to_string(), dest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: Dialect = Dialect {
        namespace: "host",
        region: Some("include"),
        trailer: Some("links"),
        scheme: Some("host"),
    };

    fn segments(body: &str, first_line: usize) -> Segments {
        super::segments(body, first_line, &HOST)
    }

    #[test]
    fn finds_includes_and_the_definition_block() {
        let body = "intro\n\n<!-- host:include a -->\n\nbody a\n\n<!-- /host:include -->\n\n```\n<!-- host:include fenced -->\n```\n\n<!-- host:links -->\n\n[host:a]: /k/a.md\n[host:b#c]: /k/b.md#c\n";
        let s = segments(body, 10);
        assert_eq!(s.regions.len(), 1);
        let inc = &s.regions[0];
        assert_eq!(inc.target, "a");
        assert_eq!((inc.open_line, inc.close_line), (12, 16));
        assert_eq!(inc.content, "\nbody a\n\n");
        let def = s.definition.clone().unwrap();
        assert_eq!(def.line, 22);
        assert_eq!(
            def.entries,
            vec![
                ("host:a".to_string(), "/k/a.md".to_string()),
                ("host:b#c".to_string(), "/k/b.md#c".to_string())
            ]
        );
        assert!(!def.trailing);
        assert!(def.text.starts_with("<!-- host:links -->\n"));
        assert!(s.unclosed.is_none());
        assert!(s.in_region(14));
        assert!(!s.in_region(12));
        assert!(s.in_definition(23));
        assert!(s.is_generated(12));
        assert!(!s.is_generated(11));
    }

    #[test]
    fn unclosed_and_stray_markers() {
        let s = segments(
            "<!-- /host:include -->\n<!-- host:include a -->\nx\n<!-- host:include b -->\n",
            1,
        );
        assert_eq!(s.stray_closes, vec![1]);
        assert_eq!(s.unclosed, Some(2));
        assert!(s.regions.is_empty());
        assert!(s.in_region(3));
        // A links marker inside an open include is content, not the block.
        let s = segments(
            "<!-- host:include a -->\n<!-- host:links -->\n<!-- /host:include -->\n",
            1,
        );
        assert!(s.definition.is_none());
        assert_eq!(s.regions.len(), 1);
    }

    #[test]
    fn definition_block_with_trailing_content() {
        let s = segments("x\n<!-- host:links -->\n[host:a]: /a.md\nnot a def\n", 1);
        let def = s.definition.unwrap();
        assert!(def.trailing);
        assert_eq!(def.entries.len(), 1);
    }

    #[test]
    fn fences_hide_markers_including_tildes_and_longer_closers() {
        let body = "~~~\n<!-- host:links -->\n~~~\n````md\n```\n<!-- host:include a -->\n````\n- ```\n  <!-- host:include a -->\n  ```\n<!-- host:links -->\n";
        let s = segments(body, 1);
        assert!(s.regions.is_empty());
        assert_eq!(s.definition.unwrap().line, 11);
        assert_eq!(fence("``` a ` b"), None);
        assert_eq!(fence("``"), None);
        assert_eq!(fence("> ```"), Some(('`', 3)));
        assert_eq!(fence("- ```"), Some(('`', 3)));
        assert_eq!(fence("1. ~~~~"), Some(('~', 4)));
        assert_eq!(fence("- text"), None);
        assert_eq!(fence("x"), None);
    }

    #[test]
    fn ref_defs() {
        assert_eq!(ref_def("[a]: /x"), Some(("a".into(), "/x".into())));
        assert_eq!(
            ref_def("[a]: /x \"title\""),
            Some(("a".into(), "/x".into()))
        );
        assert_eq!(ref_def("[a]:"), None);
        assert_eq!(ref_def("[]: x"), None);
        assert_eq!(ref_def("[a][b]: x"), None);
        assert_eq!(ref_def("text"), None);
    }
}
