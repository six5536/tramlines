//! What a block and its units must satisfy: sequence, nested blocks,
//! per-kind keys. Which block a position gets is `claim`.

use super::{
    Capture, Checker, Paired, Repair, Spot, Subject,
    claim::check_blocks,
    rules::{check_lines, check_rules, reach},
    short,
};
use crate::{
    grammar::{
        Block as Rule, Captured, FormatMatch, Lang, RuleCommon, Shape, Squashed, Text,
        Unit as UnitRule, captured, refused_as, squash,
    },
    markdown::{Block, BlockDetail, BlockKind, Unit, host_ref},
    text,
};

/// What a `text` rule expects: the literal, or the frontmatter value;
/// `None` when the key is absent.
fn expected_text(c: &Checker<'_>, text: &Text) -> Option<String> {
    c.text_of(text).map(|s| squash(&s))
}

/// What precedes a unit's text on its first line: the item marker, the
/// quote marker, the indentation; nothing for a bare paragraph.
fn repair_prefix(c: &Checker<'_>, unit: &Unit) -> Option<String> {
    let line = c.doc.text.lines().nth(unit.line.checked_sub(1)?)?;
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
fn check_text(c: &mut Checker<'_>, unit: &Unit, rule: &UnitRule) {
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
    if squash(&unit.text) == expected {
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
    if let Some(prefix) = repair_prefix(c, unit) {
        c.out.repairs.push(Repair {
            line: unit.line,
            end_line: unit.end_line.max(unit.line),
            prefix,
            text: expected,
        });
    }
}

/// What one unit of a block of `kind` is called, as a failure names it;
/// a line of a fence while the lines of a code or HTML block are checked.
pub(super) fn unit_noun(c: &Checker<'_>, kind: BlockKind) -> &'static str {
    if c.lines_block.is_some() {
        return "line";
    }
    unit_of(kind)
}

/// What one unit of a block of `kind` is called.
pub(super) fn unit_of(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Prose | BlockKind::Blockquote => "paragraph",
        BlockKind::BulletList | BlockKind::NumberedList => "item",
        BlockKind::Table => "row",
        BlockKind::Image => "image",
        BlockKind::References => "definition",
        BlockKind::Code | BlockKind::Html => "line",
        _ => "unit",
    }
}

pub(super) fn noun(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Prose | BlockKind::Blockquote => "paragraphs",
        BlockKind::BulletList | BlockKind::NumberedList => "items",
        BlockKind::Table => "rows",
        BlockKind::Image => "images",
        BlockKind::References => "definitions",
        BlockKind::Group => "repetitions",
        _ => "units",
    }
}

/// Check one block against its rule. A group is not one: `claim` walks
/// its inner rules over the blocks it covers.
pub(super) fn check_plain(c: &mut Checker<'_>, block: &Block, rule: &Rule, section: Option<usize>) {
    c.gate(&rule.common, block.line);
    // A quote that holds only headings has them where its text should be,
    // which TLMD_172 reports, so it is not also short of paragraphs.
    let headed = block.units.is_empty()
        && matches!(&block.detail, BlockDetail::Quote { headings } if !headings.is_empty());
    if let Some(seq) = &rule.sequence {
        match_units(c, seq, &block.units, rule, section, block.line);
    } else if !matches!(rule.kind, BlockKind::Region | BlockKind::ThematicBreak) && !headed {
        check_count(c, rule, block.units.len(), block.line);
    }
    if let Some(each) = &rule.each {
        for unit in &block.units {
            check_unit(c, unit, each, rule, section);
        }
    }
    if let Some(rules) = &rule.rules {
        check_rules(c, block, rules, rule, section);
    }
    // A list with no unit rule declares no `blocks` for its items either.
    if rule.each.is_none() && rule.sequence.is_none() && rule.rules.is_none() {
        for unit in &block.units {
            no_nested(c, unit, &rule.common);
        }
    }
    if matches!(rule.kind, BlockKind::Code | BlockKind::Html) {
        for unit_rule in rule.units() {
            check_lines(c, block, unit_rule, rule, section);
        }
    }
    match (&block.detail, rule.kind) {
        (BlockDetail::Code { lang, fence }, BlockKind::Code) => {
            if lang.is_empty() {
                c.emit(
                    &rule.common,
                    Some(block.line),
                    "TLMD_43",
                    "the code block has no tag",
                );
            } else if let Some(Lang::Tags(tags)) = &rule.lang
                && !tags.iter().any(|t| t == lang || t == "any")
            {
                c.emit(
                    &rule.common,
                    Some(block.line),
                    "TLMD_43",
                    format!(
                        "the code block is tagged `{lang}`; expected {}",
                        text::code_list(tags, " or ")
                    ),
                );
            }
            if *fence < rule.fence {
                // sokf-implements: TLMD_145
                c.emit(
                    &rule.common,
                    Some(block.line),
                    "TLMD_145",
                    format!(
                        "the fence has {fence} backticks; at least {} are needed",
                        rule.fence
                    ),
                );
            }
        }
        // A quote reads only its paragraphs, and a heading inside it is no
        // heading of the document.
        // sokf-implements: TLMD_172
        (BlockDetail::Quote { headings }, BlockKind::Blockquote) => {
            for (line, heading) in headings {
                c.emit(
                    &rule.common,
                    Some(*line),
                    "TLMD_172",
                    format!(
                        "heading `{heading}` is inside a blockquote; a heading of the document stands outside every quote"
                    ),
                );
            }
        }
        (BlockDetail::Table { columns }, BlockKind::Table) => {
            if let Some(expected) = &rule.columns {
                let ok = if rule.columns_open {
                    columns.starts_with(expected)
                } else {
                    columns == expected
                };
                if !ok {
                    c.emit(
                        &rule.common,
                        Some(block.line),
                        "TLMD_44",
                        format!(
                            "the table header is `{}`; expected `{}`",
                            columns.join(" | "),
                            expected.join(" | ")
                        ),
                    );
                }
            }
        }
        // A region's target is matched as the host reads it: a file by its
        // path, an asset by its own. A document matches no pattern.
        // sokf-implements: TLMD_46
        // sokf-implements: TLMD_113
        // sokf-implements: TLMD_163
        (BlockDetail::Region { target }, BlockKind::Region) => {
            // sokf-implements: MD-5_AC-3
            // sokf-implements: MD-5_AC-4
            let what = rule
                .source_pattern
                .as_ref()
                .and_then(|p| match c.host.region(target) {
                    // sokf-implements: TLMD_200
                    // sokf-implements: MD-5_AC-5
                    Subject::Document => {
                        let dialect = c.host.dialect();
                        let marker = dialect.region.map(|r| dialect.spell(r)).unwrap_or_default();
                        Some(names_a_document(
                            &format!("`{marker} {target}`"),
                            p.as_str(),
                        ))
                    }
                    Subject::Text(subject) if p.is_match(&subject) => None,
                    Subject::Text(subject) if &subject == target => Some(format!(
                        "the include target `{target}` does not match `{}`",
                        p.as_str()
                    )),
                    Subject::Text(subject) => Some(format!(
                        "the include target `{target}`, read as `{subject}`, does not match `{}`",
                        p.as_str()
                    )),
                    Subject::Unknown => None,
                });
            if let Some(what) = what {
                c.emit(
                    &rule.common,
                    Some(block.line),
                    "TLMD_163",
                    format!("{what}{}", reads("target", rule.example())),
                );
            }
        }
        (_, BlockKind::Image) => {
            if let Some(p) = &rule.source_pattern {
                for unit in &block.units {
                    // An image in the host's scheme is matched by what the
                    // host says it shows, and a document matches no pattern.
                    // One the host cannot tell is the link check's finding,
                    // not this rule's.
                    // sokf-implements: TLMD_46
                    // sokf-implements: MD-5_AC-2
                    // sokf-implements: MD-5_AC-4
                    let scheme = c.host.dialect().scheme;
                    let mismatch =
                        |path: &str| format!("the image `{path}` does not match `{}`", p.as_str());
                    let what = match host_ref(&unit.text, scheme) {
                        Some(r) => match c.host.image(&r) {
                            Subject::Unknown => continue,
                            // sokf-implements: TLMD_200
                            // sokf-implements: MD-5_AC-5
                            Subject::Document => names_a_document(
                                &format!("the image `{}:{r}`", scheme.unwrap_or_default()),
                                p.as_str(),
                            ),
                            Subject::Text(path) if p.is_match(&path) => continue,
                            Subject::Text(path) => mismatch(&path),
                        },
                        None if p.is_match(&unit.text) => continue,
                        None => mismatch(&unit.text),
                    };
                    c.emit(
                        &rule.common,
                        Some(unit.line),
                        "TLMD_163",
                        format!("{what}{}", reads("target", rule.example())),
                    );
                }
            }
        }
        _ => {}
    }
    if !rule.inline_html
        && let Some(line) = block.html_lines.first()
    {
        c.emit(
            &rule.common,
            Some(*line),
            "TLMD_45",
            "inline HTML is not accepted here",
        );
    }
}

pub(super) fn check_count(c: &mut Checker<'_>, rule: &Rule, n: usize, line: usize) {
    if n < rule.min {
        // sokf-implements: TLMD_146
        c.emit(
            &rule.common,
            Some(line),
            "TLMD_146",
            format!(
                "the {} block needs at least {}; it has {n}",
                rule.kind.name_in(&c.host.dialect()),
                text::count(rule.min, unit_of(rule.kind), noun(rule.kind))
            ),
        );
    }
    if let Some(max) = rule.max
        && n > max
    {
        // sokf-implements: TLMD_160
        c.emit(
            &rule.common,
            Some(line),
            "TLMD_160",
            format!(
                "the {} block allows at most {}; it has {n}",
                rule.kind.name_in(&c.host.dialect()),
                text::count(max, unit_of(rule.kind), noun(rule.kind))
            ),
        );
    }
}

/// The frontmatter value of `key`, as text.
/// Whether a unit satisfies a rule's shape, silently: its own shape or its
/// format's, and the key it demands.
fn satisfies(c: &Checker<'_>, unit: &Unit, rule: &UnitRule) -> bool {
    let text = squash(&unit.text);
    if let Some(shape) = &rule.shape
        && !shape.matches(&text, |k| c.front_value(k))
    {
        return false;
    }
    if let Some(name) = &rule.format
        && let Some(f) = c.schema.core.formats.get(name)
        && !f.conforms(&text)
    {
        return false;
    }
    if let Some(k) = &rule.key
        && !rule.key_optional
        && rule
            .shape
            .as_ref()
            .and_then(|s| captured(s, k, &text))
            .is_none()
    {
        return false;
    }
    true
}

/// Units against a `sequence`: one rule per position, with `optional` and
/// `repeat`.
// sokf-implements: TLMD_147
// sokf-implements: TLMD_161
fn match_units(
    c: &mut Checker<'_>,
    seq: &[UnitRule],
    units: &[Unit],
    block: &Rule,
    section: Option<usize>,
    at: usize,
) {
    let mut ui = 0;
    for (pi, rule) in seq.iter().enumerate() {
        let mut count = 0;
        while let Some(unit) = units.get(ui) {
            // A required position takes its first unit and reports what
            // that unit lacks, unless the unit has the shape of a later
            // position and none of this one's: then this position is the
            // one missing, and the positions after it keep their units. A
            // unit that starts as this position's shape does is this
            // position's, written wrong.
            let take = satisfies(c, unit, rule)
                || (count == 0
                    && !rule.optional
                    && (reach(c, rule, &squash(&unit.text)) > 0
                        || !seq[pi + 1..].iter().any(|later| {
                            (later.shape.is_some() || later.format.is_some())
                                && satisfies(c, unit, later)
                        })));
            if !take {
                break;
            }
            check_unit(c, unit, rule, block, section);
            ui += 1;
            count += 1;
            if !rule.repeat {
                break;
            }
        }
        if count == 0 && !rule.optional {
            let line = units.get(ui).map(|u| u.line).unwrap_or(at);
            // sokf-implements: TLMD_147
            c.emit(
                &rule.common,
                Some(line),
                "TLMD_147",
                format!("position {} of the sequence is missing", pi + 1),
            );
        }
    }
    for unit in &units[ui.min(units.len())..] {
        // sokf-implements: TLMD_161
        c.emit(
            &block.common,
            Some(unit.line),
            "TLMD_161",
            format!("`{}` is beyond the sequence", short(&unit.text)),
        );
    }
}

/// The example a shape failure shows: the failure is built from the rule,
/// and the example is the rule's one voice.
// sokf-implements: CLI-7_AC-2
// sokf-implements: TL_74
pub(super) fn reads(what: &str, example: Option<&str>) -> String {
    example.map_or_else(String::new, |e| {
        format!("; {} {what} of this kind reads {e}", text::article(what))
    })
}

/// Where the anchor of a key whose match starts at byte `offset` of the
/// unit's text goes: before the block when a line of a code block carries
/// it, else before the match, or before the code span or link the match
/// starts inside (TLMD_105).
fn spot(c: &Checker<'_>, unit: &Unit, offset: usize) -> Spot {
    if let Some(line) = c.lines_block {
        return Spot::Before(line);
    }
    match unit.position(offset) {
        Some((line, col)) => Spot::Inline(line, c.doc.body.span_around(line, col).unwrap_or(col)),
        None => Spot::Before(unit.line),
    }
}

/// The capture a unit's key makes, filed as the checker files every key.
// sokf-implements: MD-4_AC-11
fn capture(
    c: &Checker<'_>,
    unit: &Unit,
    squashed: &Squashed,
    got: Captured,
    rule: &UnitRule,
    block: &Rule,
    section: Option<usize>,
) -> Capture {
    Capture {
        spot: spot(c, unit, squashed.original(got.start)),
        key: got.key,
        line: unit.line,
        heading_slug: None,
        scope: rule.unique.clone(),
        rule_id: rule
            .rule_id
            .clone()
            .or_else(|| block.rule_id.clone())
            .or_else(|| c.group_rule.clone()),
        section,
        pairs: rule.for_each_key_of.clone().map(|with| Paired {
            with,
            by: rule.common.line,
        }),
        repeats: rule.key_repeats,
        derives: rule.key_derives_from.clone(),
    }
}

/// The line a failure at `offset` of a unit's squashed text lies on.
fn line_at(unit: &Unit, squashed: &Squashed, offset: usize) -> usize {
    unit.position(squashed.original(offset))
        .map_or(unit.line, |(line, _)| line)
}

/// One unit against a unit rule. Every shape reads the unit's text
/// squashed, each run of whitespace one space (TL_23).
// sokf-implements: TL_23
// sokf-implements: TLMD_6
// sokf-implements: TL_30
pub(super) fn check_unit(
    c: &mut Checker<'_>,
    unit: &Unit,
    rule: &UnitRule,
    block: &Rule,
    section: Option<usize>,
) {
    // An item whose first line markdown reads as a block, as `- 1.` or
    // `- > note`, has no text to hold to its rule: the block where its text
    // should be is the fault, which its nested blocks report.
    if unit.text.is_empty() && unit.nested.first().is_some_and(|b| b.line == unit.line) {
        check_nested(c, unit, rule, section);
        return;
    }
    let squashed = Squashed::new(&unit.text);
    let text = squashed.text.as_str();
    check_text(c, unit, rule);
    let schema = c.schema;
    let format = rule.format_in(&schema.core.formats);
    let example = rule
        .example()
        .map(str::to_string)
        .or_else(|| format.map(|f| f.first_example().to_string()));
    if let Some(shape @ (Shape::Template(_) | Shape::Pattern(_))) = &rule.shape
        && !shape.matches(text, |_| None)
    {
        // sokf-implements: TL_74
        let line = match shape {
            Shape::Template(t) => line_at(unit, &squashed, t.failure(text).offset),
            _ => unit.line,
        };
        c.emit(
            &rule.common,
            Some(line),
            "TL_74",
            format!(
                "`{}` {}{}",
                short(text),
                shape.failure(text),
                reads(unit_noun(c, block.kind), rule.example())
            ),
        );
    }
    // Whether the text has the shape of its format, so that a key the rule
    // names in it is one the text should carry.
    let mut format_matched = rule.format.is_none();
    if let Some(name) = &rule.format {
        let checked = format.map(|f| f.check_as(text, unit_noun(c, block.kind)));
        format_matched = matches!(checked, Some(FormatMatch::Ok(_)));
        match checked {
            // A key the format declares is as strict as a key the rule
            // declares: captured, or reported.
            // sokf-implements: TL_44
            // sokf-implements: TL_45
            Some(FormatMatch::Ok(None))
                if format.is_some_and(|f| f.key.is_some()) && !rule.key_optional =>
            {
                c.emit(
                    &rule.common,
                    Some(unit.line),
                    "TL_44",
                    format!(
                        "`{}` carries no key: the format `{name}` captures `{}`",
                        short(text),
                        format.and_then(|f| f.key.clone()).unwrap_or_default()
                    ),
                );
            }
            Some(FormatMatch::Fail(msg)) => {
                c.emit(
                    &rule.common,
                    Some(unit.line),
                    "TL_74",
                    format!("`{}` {msg}", short(text)),
                );
            }
            // A ban quotes what it matched, as a ban of the rule does.
            Some(FormatMatch::Refused(msg)) => {
                c.emit(&rule.common, Some(unit.line), "TL_75", msg);
            }
            Some(FormatMatch::Ok(Some(got))) => {
                let cap = capture(c, unit, &squashed, got, rule, block, section);
                c.out.captures.push(cap);
            }
            _ => {}
        }
    }
    if let Some(k) = &rule.key {
        // The rule's own shape captures the key, or its format's where the
        // format names none of its own.
        let got = match (&rule.shape, &rule.format) {
            (Some(shape), _) => captured(shape, k, text),
            (None, Some(_)) => format.and_then(|f| {
                f.shape_for(text)
                    .and_then(|(shape, _)| captured(shape, k, text))
            }),
            (None, None) => None,
        };
        match got {
            Some(got) => {
                let cap = capture(c, unit, &squashed, got, rule, block, section);
                c.out.captures.push(cap);
            }
            // A text that fails its shape or its format has had its finding;
            // one that matches and captures nothing is a key the rule missed.
            // sokf-implements: TL_44
            None if !rule.key_optional
                && rule
                    .shape
                    .as_ref()
                    .is_none_or(|s| s.matches(text, |_| None))
                && format_matched =>
            {
                c.emit(
                    &rule.common,
                    Some(unit.line),
                    "TL_44",
                    format!(
                        "`{}` carries no key: `{k}` captures nothing{}",
                        short(text),
                        reads(unit_noun(c, block.kind), rule.example())
                    ),
                );
            }
            None => {}
        }
    }
    // A unit's bans read the unit's text, whatever it holds, and a failure
    // shows the rule's example beside what matched.
    // sokf-implements: TL_48
    // sokf-implements: TLMD_94
    // sokf-implements: TL_76
    if let Some(reason) = refused_as(
        text,
        &rule.prohibited_patterns,
        unit_noun(c, block.kind),
        example.as_deref(),
    ) {
        c.emit(&rule.common, Some(unit.line), "TL_75", reason);
    }
    check_nested(c, unit, rule, section);
}

/// The blocks inside an item, against the rule's `blocks`, or none.
fn check_nested(c: &mut Checker<'_>, unit: &Unit, rule: &UnitRule, section: Option<usize>) {
    match &rule.blocks {
        Some(nested) => {
            let inner: Vec<&Block> = unit.nested.iter().collect();
            check_blocks(c, Some(nested), &inner, true, section, unit.line);
        }
        None => no_nested(c, unit, &rule.common),
    }
}

/// An item whose rule declares no `blocks` holds nothing after its text.
// sokf-implements: TLMD_162
fn no_nested(c: &mut Checker<'_>, unit: &Unit, common: &RuleCommon) {
    if let Some(first) = unit.nested.first() {
        // A block that starts on the item's own line stands where its text
        // should be.
        let message = if unit.text.is_empty() && first.line == unit.line {
            format!(
                "the item's text reads as {} block",
                first.kind.spoken_in(&c.host.dialect())
            )
        } else {
            "the item carries content beyond its text".to_string()
        };
        c.emit(common, Some(first.line), "TLMD_162", message);
    }
}

/// The message for an image or a region that names a document, which no
/// `source-pattern` admits: `shown` is how the message names it.
fn names_a_document(shown: &str, pattern: &str) -> String {
    format!(
        "{shown} names a document; this rule takes what matches `{pattern}`, a file or an asset by its path"
    )
}
