//! Repairs as edits: the engine's own (bound units, headings, generated
//! anchors) and a host's, applied to a text in one pass.
//!
//! An edit replaces a range of the text's lines as it was read, so edits
//! from several sources compose without knowing of each other. Two edits
//! that claim one line are a defect of whoever made them.

use std::ops::Range;

use crate::{
    Error, Result,
    check::{LabelRepair, Placement, Repair, Spot, heading_col},
    markdown::{Body, Segments, gap, render_anchor},
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

    /// Whether `line` holds only the anchors of an item that opens a list
    /// straight after a paragraph, and gets none of them back.
    fn empties(&self, line: usize) -> bool {
        self.body
            .anchors
            .iter()
            .any(|a| a.line == line && a.joins_text)
            && !self.placements.iter().any(|p| match p.spot {
                Spot::Inline(l, _) | Spot::Heading(l) => l == line,
                Spot::Before(_) => false,
            })
    }

    /// Whether `raw`, the text of `line`, holds text once its anchors go,
    /// and gets no anchors written before it.
    fn holds_text(&self, line: usize, raw: &str) -> bool {
        !self.body.clean_line(line, raw).trim().is_empty() && self.before(line, raw).is_empty()
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
        vec![format!("{prefix}{}", ids.concat()), gap(raw)]
    }

    /// `raw`, the text of `line`, without its anchors and with those its
    /// keys need, and each label of `labels` on the line rewritten.
    fn line(&self, line: usize, raw: &str, labels: &[LabelRepair]) -> String {
        let mut clean = self.body.clean_line(line, raw).into_owned();
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
        // An anchor that ended the line leaves no space behind it, unless an
        // anchor is written there again, as after an item's marker.
        let end = clean.trim_end().len();
        if raw.trim_end().ends_with("</a>")
            && clean.len() < raw.len()
            && at.iter().all(|(col, _)| *col <= end)
        {
            clean.truncate(end);
        }
        // The renames read the code spans of the line as parsed, so they
        // come first, and move the column of each label with the text.
        let here: Vec<&LabelRepair> = labels.iter().filter(|l| l.line == line).collect();
        let mut cols: Vec<usize> = here.iter().map(|l| l.col).collect();
        let clean = self.rename(line, clean, &mut at, &mut cols);
        let clean = relabel(clean, &here, &cols, &mut at);
        insert(clean, &mut at)
    }

    /// `clean` with each rename that reaches `line` applied outside code,
    /// and the columns in `at` and in `cols` moved with the text.
    fn rename(
        &self,
        line: usize,
        mut clean: String,
        at: &mut [(usize, &str)],
        cols: &mut [usize],
    ) -> String {
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
            for col in at.iter_mut().map(|(c, _)| c).chain(cols.iter_mut()) {
                if *col >= pos + old.len() {
                    *col = *col + new.len() - old.len();
                }
            }
        }
        clean
    }

    /// The lines of `repair` rewritten as one, with each anchor placed on
    /// them before the new text, because the columns of the old text are
    /// gone: a heading's, and any on the lines of a unit.
    fn rewritten(&self, repair: &Repair) -> String {
        let lines = repair.line..=repair.end_line.max(repair.line);
        let mut at: Vec<(usize, &str)> = self
            .placements
            .iter()
            .filter(|p| match p.spot {
                Spot::Heading(l) => l == repair.line,
                Spot::Inline(l, _) => lines.contains(&l),
                Spot::Before(_) => false,
            })
            .map(|p| (repair.prefix.len(), p.id.as_str()))
            .collect();
        insert(format!("{}{}", repair.prefix, repair.text), &mut at)
    }
}

/// `clean`, the text of a line without its anchors, with each label of
/// `labels` rewritten at its column of `cols`, and the columns in `at`
/// after it moved with the text. A label that doesn't stand at its column
/// is left as it is.
// sokf-implements: TLMD_254
// sokf-implements: MD-8_AC-9
fn relabel(
    mut clean: String,
    labels: &[&LabelRepair],
    cols: &[usize],
    at: &mut [(usize, &str)],
) -> String {
    let mut here: Vec<(usize, &LabelRepair)> =
        cols.iter().copied().zip(labels.iter().copied()).collect();
    here.sort_by_key(|(col, _)| std::cmp::Reverse(*col));
    here.dedup_by_key(|(col, _)| *col);
    for (start, l) in here {
        if !clean
            .get(start..)
            .is_some_and(|rest| rest.starts_with(&l.old))
        {
            continue;
        }
        let end = start + l.old.len();
        clean.replace_range(start..end, &l.new);
        for (col, _) in at.iter_mut() {
            if *col >= end {
                *col = *col - l.old.len() + l.new.len();
            }
        }
    }
    clean
}

