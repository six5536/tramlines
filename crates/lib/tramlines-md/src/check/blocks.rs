//! What a block and its units must satisfy: sequence, nested blocks,
//! per-kind keys. Which block a position gets is `claim`.

use std::{borrow::Cow, ops::Range};

use super::{
    Capture, Checker, Paired, Spot, Subject,
    bound::check_text,
    claim::check_blocks,
    rules::{check_lines, check_rules, reach},
    short,
};
use crate::{
    grammar::{
        Block as Rule, Captured, FormatMatch, Lang, RuleCommon, Shape, Squashed, Unit as UnitRule,
        captured, refused_masked,
    },
    markdown::{self, Block, BlockDetail, BlockKind, Unit, host_ref},
    text,
};

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
            // A unit of a sequence has its label judged by its position.
            check_unit(c, unit, each, rule, section, rule.sequence.is_none());
        }
    }
    if let Some(rules) = &rule.rules {
        check_rules(c, block, rules, rule, section);
    }
    // A list with no unit rule declares no `blocks` for its items either.
    // A unit of such a block carries the label that the block declares,
    // and no other (TLMD_242).
    if rule.each.is_none() && rule.sequence.is_none() && rule.rules.is_none() {
        for unit in &block.units {
            let read = c.read(unit, rule, None);
            if c.check_label(unit, rule, None, &read, true) {
                c.broke(unit);
            }
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
    // A soft limit is never checked.
    if let Some(limit) = &rule.max
        && let Some(max) = limit.hard()
        && n > max
    {
        // sokf-implements: TLMD_160
        c.emit(
            &rule.common,
            Some(line),
            "TLMD_160",
            text::at_limit(
                format!(
                    "the {} block allows at most {}; it has {n}",
                    rule.kind.name_in(&c.host.dialect()),
                    text::count(max, unit_of(rule.kind), noun(rule.kind))
                ),
                &limit.when_at_limit,
            ),
        );
    }
}

/// Whether a unit of `block` satisfies a rule's shape, silently: its own
/// shape or its format's, and the key it demands, read without the unit's
/// label.
fn satisfies(c: &Checker<'_>, unit: &Unit, rule: &UnitRule, block: &Rule) -> bool {
    let read = c.read(unit, block, Some(rule));
    let squashed = &read.squashed;
    let text = squashed.text.as_str();
    if let Some(shape) = &rule.shape
        && !shape.matches(text, |k| c.front_value(k))
    {
        return false;
    }
    if let Some(name) = &rule.format
        && let Some(f) = c.schema.core.formats.get(name)
        && !(if f.prohibited.is_empty() {
            f.conforms(text)
        } else {
            f.conforms_masked(text, &ban_text(c, unit, squashed, rule))
        })
    {
        return false;
    }
    if let Some(k) = &rule.key
        && !rule.key_optional
        && rule
            .shape
            .as_ref()
            .and_then(|s| captured(s, k, text))
            .is_none()
    {
        return false;
    }
    true
}

/// The text that the bans of `rule` and of its format read: `squashed`,
/// the squashed text of `unit`, with its code spans masked where a ban
/// reads it.
fn ban_text<'a>(
    c: &Checker<'_>,
    unit: &Unit,
    squashed: &'a Squashed,
    rule: &UnitRule,
) -> Cow<'a, str> {
    if !rule.bans_in(&c.schema.core.formats) {
        return Cow::Borrowed(&squashed.text);
    }
    markdown::mask_code(&squashed.text, &code_of(c, unit, squashed))
}

/// What a ban leaves out of `squashed`, the squashed text of `unit`: the
/// byte ranges that lie in a code span, in order.
// sokf-implements: TLMD_111
fn code_of(c: &Checker<'_>, unit: &Unit, squashed: &Squashed) -> Vec<Range<usize>> {
    let code = c.doc.body.code_in(unit);
    let mut out: Vec<Range<usize>> = Vec::new();
    if code.is_empty() {
        return out;
    }
    // Both run in the order of the text, so one pass walks them together.
    let mut k = 0;
    for i in 0..squashed.text.len() {
        let at = squashed.original(i);
        while code.get(k).is_some_and(|r| r.end <= at) {
            k += 1;
        }
        if code.get(k).is_none_or(|r| r.start > at) {
            continue;
        }
        match out.last_mut() {
            Some(r) if r.end == i => r.end = i + 1,
            _ => out.push(i..i + 1),
        }
    }
    out
}

