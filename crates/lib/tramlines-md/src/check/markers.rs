//! The engine's own markers in a governed document: a scaffold writes
//! them, and a document that keeps one has a part it has not decided.

use super::Checker;
use crate::{Finding, Severity};

/// Each `only` or `guidance` marker a governed document keeps is an error;
/// an `optional` region is a warning, and an end with no partner an error,
/// which is no region and so no warning too.
/// A fence and a region interior hold text, not markers. A scaffold carries
/// them all, so it is not held to this.
// sokf-implements: TLMD_10
// sokf-implements: TLMD_158
// sokf-implements: TLMD_11
// sokf-implements: TLMD_12
pub(super) fn check(c: &mut Checker<'_>) {
    if c.options.scaffold {
        return;
    }
    // sokf-implements: MD-5_AC-1
    let dialect = c.host.dialect();
    let optional = dialect.spell("optional");
    let unclosed = |line: usize| {
        (
            line,
            Severity::Error,
            "TLMD_12",
            format!("a `{optional}` region opens here and nothing closes it"),
        )
    };
    let mut found = Vec::new();
    let mut open: Option<usize> = None;
    for (i, raw) in c.doc.text.lines().enumerate() {
        let line = i + 1;
        if c.doc.body.in_code(line) || c.doc.segments.in_region(line) {
            continue;
        }
        if dialect.is_only_marker(raw) {
            found.push((
                line,
                Severity::Error,
                "TLMD_10",
                format!(
                    "a `{}` marker belongs in a scaffold, not a governed document",
                    dialect.spell("only")
                ),
            ));
        }
        if dialect.is_guidance_marker(raw) {
            found.push((
                line,
                Severity::Error,
                "TLMD_158",
                format!(
                    "a `{}` line is left here: write the part it guides, then delete the line",
                    dialect.spell("guidance")
                ),
            ));
        }
        if dialect.is_optional_marker(raw) {
            // A region does not nest, so one that is still open when the
            // next begins never closes.
            found.extend(open.map(unclosed));
            open = Some(line);
        }
        if dialect.is_optional_close(raw) {
            found.push(match open {
                Some(opened) => (
                    opened,
                    Severity::Warning,
                    "TLMD_11",
                    format!(
                        "a `{optional}` region is left here: keep what it holds and drop both markers, or drop the region"
                    ),
                ),
                None => (
                    line,
                    Severity::Error,
                    "TLMD_12",
                    format!("a `{optional}` region closes here and nothing opened it"),
                ),
            });
            open = None;
        }
    }
    found.extend(open.map(unclosed));
    found.sort_by_key(|(line, ..)| *line);
    for (line, severity, criterion, message) in found {
        let finding = Finding::new(c.doc.path.clone(), Some(line), severity, criterion, message);
        c.out.findings.push(finding);
    }
}
