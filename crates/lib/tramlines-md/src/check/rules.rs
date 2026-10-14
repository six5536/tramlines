//! The grammar of a block's units and of a fence's lines (TLMD_52 to TLMD_57):
//! productions take what their shape matches, a guard selects, a counter
//! counts and takes nothing, and a unit no rule takes is reported against
//! the production it follows furthest.

use super::{
    Checker,
    blocks::{check_unit, noun, reads, unit_noun},
    short,
};
use crate::{
    grammar::{Block as Rule, RuleCommon, Shape, Unit as UnitRule, squash},
    markdown::{Block, BlockDetail, Unit},
    text,
};

/// The content lines of a code block, each with the file line it sits on,
/// without its indent: the indent of a line is layout, as an item's marker
/// is (TLMD_6). A blank line is layout too, and no line at all. A
/// fenced block opens with a marker line; an indented one does not.
// sokf-implements: TLMD_82
fn content_lines(block: &Block) -> Vec<(usize, &str)> {
    let first = match block.detail {
        BlockDetail::Code { fence, .. } if fence > 0 => block.line + 1,
        _ => block.line,
    };
    match block.units.first() {
        Some(u) => u
            .text
            .lines()
            .enumerate()
            .map(|(i, text)| (first + i, text.trim_start()))
            .filter(|(_, text)| !text.trim().is_empty())
            .collect(),
        None => Vec::new(),
    }
}

/// The `lines` rules of a code block's unit rule, over its content lines:
/// the rules are the grammar of the fence, and a line no rule takes is an
/// error unless the rule opens itself with `lines-open`.
// sokf-implements: TLMD_83
// sokf-implements: TLMD_84
// sokf-implements: TLMD_86
// sokf-implements: TLMD_87
pub(super) fn check_lines(
    c: &mut Checker<'_>,
    block: &Block,
    unit_rule: &UnitRule,
    rule: &Rule,
    section: Option<usize>,
) {
    let Some(rules) = &unit_rule.lines else {
        return;
    };
    let units: Vec<Unit> = content_lines(block)
        .into_iter()
        .map(|(line, text)| Unit::leaf(text.to_string(), line, line))
        .collect();
    // A key a line carries has its anchor before the block (TLMD_106).
    let held = c.lines_block.replace(block.line);
    let taken = apply_rules(c, &units, rules, rule, section, block.line, "lines");
    c.lines_block = held;
    if !unit_rule.lines_open {
        untaken(
            c,
            &units,
            &taken,
            rules,
            &unit_rule.common,
            ("the fence", "line"),
            "lines-open",
        );
    }
}

/// A rule list over units, which is the grammar of the block (TLMD_52 to
/// TLMD_57). A production, a rule with a shape and no guard, takes the units
/// its shape matches, each unit the first such rule; a rule with a guard
/// takes what its guard selects and checks it, unless it only counts; a
/// rule with neither takes every unit. `min` and `max` bound what each rule
/// took, or counted. Returns which units some rule took.
// sokf-implements: TLMD_83
// sokf-implements: TLMD_84
// sokf-implements: TLMD_86
// sokf-implements: TLMD_52
// sokf-implements: TLMD_54
// sokf-implements: TLMD_55
// sokf-implements: TLMD_56
fn apply_rules(
    c: &mut Checker<'_>,
    units: &[Unit],
    rules: &[UnitRule],
    rule: &Rule,
    section: Option<usize>,
    at: usize,
    noun: &str,
) -> Vec<bool> {
    // Each unit's production: the first whose shape it has.
    let production: Vec<Option<usize>> = units
        .iter()
        .map(|u| {
            rules
                .iter()
                .position(|r| r.is_production() && satisfies_shape(c, u, r))
        })
        .collect();
    let mut taken = vec![false; units.len()];
    for (ri, guarded) in rules.iter().enumerate() {
        let selected: Vec<usize> = (0..units.len())
            .filter(|&i| match &guarded.select {
                Some(s) => s.is_match(&squash(&units[i].text)),
                None if guarded.has_shape() => production[i] == Some(ri),
                None => true,
            })
            .collect();
        let n = selected.len();
        let what = match (&guarded.select, &guarded.shape, &guarded.format) {
            (Some(s), _, _) | (None, Some(Shape::Pattern(s)), _) => {
                format!("the {noun} matching `{}`", s.as_str())
            }
            (None, Some(Shape::Template(t)), _) => {
                format!("the {noun} of the template `{}`", t.source)
            }
            (None, Some(shape @ Shape::Text(_)), _) => {
                format!("the {noun} reading `{}`", shape.label())
            }
            (None, None, Some(name)) => format!("the {noun} of the format `{name}`"),
            (None, None, None) => format!("the block's {noun}"),
        };
        if n < guarded.min {
            c.emit(
                &guarded.common,
                Some(at),
                "TLMD_93",
                format!(
                    "{what}: at least {} {} needed; there {} {n}",
                    guarded.min,
                    text::is_are(guarded.min),
                    text::is_are(n)
                ),
            );
        }
        if let Some(max) = guarded.max
            && n > max
        {
            // The first one over the limit is the one to look at, so a
            // `max` of zero names the unit it bans.
            let line = selected.get(max).map_or(at, |&i| units[i].line);
            // sokf-implements: TLMD_159
            c.emit(
                &guarded.common,
                Some(line),
                "TLMD_159",
                format!(
                    "{what}: at most {max} {} allowed; there {} {n}",
                    text::is_are(max),
                    text::is_are(n)
                ),
            );
        }
        // A counter counts and takes nothing, so it never hides a unit the
        // productions would report.
        if guarded.is_counter() {
            continue;
        }
        for i in selected {
            taken[i] = true;
            // `check_unit` files a capture under the line rule's `rule-id`,
            // else the block's, so a key from a line is reachable by
            // `for-each-key-of`.
            // sokf-implements: TLMD_88
            // sokf-implements: TLMD_89
            check_unit(c, &units[i], guarded, rule, section);
        }
    }
    taken
}

