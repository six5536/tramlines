//! `line-limit` and `prohibited-patterns`.

use super::{At, Checker};
use crate::grammar::refused;

/// The line limit; document-scope bans.
// sokf-implements: TL_21
// sokf-implements: TLMD_111
// sokf-implements: TLMD_171
pub(super) fn check(c: &mut Checker<'_>) {
    let schema = c.schema;
    let dialect = c.host.dialect();
    // A marker line is one the author deletes, with a blank line that sets
    // it apart, so neither counts: a scaffold is held to the length of what
    // it leaves behind. The blank line is the one after the marker, or the
    // one before it where the marker ends the file. An `only` marker sits
    // directly before the part it gates, which keeps the blank line above.
    let all: Vec<&str> = c.doc.text.lines().collect();
    let blank = |i: usize| all.get(i).is_some_and(|l| l.trim().is_empty());
    let mut dropped = vec![false; all.len()];
    for (i, l) in all.iter().enumerate() {
        // A YAML comment after a key marks a line that stays.
        let whole = l.trim_start().starts_with("<!--") || l.trim_start().starts_with('#');
        let marker = whole
            && (dialect.is_guidance_marker(l)
                || dialect.is_only_marker(l)
                || dialect.is_optional_mark(l));
        if !marker || c.doc.body.in_code(i + 1) {
            continue;
        }
        dropped[i] = true;
        if blank(i + 1) {
            dropped[i + 1] = true;
        } else if i + 1 == all.len() && i > 0 && blank(i - 1) && !dropped[i - 1] {
            dropped[i - 1] = true;
        }
    }
    let markers = dropped.iter().filter(|d| **d).count();
    let lines = c.doc.line_count.saturating_sub(markers);
    let key_line = |key: &str| At {
        line: schema.core.key_lines.get(key).copied(),
        ..At::default()
    };
    if schema.core.line_limit > 0 && lines as i64 > schema.core.line_limit {
        c.located(key_line("line-limit"), |c| {
            c.error(
                None,
                "TL_21",
                format!(
                    "the document has {lines} lines; the limit is {}; split it",
                    schema.core.line_limit
                ),
            );
        });
    }
    let body_line = c.doc.body_line;
    for (i, line) in all.iter().enumerate() {
        let file_line = i + 1;
        // A guidance line is the scaffold's note to its writer, not text of
        // the document, and it can quote the text that a ban refuses.
        // sokf-implements: TLMD_196
        if file_line < body_line
            || c.doc.body.in_code(file_line)
            || c.doc.segments.is_generated(file_line)
            || dialect.is_guidance_marker(line)
        {
            continue;
        }
        // A generated anchor is no part of the text (TLMD_107), and a
        // pattern quoted as code is not prose (LANG-1_AC-2).
        let clean = c.doc.body.clean_line(file_line, line);
        let line = &c.doc.body.mask_spans(file_line, &clean);
        // A document-wide ban guards no one rule, so its failure is the
        // span it matched and its pattern.
        // sokf-implements: TL_75
        if let Some(reason) = refused(line, &schema.core.prohibited_patterns, None) {
            c.located(key_line("prohibited-patterns"), |c| {
                c.error(Some(file_line), "TL_75", reason);
            });
        }
    }
}