/// `raw`, the text of `line` with its anchors, with each label of `labels`
/// on the line rewritten where it stands: at its column, which counts no
/// anchor, moved right past the anchors that stand before the label.
fn relabel_raw(line: usize, raw: &str, labels: &[LabelRepair]) -> String {
    let mut out = raw.to_string();
    let mut here: Vec<&LabelRepair> = labels.iter().filter(|l| l.line == line).collect();
    here.sort_by_key(|l| std::cmp::Reverse(l.col));
    here.dedup_by_key(|l| l.col);
    for l in here {
        let Some(start) = past_anchors(&out, l.col) else {
            continue;
        };
        if out[start..].starts_with(&l.old) {
            out.replace_range(start..start + l.old.len(), &l.new);
        }
    }
    out
}

/// The byte of `raw` that stands at `col` of the line without its anchors:
/// each anchor before it, or at it, moves it right.
fn past_anchors(raw: &str, col: usize) -> Option<usize> {
    let (mut clean, mut i) = (0, 0);
    loop {
        let rest = raw.get(i..)?;
        if rest.starts_with("<a id=\"")
            && let Some(end) = rest.find("\"></a>")
        {
            i += end + "\"></a>".len();
            continue;
        }
        if clean == col {
            return Some(i);
        }
        let c = rest.chars().next()?;
        i += c.len_utf8();
        clean += c.len_utf8();
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
/// rewritten whole, each label of `labels` rewritten where it stands, and
/// with `anchors`, every line's anchors and renames.
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
    labels: &[LabelRepair],
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
    // Whether the body so far is empty or ends with a blank line: the
    // frontmatter joins no block of the body.
    let body_start = anchors.map_or(1, |a| a.body.first_line);
    let mut apart = true;
    let mut i = 1;
    // sokf-implements: MD-8_AC-3
    while i < generated {
        // A region's close marker, which follows its interior, sets `apart`.
        if segments.in_region(i) {
            i += 1;
            continue;
        }
        let raw = lines[i - 1];
        // A line of anchors goes, with the blank line after it; the anchors
        // a block needs are written before the block. The parser read the
        // line as blank, so where text stands on both sides, a blank line
        // keeps the next line from joining the block before it.
        if let Some(a) = anchors
            && a.drops(i)
        {
            let end = if i + 1 < generated && blank(i + 1) {
                i + 2
            } else {
                i + 1
            };
            let joins = !apart && end < generated && a.holds_text(end, lines[end - 1]);
            let keep = if joins {
                vec![String::new()]
            } else {
                Vec::new()
            };
            edits.push(Edit::replace(i..end, &keep));
            apart |= joins;
            i = end;
            continue;
        }
        let mut new = anchors.map(|a| a.before(i, raw)).unwrap_or_default();
        let repair = repairs.iter().find(|r| r.line == i);
        // An item whose anchors go would leave its marker alone, which the
        // paragraph before it takes, so a blank line keeps them apart.
        if let Some(a) = anchors
            && repair.is_none()
            && !apart
            && a.empties(i)
        {
            new.insert(0, gap(raw));
        }
        // sokf-implements: MD-8_AC-1
        let end = match repair {
            Some(r) => {
                new.push(match anchors {
                    Some(a) => a.rewritten(r),
                    None => format!("{}{}", r.prefix, r.text),
                });
                r.end_line.max(i) + 1
            }
            None => {
                new.push(match anchors {
                    Some(a) => a.line(i, raw, labels),
                    None => relabel_raw(i, raw, labels),
                });
                i + 1
            }
        };
        apart = i < body_start || new.last().is_none_or(|l| l.trim().is_empty());
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
    use crate::{RepoPath, document::Document, markdown::Dialect};

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
            line_edits(text, &segments, &repairs, &[], None),
            vec![Edit::replace(1..3, &["# Title".into()])]
        );
    }

    /// `text` with `repairs` and the anchors of `placements` applied.
    fn repaired(text: &str, repairs: &[Repair], placements: &[Placement]) -> String {
        let doc = crate::document::Document::parse(
            crate::RepoPath::root(),
            text,
            &crate::markdown::Dialect::default(),
        );
        let anchors = AnchorEdit {
            body: &doc.body,
            placements,
            text: &doc.text,
            renames: &[],
        };
        let edits = line_edits(&doc.text, &doc.segments, repairs, &[], Some(&anchors));
        apply(&doc.text, &edits).unwrap()
    }

    // The anchor of a key that a host places on the lines of a unit that a
    // repair rewrites goes at the start of the new text, and stays there.
    // sokf-tests: MD-8_AC-2
    #[test]
    fn a_rewritten_unit_keeps_the_anchors_placed_on_its_lines() {
        let place = |id: &str, line, col| Placement {
            id: id.into(),
            key: id.into(),
            spot: Spot::Inline(line, col),
        };
        let item = |line, end_line, text: &str| Repair {
            line,
            end_line,
            prefix: "- ".into(),
            text: text.into(),
        };
        let text = "# Note\n\n- <a id=\"d-1\"></a>D-1 Old\n- Two\n  <a id=\"d-2\"></a>lines\n";
        let placements = [place("d-1", 3, 2), place("d-2", 5, 2)];
        let repairs = [item(3, 3, "D-1 New"), item(4, 5, "D-2 New")];
        let fixed = repaired(text, &repairs, &placements);
        assert_eq!(
            fixed,
            "# Note\n\n- <a id=\"d-1\"></a>D-1 New\n- <a id=\"d-2\"></a>D-2 New\n"
        );
        let placements = [place("d-1", 3, 2), place("d-2", 4, 2)];
        assert_eq!(repaired(&fixed, &[], &placements), fixed);
    }

    // A label repaired on a line that a rename reaches keeps the rename
    // out of the code span, and a second repair of one label changes
    // nothing more.
    // sokf-tests: MD-8_AC-9
    #[test]
    fn a_label_and_a_rename_on_one_line_each_find_their_text() {
        let text = "- D-1 [human] see `#U` and #U, [human]\n";
        let document = Document::parse(RepoPath::root(), text, &Dialect::default());
        let renames = [(None, "#U".to_string(), "#u".to_string())];
        let label = LabelRepair {
            line: 1,
            col: 6,
            old: "[human]".into(),
            new: "[Mensch]".into(),
        };
        let labels = [label.clone(), label];
        let anchors = AnchorEdit {
            body: &document.body,
            placements: &[],
            text,
            renames: &renames,
        };
        let edits = line_edits(text, &document.segments, &[], &labels, Some(&anchors));
        assert_eq!(
            apply(text, &edits).unwrap(),
            "- D-1 [Mensch] see `#U` and #u, [human]\n"
        );
        let edits = line_edits(text, &document.segments, &[], &labels, None);
        assert_eq!(
            apply(text, &edits).unwrap(),
            "- D-1 [Mensch] see `#U` and #U, [human]\n"
        );
    }

    // Comrak drops the backslash of each `\|` in a paragraph directly
    // before a table, and a respelling there still reaches the text right
    // up to a code span and leaves the code alone.
    // sokf-tests: MD-8_AC-5
    #[test]
    fn a_respelling_past_an_escaped_pipe_before_a_table_leaves_code_alone() {
        let text = "a \\| \\| \\| see #U`#U`\n| A |\n|---|\n| c |\n";
        let document = Document::parse(RepoPath::root(), text, &Dialect::default());
        let renames = [(None, "#U".to_string(), "#u".to_string())];
        let anchors = AnchorEdit {
            body: &document.body,
            placements: &[],
            text,
            renames: &renames,
        };
        let edits = line_edits(text, &document.segments, &[], &[], Some(&anchors));
        assert_eq!(
            apply(text, &edits).unwrap(),
            "a \\| \\| \\| see #u`#U`\n| A |\n|---|\n| c |\n"
        );
    }
}
