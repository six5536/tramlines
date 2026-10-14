//! Repairs as edits: the engine's own (bound units, headings, generated
//! anchors) and a host's, applied to a text in one pass.
//!
//! An edit replaces a range of the text's lines as it was read, so edits
//! from several sources compose without knowing of each other. Two edits
//! that claim one line are a defect of whoever made them.

use std::ops::Range;

use crate::{
    Error, Result,
    check::{Placement, Repair, Spot, heading_col},
    markdown::{Body, Segments, render_anchor},
};

// sokf:begin edit
/// Lines `lines` of a text, one-based and end-exclusive, replaced by
/// `text`: each line ending with `\n`, empty to delete them. An empty
/// range inserts before its start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The lines replaced.
    pub lines: Range<usize>,
    /// What replaces them.
    pub text: String,
}
// sokf:end edit

impl Edit {
    /// Lines `lines` replaced by `new`.
    pub fn replace(lines: Range<usize>, new: &[String]) -> Self {
        Edit {
            lines,
            text: new.iter().map(|l| format!("{l}\n")).collect(),
        }
    }
}

/// Text as written and as it is respelled, with the first and last line
/// it may be on, or on every line with `None`. A respelling never reaches
/// into a code span.
pub type Rename = (Option<(usize, usize)>, String, String);

// sokf:begin line-edits
/// The anchors of one document, rewritten: every anchor the body holds is
/// removed, and the anchor each key needs is written where it goes, with
/// the renames on the lines they reach.
#[derive(Debug, Clone, Copy)]
pub struct AnchorEdit<'a> {
    /// The body as parsed: the anchors it holds.
    pub body: &'a Body,
    /// The anchor each key needs.
    pub placements: &'a [Placement],
    /// The whole text, LF.
    pub text: &'a str,
    /// The respellings, as `[host:#User]` to `[host:#user]`.
    pub renames: &'a [Rename],
}
// sokf:end line-edits

impl AnchorEdit<'_> {
    /// Whether `line` holds anchors only, and goes.
    fn drops(&self, line: usize) -> bool {
        self.body.anchors.iter().any(|a| a.line == line && a.alone)
    }

    /// The lines written before the block that starts on `line`, as `raw`
    /// is written: its anchors, then a blank line, each with the block's
    /// indentation and quote markers.
    fn before(&self, line: usize, raw: &str) -> Vec<String> {
        let ids: Vec<String> = self
            .placements
            .iter()
            .filter(|p| p.spot == Spot::Before(line))
            .map(|p| render_anchor(&p.id))
            .collect();
        if ids.is_empty() {
            return Vec::new();
        }
        let prefix: String = raw
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t' | '>'))
            .collect();
        vec![
            format!("{prefix}{}", ids.concat()),
            prefix.trim_end().to_string(),
        ]
    }

    /// `raw`, the text of `line`, without its anchors and with those its
    /// keys need.
    fn line(&self, line: usize, raw: &str) -> String {
        let mut clean = self.body.clean_line(line, raw).into_owned();
        // An anchor that ended the line leaves no space behind it.
        if raw.trim_end().ends_with("</a>") && clean.len() < raw.len() {
            clean.truncate(clean.trim_end().len());
        }
        let mut at: Vec<(usize, &str)> = self
            .placements
            .iter()
            .filter_map(|p| match p.spot {
                Spot::Inline(l, col) if l == line => Some((col, p.id.as_str())),
                Spot::Heading(l) if l == line => {
                    Some((heading_col(self.body, self.text, line), p.id.as_str()))
                }
                _ => None,
            })
            .collect();
        let clean = self.rename(line, clean, &mut at);
        insert(clean, &mut at)
    }

    /// `clean` with each rename that reaches `line` applied outside code,
    /// and the columns in `at` moved with the text.
    fn rename(&self, line: usize, mut clean: String, at: &mut [(usize, &str)]) -> String {
        let mut found: Vec<(usize, &str, &str)> = Vec::new();
        let on_line = |r: &&Rename| {
            r.0.is_none_or(|(first, last)| first <= line && line <= last)
        };
        for (_, old, new) in self.renames.iter().filter(on_line) {
            for (pos, _) in clean.match_indices(old.as_str()) {
                // sokf-implements: MD-8_AC-5
                if !self.body.in_code_span(line, pos) {
                    found.push((pos, old, new));
                }
            }
        }
        found.sort_by_key(|f| std::cmp::Reverse(f.0));
        for (pos, old, new) in found {
            clean.replace_range(pos..pos + old.len(), new);
            for (col, _) in at.iter_mut() {
                if *col >= pos + old.len() {
                    *col = *col + new.len() - old.len();
                }
            }
        }
        clean
    }

    /// A heading rewritten as `prefix` and its text, with the anchor its
    /// key needs before the text.
    fn heading(&self, line: usize, prefix: &str, text: &str) -> String {
        let mut at: Vec<(usize, &str)> = self
            .placements
            .iter()
            .filter(|p| p.spot == Spot::Heading(line))
            .map(|p| (prefix.len(), p.id.as_str()))
            .collect();
        insert(format!("{prefix}{text}"), &mut at)
    }
}

/// `line` with an anchor inserted at each byte column, from the right so
/// the columns before hold.
fn insert(mut line: String, at: &mut [(usize, &str)]) -> String {
    at.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(a.1)));
    for (col, id) in at.iter() {
        let col = (*col).min(line.len());
        if line.is_char_boundary(col) {
            line.insert_str(col, &render_anchor(id));
        }
    }
    line
}