/// Whether `unit` has `rule`'s shape, its own or its format's, silently.
fn satisfies_shape(c: &Checker<'_>, unit: &Unit, rule: &UnitRule) -> bool {
    rule.fits(&c.schema.core.formats, &squash(&unit.text), |k| {
        c.front_value(k)
    })
}

/// Each unit no rule took, reported: against the production it follows
/// furthest, with where it fails and that rule's example, or as matching
/// no rule where the block has no production.
// sokf-implements: TLMD_57
fn untaken(
    c: &mut Checker<'_>,
    units: &[Unit],
    taken: &[bool],
    rules: &[UnitRule],
    common: &RuleCommon,
    (whose, unit_is): (&str, &str),
    open: &str,
) {
    for (unit, seen) in units.iter().zip(taken) {
        if *seen {
            continue;
        }
        let text = squash(&unit.text);
        // The first of the productions the text follows furthest, where it
        // follows one at all; a text that follows none matches no rule.
        let best = rules
            .iter()
            .filter(|r| r.is_production())
            .fold(None::<(&UnitRule, usize)>, |best, r| {
                let n = reach(c, r, &text);
                match best {
                    Some((_, m)) if m >= n => best,
                    _ => Some((r, n)),
                }
            })
            .filter(|(_, n)| *n > 0)
            .map(|(r, _)| r);
        match best {
            Some(r) => {
                let reason = shape_failure(c, r, &text, unit_is);
                c.emit(
                    &r.common,
                    Some(unit.line),
                    "TLMD_57",
                    format!(
                        "`{}` matches no rule of {whose}; the nearest {reason}",
                        short(&text)
                    ),
                );
            }
            None => c.emit(
                common,
                Some(unit.line),
                if open == "lines-open" {
                    "TLMD_87"
                } else {
                    "TLMD_53"
                },
                format!(
                    "`{}` matches no rule of {whose}; declare it or set `{open}`",
                    short(&text)
                ),
            ),
        }
    }
}

/// How far `text` follows `rule`'s shape before it fails.
pub(super) fn reach(c: &Checker<'_>, rule: &UnitRule, text: &str) -> usize {
    rule.reach(&c.schema.core.formats, text)
}

/// Why `text` does not have `rule`'s shape, with the example it shows.
fn shape_failure(c: &Checker<'_>, rule: &UnitRule, text: &str, unit_is: &str) -> String {
    match (&rule.shape, &rule.format) {
        (Some(shape), _) => format!("{}{}", shape.failure(text), reads(unit_is, rule.example())),
        (None, Some(name)) => match rule
            .format_in(&c.schema.core.formats)
            .map(|f| f.matching_as(text, unit_is))
        {
            Some(Err(reason)) => reason,
            _ => format!("format `{name}`"),
        },
        (None, None) => String::new(),
    }
}

/// A block's `rules`: as `lines`, and a unit no rule takes is an error
/// unless the block opens itself with `units-open`.
// sokf-implements: TLMD_53
pub(super) fn check_rules(
    c: &mut Checker<'_>,
    block: &Block,
    rules: &[UnitRule],
    rule: &Rule,
    section: Option<usize>,
) {
    let taken = apply_rules(
        c,
        &block.units,
        rules,
        rule,
        section,
        block.line,
        noun(rule.kind),
    );
    if !rule.units_open {
        untaken(
            c,
            &block.units,
            &taken,
            rules,
            &rule.common,
            ("the block", unit_noun(c, rule.kind)),
            "units-open",
        );
    }
}
