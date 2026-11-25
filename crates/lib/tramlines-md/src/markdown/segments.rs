//! Finds a host's region and trailer markers outside the fenced code
//! blocks that comrak reads, before block grouping, so generated content
//! never reaches block matching.

use std::{fmt::Write as _, ops::Range};

use serde::{Deserialize, Serialize};

use super::{Dialect, Marker, code, link_definitions};

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
    /// The reference definitions after it, as comrak reads them, each one
    /// over all its lines: label and destination as written.
    pub entries: Vec<(String, String)>,
    /// Whether anything but a definition at the top level follows the
    /// marker.
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

/// Find the markers of `body`, whose first line is file line `first_line`,
/// as `dialect` spells them.
// sokf-implements: TLMD_13
// sokf-implements: TLMD_14
// sokf-implements: TLMD_16
pub fn segments(body: &str, first_line: usize, dialect: &Dialect) -> Segments {
    let mut out = Segments::default();
    let lines: Vec<&str> = body.lines().collect();
    // The bytes of each fenced code block, which only a marker needs.
    let mut fences: Option<Vec<Range<usize>>> = None;
    let mut open: Option<(String, usize, usize)> = None; // target, line, index
    let mut start = 0;
    for (i, raw) in body.split_inclusive('\n').enumerate() {
        let line = lines[i];
        let file_line = first_line + i;
        let at = start;
        start += raw.len();
        let marker = dialect.marker(line).filter(|_| {
            // The fences are in order and apart, so the one that could hold
            // the line is the first that ends after it starts.
            let fences = fences.get_or_insert_with(|| code::fenced(body));
            let k = fences.partition_point(|f| f.end <= at);
            !fences.get(k).is_some_and(|f| f.contains(&at))
        });
        match marker {
            Some(Marker::Open(target)) if open.is_none() => {
                open = Some((target, file_line, i));
            }
            Some(Marker::Close) => match open.take() {
                Some((target, open_line, open_index)) => {
                    let content: String =
                        lines[open_index + 1..i]
                            .iter()
                            .fold(String::new(), |mut out, l| {
                                let _ = writeln!(out, "{l}");
                                out
                            });
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
                return finish(out, open.as_ref());
            }
            Some(Marker::Open(_) | Marker::Trailer) | None => {}
        }
    }
    finish(out, open.as_ref())
}

fn finish(mut out: Segments, open: Option<&(String, usize, usize)>) -> Segments {
    if let Some(&(_, line, _)) = open {
        out.unclosed = Some(line);
    }
    out
}

/// The trailer whose marker is the first of `lines`, at file line `line`.
/// Its definitions read as comrak reads them, so one whose destination or
/// title a formatter moves to a later line is one entry.
// sokf-implements: TLMD_268
// sokf-implements: MD-3_AC-17
fn definition(lines: &[&str], line: usize) -> DefinitionBlock {
    // A definition reads as one at the top level, whatever its indent, and
    // one in a quote or an item is other content.
    let after: Vec<&str> = lines[1..].iter().map(|l| l.trim_start()).collect();
    let top: Vec<_> = link_definitions::read(&after)
        .defs
        .into_iter()
        .filter(|d| after[d.first].starts_with('['))
        .collect();
    // Every line with text that no definition at the top level takes,
    // a `>` alone among them, trails the trailer.
    let taken = |i: usize| top.iter().any(|d| d.first <= i && i <= d.last);
    let trailing = after
        .iter()
        .enumerate()
        .any(|(i, l)| !l.trim().is_empty() && !taken(i));
    let entries = top.into_iter().map(|d| (d.label, d.dest)).collect();
    let text: String = lines.iter().fold(String::new(), |mut out, l| {
        let _ = writeln!(out, "{l}");
        out
    });
    DefinitionBlock {
        line,
        entries,
        trailing,
        text,
    }
}

/// Parse `[label]: destination` on one line, without the lines that a
/// definition's destination or title can take after it.
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
        assert!(s.regions.is_empty(), "{:?}", s.regions);
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

    // A definition of the trailer whose destination or title a formatter
    // moves to a later line is one entry, and nothing trails it.
    // sokf-tests: TLMD_14
    // sokf-tests: TLMD_268
    // sokf-tests: MD-3_AC-17
    #[test]
    fn a_wrapped_definition_of_the_trailer_is_one_entry() {
        for after in [
            "[host:a]: /k/a.md\n[host:b#c]: /k/b.md#c \"T\"\n",
            "[host:a]:\n  /k/a.md\n[host:b#c]:\n  /k/b.md#c\n  \"T\"\n",
            "[host:a]: /k/a.md\n\n[host:b#c]: /k/b.md#c\n\"T\"\n",
            "    [host:a]: /k/a.md\n[host:b#c]: /k/b.md#c \"T\nU\"\n",
        ] {
            let s = segments(&format!("x\n\n<!-- host:links -->\n\n{after}"), 1);
            let def = s.definition.unwrap();
            assert!(!def.trailing, "{after:?}");
            assert_eq!(
                def.entries,
                vec![
                    ("host:a".to_string(), "/k/a.md".to_string()),
                    ("host:b#c".to_string(), "/k/b.md#c".to_string())
                ],
                "{after:?}"
            );
        }
        let s = segments("<!-- host:links -->\n[host:a]: <k a.md>\n", 1);
        let def = s.definition.unwrap();
        assert!(!def.trailing);
        assert_eq!(
            def.entries,
            vec![("host:a".to_string(), "<k a.md>".to_string())]
        );
        // A label alone, a definition in a quote or an item, and a title
        // that never closes trail the trailer.
        for after in [
            "[host:a]:\n",
            "> [host:a]: /k/a.md\n",
            "[host:a]: /k/a.md\n>\n",
            "- [host:a]: /k/a.md\n",
            "[host:a]: /k/a.md \"T\n",
        ] {
            let s = segments(&format!("<!-- host:links -->\n{after}"), 1);
            assert!(s.definition.unwrap().trailing, "{after:?}");
        }
    }

    // sokf-tests: TLMD_16
    #[test]
    fn fences_hide_markers_including_tildes_and_longer_closers() {
        let body = "~~~\n<!-- host:links -->\n~~~\n````md\n```\n<!-- host:include a -->\n````\n- ```\n  <!-- host:include a -->\n  ```\n<!-- host:links -->\n";
        let s = segments(body, 1);
        assert!(s.regions.is_empty(), "{:?}", s.regions);
        assert_eq!(s.definition.unwrap().line, 11);
    }

    /// The target of each region of `body`.
    fn targets(body: &str) -> Vec<String> {
        segments(body, 1)
            .regions
            .into_iter()
            .map(|r| r.target)
            .collect()
    }

    // sokf-tests: TLMD_16
    #[test]
    fn a_fence_in_a_footnote_hides_a_marker() {
        let body = "Text[^1].\n\n[^1]: Note.\n\n    ```\n    <!-- host:include a -->\n    <!-- /host:include -->\n    ```\n";
        let found = targets(body);
        assert!(found.is_empty(), "{found:?}");
    }

    // sokf-tests: TLMD_16
    #[test]
    fn a_fence_ends_with_the_quote_or_the_item_that_holds_it() {
        let region = "\n<!-- host:include a -->\nx\n<!-- /host:include -->\n";
        for fence in [
            "> - ```\n>   code\n",
            "- a\n  - b\n\n    > ```\n    > code\n",
            ">\t```\n>\tcode\n",
            "- ```\n  code\n",
        ] {
            assert_eq!(targets(&format!("{fence}{region}")), ["a"], "{fence:?}");
        }
    }

    // sokf-tests: TLMD_16
    #[test]
    fn only_a_fence_hides_a_marker() {
        // A line of backticks inside an indented block, or after a fence's
        // info string holds a backtick, opens no fence.
        for text in ["    ```\n", "``` a ` b\n", "``\n"] {
            let body = format!("{text}\n<!-- host:include a -->\nx\n<!-- /host:include -->\n");
            assert_eq!(targets(&body), ["a"], "{text:?}");
        }
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