/// The engine's edits of `text`: each repair, a heading or a bound unit
/// rewritten whole, and with `anchors`, every line's anchors and renames.
/// Generated content is the host's: a region's interior, the definition
/// block and the blank lines before it are never touched.
// sokf-implements: MD-8_AC-2
// sokf-implements: TLMD_139
// sokf-implements: TLMD_140
// sokf:begin line-edits
pub fn line_edits(
    text: &str,
    segments: &Segments,
    repairs: &[Repair],
    anchors: Option<&AnchorEdit<'_>>,
) -> Vec<Edit> {
    // sokf:end line-edits
    let lines: Vec<&str> = text.lines().collect();
    let blank = |n: usize| lines.get(n - 1).is_some_and(|l| l.trim().is_empty());
    let generated = segments.definition.as_ref().map_or(lines.len() + 1, |d| {
        let mut first = d.line;
        while first > 1 && blank(first - 1) {
            first -= 1;
        }
        first
    });
    let mut edits = Vec::new();
    let mut i = 1;
    // sokf-implements: MD-8_AC-3
    while i < generated {
        if segments.in_region(i) {
            i += 1;
            continue;
        }
        let raw = lines[i - 1];
        // A line of anchors goes, with the blank line after it; the anchors
        // a block needs are written before the block.
        if let Some(a) = anchors
            && a.drops(i)
        {
            let end = if i + 1 < generated && blank(i + 1) {
                i + 2
            } else {
                i + 1
            };
            edits.push(Edit::replace(i..end, &[]));
            i = end;
            continue;
        }
        let mut new = anchors.map(|a| a.before(i, raw)).unwrap_or_default();
        // sokf-implements: MD-8_AC-1
        let end = match repairs.iter().find(|r| r.line == i) {
            Some(r) => {
                new.push(match anchors {
                    Some(a) => a.heading(i, &r.prefix, &r.text),
                    None => format!("{}{}", r.prefix, r.text),
                });
                r.end_line.max(i) + 1
            }
            None => {
                new.push(match anchors {
                    Some(a) => a.line(i, raw),
                    None => raw.to_string(),
                });
                i + 1
            }
        };
        // sokf-implements: MD-8_AC-4
        if end != i + 1 || new.len() != 1 || new[0] != raw {
            edits.push(Edit::replace(i..end, &new));
        }
        i = end;
    }
    edits
}

/// `text` with `edits` applied, LF, ending with a newline unless empty.
/// Edits refer to the lines of `text`; two that claim one line are a
/// defect of whoever made them. At one line, an insertion goes before a
/// replacement.
// sokf-implements: MD-8_AC-6
// sokf-implements: TL_82
// sokf-implements: TL_83
// sokf-implements: TL_84
// sokf:begin line-edits
pub fn apply(text: &str, edits: &[Edit]) -> Result<String> {
    // sokf:end line-edits
    let lines: Vec<&str> = text.lines().collect();
    let mut sorted: Vec<&Edit> = edits.iter().collect();
    sorted.sort_by_key(|e| (e.lines.start, e.lines.end));
    let mut out: Vec<&str> = Vec::new();
    let mut next = 1;
    for e in sorted {
        // sokf-implements: MD-8_AC-7
        if e.lines.start < next || e.lines.end < e.lines.start || e.lines.end > lines.len() + 1 {
            return Err(Error::Internal(format!(
                "edit of lines {}..{} overlaps another or leaves the text",
                e.lines.start, e.lines.end
            )));
        }
        out.extend(&lines[next - 1..e.lines.start - 1]);
        out.extend(e.text.lines());
        next = e.lines.end;
    }
    out.extend(&lines[next - 1..]);
    // sokf-implements: MD-8_AC-8
    if out.is_empty() {
        return Ok(String::new());
    }
    let mut result = out.join("\n");
    result.push('\n');
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_go_in_from_the_right() {
        let mut at = vec![(0, "a"), (4, "b")];
        assert_eq!(
            insert("one two".into(), &mut at),
            "<a id=\"a\"></a>one <a id=\"b\"></a>two"
        );
    }

    // sokf-tests: MD_P-3
    // sokf-tests: MD-8_AC-7
    #[test]
    fn edits_apply_in_one_pass_and_refuse_an_overlap() {
        let text = "a\nb\nc\nd\n";
        let edits = [
            Edit::replace(2..3, &["B".into(), "B2".into()]),
            Edit::replace(4..4, &["before d".into()]),
            Edit::replace(4..5, &[]),
            Edit::replace(1..1, &["first".into()]),
        ];
        assert_eq!(
            apply(text, &edits).unwrap(),
            "first\na\nB\nB2\nc\nbefore d\n"
        );
        assert_eq!(apply(text, &[]).unwrap(), text);
        assert_eq!(apply("", &[]).unwrap(), "");
        assert_eq!(
            apply("a", &[Edit::replace(2..2, &["b".into()])]).unwrap(),
            "a\nb\n"
        );
        let clash = [Edit::replace(1..3, &[]), Edit::replace(2..3, &[])];
        assert!(apply(text, &clash).is_err());
        assert!(apply(text, &[Edit::replace(4..9, &[])]).is_err());
    }

    #[test]
    fn repairs_are_edits_and_generated_content_is_left_alone() {
        let text = "Title\n=====\n\n<!-- host:include x -->\n\nstale\n\n<!-- /host:include -->\n\n<!-- host:links -->\n\n[host:x]: /x.md\n";
        let dialect = crate::markdown::Dialect {
            namespace: "host",
            region: Some("include"),
            trailer: Some("links"),
            scheme: Some("host"),
        };
        let segments = crate::markdown::segments(text, 1, &dialect);
        let repairs = [Repair::heading(1, 2, 1, "Title".into())];
        assert_eq!(
            line_edits(text, &segments, &repairs, None),
            vec![Edit::replace(1..3, &["# Title".into()])]
        );
    }
}
