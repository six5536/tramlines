//! A unit bound to a `text`: the literal or the frontmatter value that it
//! must read, the finding where it reads otherwise, and the repair that
//! rewrites it whole.

use super::{Checker, Repair, label::Read, short};
use crate::{
    grammar::{Block as Rule, Shape, Text, Unit as UnitRule, squash, task_box},
    markdown::{BlockKind, Unit},
};

/// What a `text` rule expects: the literal, or the frontmatter value;
/// `None` when the key is absent.
fn expected_text(c: &Checker<'_>, text: &Text) -> Option<String> {
    c.text_of(text).map(|s| squash(&s))
}

/// What precedes a unit's text on its first line: the item marker, the
/// quote marker, the indentation; nothing for a bare paragraph. The line
/// is read without its anchors, so the repaired unit holds none, as the
/// anchor check judges the unit once repaired.
fn repair_prefix(c: &Checker<'_>, unit: &Unit) -> Option<String> {
    let raw = c.doc.text.lines().nth(unit.line.checked_sub(1)?)?;
    let clean = c.doc.body.clean_line(unit.line, raw);
    let line: &str = &clean;
    if unit.text.is_empty() {
        return Some(format!("{} ", line.trim_end()));
    }
    (0..=line.len())
        .filter(|&i| line.is_char_boundary(i))
        .find(|&i| {
            let rest = line[i..].trim_end();
            !rest.is_empty()
                && !rest.starts_with(char::is_whitespace)
                && unit.text.starts_with(rest)
        })
        .map(|i| line[..i].to_string())
}

/// The `text` rule on one unit: the finding, and the repair it
/// applies.
// sokf-implements: TLMD_136
// sokf-implements: TLMD_165
// sokf-implements: TLMD_168
// sokf-implements: MD-4_AC-7
pub(super) fn check_text(
    c: &mut Checker<'_>,
    unit: &Unit,
    rule: &UnitRule,
    block: &Rule,
    read: &Read,
) {
    let Some(Shape::Text(text)) = &rule.shape else {
        return;
    };
    let Some(expected) = expected_text(c, text) else {
        let Text::From(key) = text else {
            return;
        };
        if !c.unbound_is_own_fault(key) {
            return;
        }
        c.emit(
            &rule.common,
            Some(unit.line),
            "TLMD_168",
            format!(
                "`{}` is bound to the frontmatter `{key}`, which is absent",
                short(&unit.text)
            ),
        );
        return;
    };
    // The label is no part of the text that the rule fixes (TLMD_248).
    if read.squashed.text == expected {
        return;
    }
    let (criterion, message) = match text {
        Text::Literal(_) => (
            "TL_74",
            format!("`{}` must read `{expected}`", short(&unit.text)),
        ),
        Text::From(key) => (
            "TLMD_165",
            format!(
                "`{}` must equal the frontmatter `{key}`: `{expected}`",
                short(&unit.text)
            ),
        ),
    };
    c.emit(&rule.common, Some(unit.line), criterion, message);
    // The repair rewrites the unit whole, so it writes the unit's label
    // before the text, after the box of a task, in the host's words.
    if let Some(prefix) = repair_prefix(c, unit) {
        let at = match block.kind {
            BlockKind::BulletList | BlockKind::NumberedList => task_box(&expected),
            _ => 0,
        };
        // A label that the rule doesn't allow goes, so that the repaired
        // unit passes.
        let label = match block.label_of(Some(rule)) {
            Some(_) => c.label_kept(read),
            None => String::new(),
        };
        let text = format!("{}{label}{}", &expected[..at], &expected[at..]);
        c.out.repairs.push(Repair {
            line: unit.line,
            end_line: unit.end_line.max(unit.line),
            prefix,
            text,
        });
    }
}