/// Whether a unit's squashed `text` has the shape of `rule`: its own
/// template or pattern, and a shape of its format, whatever else the rule
/// reports, its bans and its key among them.
pub(super) fn has_shape(c: &Checker<'_>, text: &str, rule: &UnitRule) -> bool {
    rule.shape
        .as_ref()
        .is_none_or(|s| s.matches(text, |k| c.front_value(k)))
        && rule
            .format_in(&c.schema.core.formats)
            .is_none_or(|f| f.shape_for(text).is_some())
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
            let take = satisfies(c, unit, rule, block)
                || (count == 0
                    && !rule.optional
                    && (reach(c, rule, &c.read(unit, block, Some(rule)).squashed.text) > 0
                        || !seq[pi + 1..].iter().any(|later| {
                            (later.shape.is_some() || later.format.is_some())
                                && satisfies(c, unit, later, block)
                        })));
            if !take {
                break;
            }
            check_unit(c, unit, rule, block, section, true);
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
        c.broke(unit);
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
            .files_under(&[block.rule_id.as_deref(), c.group_rule.as_deref()])
            .map(str::to_string),
        rule_name: rule
            .named_by(&[block.rule_id.as_deref(), c.group_rule.as_deref()])
            .map(str::to_string),
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
/// Where several rules check the unit, only the one that takes it judges
/// its label, which `judges_label` says.
pub(super) fn check_unit(
    c: &mut Checker<'_>,
    unit: &Unit,
    rule: &UnitRule,
    block: &Rule,
    section: Option<usize>,
    judges_label: bool,
) {
    // An item whose first line markdown reads as a block, as `- 1.` or
    // `- > note`, has no text to hold to its rule: the block where its text
    // should be is the fault, which its nested blocks report.
    if unit.text.is_empty() && unit.nested.first().is_some_and(|b| b.line == unit.line) {
        check_nested(c, unit, rule, section);
        return;
    }
    // Every reading of the unit by its rule leaves out its label.
    // sokf-implements: TLMD_248
    let read = c.read(unit, block, Some(rule));
    let squashed = &read.squashed;
    let text = squashed.text.as_str();
    let reported = c.out.findings.len();
    // Every ban of the rule and of its format reads this text.
    let masked = ban_text(c, unit, squashed, rule);
    if c.records_accepted && accepts(c, unit_noun(c, block.kind), text, rule) {
        c.accept(rule, unit.line, unit.end_line.max(unit.line));
    }
    check_text(c, unit, rule, block, &read);
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
            Shape::Template(t) => line_at(unit, squashed, t.failure(text).offset),
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
        // A format's bans leave out code spans, as the schema's do.
        // sokf-implements: TLMD_111
        let checked = format.map(|f| f.check_masked(text, &masked, unit_noun(c, block.kind)));
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
                let cap = capture(c, unit, squashed, got, rule, block, section);
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
                let cap = capture(c, unit, squashed, got, rule, block, section);
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
    // A unit's bans read the unit's text with its code spans masked, as a
    // format's do, and a failure shows the rule's example beside what
    // matched, quoted from the text.
    // sokf-implements: TL_48
    // sokf-implements: TLMD_94
    // sokf-implements: TL_76
    // sokf-implements: TLMD_111
    if let Some(reason) = refused_masked(
        text,
        &masked,
        &rule.prohibited_patterns,
        unit_noun(c, block.kind),
        example.as_deref(),
    ) {
        c.emit(&rule.common, Some(unit.line), "TL_75", reason);
    }
    // A unit that fails its shape has its finding, so a label that it lacks
    // is not one more (TLMD_249). A label that its repair rewrites breaks
    // no rule, as a unit that its repair rewrites doesn't.
    let broke = c.out.findings.len() > reported;
    let by_label = judges_label && {
        let shaped = has_shape(c, text, rule);
        c.check_label(unit, block, Some(rule), &read, shaped)
    };
    // The blocks the item holds are units of their own, which their own
    // checks record.
    if broke || by_label {
        c.broke(unit);
    }
    check_nested(c, unit, rule, section);
}

/// Whether `rule` accepts a unit, the `noun` of its kind, whose squashed
/// text is `text`: a paragraph, an item, or a row, never a line of a fence,
/// that has the rule's shape, its own and its format's, whatever else the
/// rule reports, the bans of the rule and of its format among them.
// sokf-implements: TLMD_212
fn accepts(c: &Checker<'_>, noun: &str, text: &str, rule: &UnitRule) -> bool {
    matches!(noun, "paragraph" | "item" | "row") && has_shape(c, text, rule)
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
        // No rule takes what the item holds, so no check reads it either,
        // and a repair of the item's text leaves it.
        c.broke_beyond_text(unit);
        for inner in &unit.nested {
            c.broke_block(inner);
        }
    }
}

/// The message for an image or a region that names a document, which no
/// `source-pattern` admits: `shown` is how the message names it.
fn names_a_document(shown: &str, pattern: &str) -> String {
    format!(
        "{shown} names a document; this rule takes what matches `{pattern}`, a file or an asset by its path"
    )
}
