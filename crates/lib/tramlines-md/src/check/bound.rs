//! A unit bound to a `text`: the literal or the frontmatter value that it
//! must read, the finding where it reads otherwise, and the repair that
//! rewrites it whole.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use super::{At, Checker, Repair, blocks, label::Read, quoted};
use crate::{
    Finding, Severity,
    grammar::{Block as Rule, Shape, Text, Unit as UnitRule, squash, task_box},
    markdown::{self, Block, BlockKind, Unit},
    text,
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
// sokf-implements: TLMD_283
// sokf-implements: TLMD_259
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
                "{} is bound to the frontmatter `{key}`, which is absent",
                quoted(&unit.text)
            ),
        );
        return;
    };
    // The label is no part of the text that the rule fixes (TLMD_248), and
    // a unit that reads as the text reads repairs nothing (TLMD_263).
    // sokf-implements: TLMD_263
    // The text reads before it is squashed, as the unit does.
    let reading = c.text_of(text).map(|t| squash(&rule.read_text(&t)));
    if reading.as_deref() == Some(read.squashed.text.as_str()) {
        // The unit's text is the value's own, so what its block refuses in
        // it is the value's fault (`TLMD_312`).
        if let Text::From(key) = text {
            record(c, unit, block, key, expected);
        }
        return;
    }
    // A required value that is empty is missing, which the check of the
    // frontmatter reports (`TL_143`): that is the one fault, and the unit
    // is left as it stands, because no text is yet known for it.
    // sokf-implements: TLMD_300
    if let Text::From(key) = text
        && c.value_missing(key)
    {
        return;
    }
    // The repair rewrites the unit whole, so it writes the unit's label
    // before the text, after the box of a task, in the host's words. A
    // label that the rule doesn't allow goes, so that the repaired unit
    // passes.
    let prefix = repair_prefix(c, unit);
    let at = match block.kind {
        BlockKind::BulletList | BlockKind::NumberedList => task_box(&expected),
        _ => 0,
    };
    let label = match block.label_of(Some(rule)) {
        Some(_) => c.label_kept(read),
        None => String::new(),
    };
    let full = format!("{}{label}{}", &expected[..at], &expected[at..]);
    // The text reads back as a unit of its kind (`TLMD_275`), or no unit of
    // its kind holds it, which a value of the frontmatter reports where it
    // stands and the rule's own `text` the check of the schema (`TLMD_276`,
    // `TLMD_277`).
    // A line of a fence or of an HTML block reads as written.
    let written = match c.lines_block {
        Some(_) => Ok(Cow::Borrowed(full.as_str())),
        None => markdown::unit_written(prefix.as_deref().unwrap_or_default(), &full),
    };
    let (criterion, message) = match (text, &written) {
        (Text::From(key), Err(why)) => {
            let line = c.doc.frontmatter.and_then(|f| f.get(key)).map(|v| v.line);
            let at = At {
                named: Some(format!("key `{key}`")),
                ..At::default()
            };
            let said = why.said(
                &format!("the frontmatter `{key}`"),
                &expected,
                blocks::unit_of(block.kind),
            );
            // One value is one fault, however many units it binds.
            let subject = format!("the frontmatter `{key}` ");
            let reported = c.out.findings.iter().any(|f| {
                f.criterion == "TLMD_276" && f.line == line && f.message.starts_with(&subject)
            });
            if !reported {
                // sokf-implements: TLMD_276
                c.located(at, |c| c.emit(&rule.common, line, "TLMD_276", said));
            }
            return;
        }
        (Text::Literal(_), _) => (
            "TL_74",
            format!("{} must read {}", quoted(&unit.text), text::code(&expected)),
        ),
        (Text::From(key), Ok(_)) => (
            "TLMD_165",
            format!(
                "{} must equal the frontmatter `{key}`: {}",
                quoted(&unit.text),
                text::code(&expected)
            ),
        ),
    };
    c.emit(&rule.common, Some(unit.line), criterion, message);
    // A unit that has the shape of another rule of its body can be that
    // rule's, so the check is not sure that the text is the unit's own.
    // sokf-implements: TLMD_280
    if c.has_other_shape(unit, rule, block) {
        return;
    }
    let written = sure_text(c, unit, block.kind, prefix.as_deref(), written.ok(), &full);
    if let (Some(prefix), Some(text)) = (prefix, written) {
        if let Some(finding) = c.out.findings.last().cloned() {
            c.settles(&finding);
        }
        c.out.repairs.push(Repair {
            line: unit.line,
            end_line: unit.end_line.max(unit.line),
            prefix,
            text,
        });
    }
}

/// The text that the repair of `unit` writes after `prefix`: `text`, or
/// `text` with one of the escapes of `markdown::escapes`, whichever reads
/// back as the unit's text with the lines above it in its container
/// unchanged, or none where neither does. The lines above that can take
/// the text as theirs are a link reference definition, which a text such as
/// `(Right)` would give a title, or make a table of, and, where `below`,
/// the marker of an item whose text starts below it (`TLMD_290`,
/// `TLMD_291`). A unit after a blank line, or after a line of no
/// definition, reads as `markdown::unit_written` judged it alone.
// sokf-implements: TLMD_290
// sokf-implements: TLMD_291
fn read_back(
    c: &Checker<'_>,
    unit: &Unit,
    prefix: &str,
    text: &str,
    full: &str,
    below: bool,
) -> Option<String> {
    let last = unit.line.checked_sub(1)?;
    let lines: Vec<&str> = c.doc.text.lines().take(last).collect();
    if lines.last().is_none_or(|l| l.trim().is_empty()) {
        return Some(text.to_string());
    }
    // The first of the lines above the text that the probe reads: the
    // item's own marker, the first line above that opens an item or whose
    // indent is narrower than the text's, or the label of the definition
    // above, within a few lines, because only the definitions of the unit's
    // block stand between them.
    let indent = |l: &str| l.len() - l.trim_start().len();
    let defined = |i: usize| c.doc.body.ref_defs.iter().any(|d| d.2 == i + 1);
    let mut first = None;
    for i in (c.doc.body_line.max(1) - 1..last).rev().take(REACH) {
        if lines[i].trim().is_empty() {
            break;
        }
        let found = if below {
            opens_item(lines[i]) || indent(lines[i]) < prefix.len()
        } else {
            defined(i)
        };
        if found {
            first = Some(i);
            break;
        }
    }
    // A unit after a line that no definition holds, such as a heading,
    // follows nothing that takes it.
    if !below && first.is_none_or(|f| !markdown::all_definitions(&lines[f..last])) {
        return Some(text.to_string());
    }
    let first = first?;
    // Every line loses the indent of the marker's line, which alone would
    // make the probe code.
    let base = &lines[first][..indent(lines[first])];
    let unindent = |l: &'_ str| -> String {
        l.strip_prefix(base)
            .unwrap_or_else(|| l.trim_start())
            .to_string()
    };
    let above = lines[first..last]
        .iter()
        .map(|l| unindent(l))
        .collect::<Vec<_>>()
        .join("\n");
    let original = prefix;
    let prefix = unindent(prefix);
    let prefix = prefix.as_str();
    let at = last - first + 1;
    let reads = |written: &str| {
        let parse = |line: &str| {
            let body = format!("{above}\n{prefix}{line}\n");
            markdown::parse(
                &body,
                1,
                &markdown::Segments::default(),
                &markdown::Dialect::default(),
            )
        };
        let (before, after) = (parse(&unit.text), parse(written));
        before.ref_defs == after.ref_defs
            && unit_at(&after.blocks, at).is_some_and(|u| u.text == written.trim())
    };
    // Each escape of the text as the rule fixes it that reads alone as a
    // unit of its kind, so the text takes one backslash.
    let alone =
        |e: &String| markdown::unit_written(original, e).is_ok_and(|w| w.as_ref() == e.as_str());
    std::iter::once(text.to_string())
        .chain(markdown::escapes(full).filter(alone))
        .find(|t| reads(t))
}

/// A unit that reads as the frontmatter value that its rule binds it to.
#[derive(Debug, Clone)]
pub(super) struct Bound {
    /// The unit's first line.
    first: usize,
    /// The unit's last line.
    last: usize,
    /// The frontmatter key.
    key: String,
    /// The value, its whitespace squashed.
    value: String,
    /// What the unit is: `paragraph` or `item`.
    noun: &'static str,
    /// The address of the block rule that checks the unit, which tells the
    /// unit's own block from a block nested in it.
    rule: usize,
    /// Whether the unit's position took its block, so that the claims
    /// around it read as they would if the block accepted the value.
    claimed: bool,
    /// The criterion, the line, and the message of each finding of the walk
    /// whose fault is the value's: a ban of the unit's rule, and the inline
    /// HTML that the unit's block refuses.
    moved: Vec<Tag>,
}

/// The criterion, the line, and the message of a finding.
type Tag = (&'static str, Option<usize>, String);

/// A finding's criterion, line, and message, as a record's tag is looked
/// up.
type TagOf<'f> = (&'f str, Option<usize>, &'f str);

/// A finding as a record's tag is looked up.
fn tag_of(f: &Finding) -> TagOf<'_> {
    (f.criterion.as_str(), f.line, f.message.as_str())
}

/// Record `unit`, of `block`, as one that reads as `value`, the value of
/// `key`, where it is a unit of prose, of a quote, or of a list.
fn record(c: &mut Checker<'_>, unit: &Unit, block: &Rule, key: &str, value: String) {
    let prose = matches!(
        block.kind,
        BlockKind::Prose | BlockKind::Blockquote | BlockKind::BulletList | BlockKind::NumberedList
    );
    if prose && c.lines_block.is_none() {
        c.bound.push(Bound {
            first: unit.line,
            last: unit.end_line.max(unit.line),
            key: key.to_string(),
            value,
            noun: blocks::unit_of(block.kind),
            rule: std::ptr::from_ref(block) as usize,
            claimed: true,
            moved: Vec::new(),
        });
    }
}

/// `records` as those of a position that refused its block, which the
/// check keeps for the findings that they mark: the position took no block,
/// whether it keeps the block or another position takes it, and the walk
/// goes on as though it were empty, so the claims around it are in doubt.
pub(super) fn released(mut records: Vec<Bound>) -> Vec<Bound> {
    release(&mut records);
    records
}

/// Mark `records` as those of a position that refused its block, as
/// `released` does.
pub(super) fn release(records: &mut [Bound]) {
    for b in records {
        b.claimed = false;
    }
}

/// The tags of the findings that `records` mark as their values' where the
/// unit's position took its block, which the check of `TLMD_279` counts as
/// faults of the frontmatter.
pub(super) fn claimed_tags(records: &[Bound]) -> HashSet<TagOf<'_>> {
    records
        .iter()
        .filter(|b| b.claimed)
        .flat_map(|b| {
            b.moved
                .iter()
                .map(|(crit, line, message)| (*crit, *line, message.as_str()))
        })
        .collect()
}

/// Mark the last finding as one whose fault is the value of the record at
/// `at`, where the check of the unit wrote a record there.
pub(super) fn moves(c: &mut Checker<'_>, at: Option<usize>) {
    let Some(f) = c.out.findings.last() else {
        return;
    };
    let tag = (crit(&f.criterion), f.line, f.message.clone());
    if let Some(b) = at.and_then(|i| c.bound.get_mut(i)) {
        b.moved.push(tag);
    }
}

/// The criteria whose finding a record can move, as `'static` names.
fn crit(criterion: &str) -> &'static str {
    if criterion == "TLMD_45" {
        "TLMD_45"
    } else {
        "TL_75"
    }
}

/// The index of the record that the check of `unit` wrote since `mark`.
pub(super) fn bound_since(c: &Checker<'_>, mark: usize, unit: &Unit) -> Option<usize> {
    (mark..c.bound.len()).find(|&i| c.bound[i].first == unit.line)
}

/// The index of the range of `ranges`, sorted and apart, that holds `line`.
fn holding(ranges: &[(usize, usize, usize)], line: usize) -> Option<usize> {
    let after = ranges.partition_point(|&(first, _, _)| first <= line);
    after
        .checked_sub(1)
        .filter(|&i| line <= ranges[i].1)
        .map(|i| ranges[i].2)
}

/// The inline HTML of `block`, which `rule` doesn't accept: `TLMD_45` at its
/// first line, and at the first line of HTML that the block holds as its
/// own and at the first of each unit of the block that reads as its bound
/// value where those differ, each of the units' marked as the value's,
/// among the records that the check of the block wrote since `mark`
/// (`TLMD_45`, `TLMD_312`). The HTML of a block nested in an item, where a
/// unit rule of `rule` declares the item's `blocks`, is the nested block's
/// to judge, so only the first line can name it, as before. The walk reads
/// the same errors as before, and the report on the values comes after it.
pub(super) fn inline_html(c: &mut Checker<'_>, block: &Block, rule: &Rule, mark: usize) {
    let me = std::ptr::from_ref(rule) as usize;
    // The block's own units that read as their values, and the blocks that
    // its items hold under rules of their own, by their lines.
    let mut values: Vec<(usize, usize, usize)> = (mark..c.bound.len())
        .filter(|&i| c.bound[i].rule == me)
        .map(|i| (c.bound[i].first, c.bound[i].last, i))
        .collect();
    values.sort_unstable();
    let mut nested: Vec<(usize, usize, usize)> = if rule.units().any(|u| u.blocks.is_some()) {
        block
            .units
            .iter()
            .flat_map(|u| &u.nested)
            .map(|b| (b.line, b.end_line.max(b.line), 0))
            .collect()
    } else {
        Vec::new()
    };
    nested.sort_unstable();
    let mut seen = HashSet::new();
    let mut own = false;
    let mut lines: Vec<(usize, Option<usize>)> = Vec::new();
    for (n, &line) in block.html_lines.iter().enumerate() {
        let at = holding(&values, line);
        let first_of_its_kind = match at {
            Some(i) => seen.insert(i),
            None => !own && holding(&nested, line).is_none(),
        };
        if n == 0 || first_of_its_kind {
            own |= at.is_none();
            lines.push((line, at));
        }
    }
    for (line, at) in lines {
        c.emit(
            &rule.common,
            Some(line),
            "TLMD_45",
            "inline HTML is not accepted here",
        );
        moves(c, at);
    }
}

/// Report on its value each finding of the walk whose fault a record marks
/// as the value's, in place of the unit's: the frontmatter dictates the
/// text, and a repair writes it back, so the writer changes the value. One
/// value is one finding, at the strongest severity of the findings that it
/// replaces, which names the first reason, whatever units it binds
/// (`TLMD_312`).
// sokf-implements: TLMD_312
pub(super) fn report_values(c: &mut Checker<'_>) {
    let bound = std::mem::take(&mut c.bound);
    // The first record that marks each tag.
    let mut owners: HashMap<TagOf<'_>, usize> = HashMap::new();
    for (i, b) in bound.iter().enumerate() {
        for (crit, line, message) in &b.moved {
            owners.entry((*crit, *line, message.as_str())).or_insert(i);
        }
    }
    // Each key with its record, the first reason, and whether an error.
    let mut faults: Vec<(&Bound, String, bool)> = Vec::new();
    let findings = std::mem::take(&mut c.out.findings);
    for f in findings {
        match owners.get(&tag_of(&f)) {
            Some(&i) => {
                let why = f
                    .message
                    .rsplit_once(" (schema ")
                    .map_or(f.message.as_str(), |(why, _)| why)
                    .to_string();
                fault(&mut faults, &bound[i], why, f.severity == Severity::Error);
            }
            None => c.out.findings.push(f),
        }
    }
    for (b, why, error) in faults {
        let line = c
            .doc
            .frontmatter
            .and_then(|f| f.get(&b.key))
            .map(|v| v.line);
        let at = At {
            named: Some(format!("key `{}`", b.key)),
            ..At::default()
        };
        let message = format!(
            "the frontmatter `{}` holds {}, which the {} bound to it can't hold: {why}",
            b.key,
            text::code(&b.value),
            b.noun
        );
        c.located(at, |c| c.error(line, "TLMD_312", message));
        if !error && let Some(f) = c.out.findings.last_mut() {
            f.severity = Severity::Warning;
        }
    }
}

/// Add `b`'s fault `why` to `faults`, one for each key, an error where any
/// of its findings is one.
fn fault<'b>(faults: &mut Vec<(&'b Bound, String, bool)>, b: &'b Bound, why: String, error: bool) {
    match faults.iter_mut().find(|(o, _, _)| o.key == b.key) {
        Some(f) => f.2 |= error,
        None => faults.push((b, why, error)),
    }
}

/// The text that the repair of `unit`, of a block of `kind`, writes after
/// `prefix`, from `written`, the text that `markdown::unit_written` writes
/// alone for `full`, the text as the rule fixes it: the text, read back
/// where a line around it can take it as its own, or none.
fn sure_text(
    c: &Checker<'_>,
    unit: &Unit,
    kind: BlockKind,
    prefix: Option<&str>,
    written: Option<Cow<'_, str>>,
    full: &str,
) -> Option<String> {
    let list = matches!(kind, BlockKind::BulletList | BlockKind::NumberedList);
    let below = list && prefix.is_some_and(|p| p.trim().is_empty());
    let prefix_or_none = prefix.unwrap_or_default();
    match written {
        // An empty item can't interrupt a paragraph, and ends at a blank
        // line, so the lines around it can take its marker or what it holds.
        Some(text) if list && text.trim().is_empty() => emptied(c, unit, prefix_or_none),
        // An item's text on its marker's line follows nothing that takes it.
        Some(text) if list && !below => Some(text.into_owned()),
        Some(text) if c.lines_block.is_none() => {
            read_back(c, unit, prefix_or_none, &text, full, below)
        }
        other => other.map(Cow::into_owned),
    }
}

/// The empty text that the repair of `unit`, a list item, writes after
/// `prefix`, or none where the lines around it would read otherwise
/// (`TLMD_291`). An empty item can't interrupt a paragraph, so the marker of
/// the first item of a list, on the line after a paragraph or a definition,
/// becomes an underline or text there, and an item that starts empty ends
/// at a blank line, so the lines that it holds after one leave it.
// sokf-implements: TLMD_291
fn emptied(c: &Checker<'_>, unit: &Unit, prefix: &str) -> Option<String> {
    let lines: Vec<&str> = c.doc.text.lines().collect();
    let line = |n: usize| n.checked_sub(1).and_then(|i| lines.get(i)).copied();
    let blank = |n: usize| line(n).is_none_or(|l| l.trim().is_empty());
    let end = unit.end_line.max(unit.line);
    let indent = |l: &str| l.len() - l.trim_start().len();
    // A line after a blank one, indented to the item's content, is the item's.
    if blank(end + 1)
        && line(end + 2).is_some_and(|l| !l.trim().is_empty() && indent(l) >= prefix.len())
    {
        return None;
    }
    if !first_of_list(&c.doc.body.blocks, unit.line) || blank(unit.line - 1) {
        return Some(String::new());
    }
    // The first item after a line with text: the two lines read alone, each
    // without the indent of the line above, say whether the item stays one.
    let above = line(unit.line - 1)?;
    let base = &above[..indent(above)];
    let marker = prefix.strip_prefix(base).unwrap_or(prefix);
    let probe = format!("{}\n{}\n", &above[base.len()..], marker.trim_end());
    let read = markdown::parse(
        &probe,
        1,
        &markdown::Segments::default(),
        &markdown::Dialect::default(),
    );
    unit_at(&read.blocks, 2)
        .is_some_and(|u| u.text.is_empty())
        .then(String::new)
}

/// Whether the unit on `line` is the first item of its list, among
/// `blocks` and the blocks that their units hold.
fn first_of_list(blocks: &[markdown::Block], line: usize) -> bool {
    blocks.iter().any(|b| {
        let list = matches!(b.kind, BlockKind::BulletList | BlockKind::NumberedList);
        (list && b.units.first().is_some_and(|u| u.line == line))
            || b.units.iter().any(|u| first_of_list(&u.nested, line))
    })
}

/// How many lines above the text of a unit the repair looks for the
/// marker or the definition that can take it.
const REACH: usize = 32;

/// Whether `line` opens a list item: `-`, `*`, or `+`, or one to nine
/// digits and `.` or `)`, after its indent, then a space, a tab, or the end.
fn opens_item(line: &str) -> bool {
    let t = line.trim_start();
    let digits = t.len() - t.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let rest = match digits {
        0 => t.strip_prefix(['-', '*', '+']),
        1..=9 => t[digits..].strip_prefix(['.', ')']),
        _ => None,
    };
    rest.is_some_and(|r| r.is_empty() || r.starts_with([' ', '\t']))
}

/// The unit of `blocks`, or of the blocks nested in their units, that
/// starts on `line`.
fn unit_at(blocks: &[markdown::Block], line: usize) -> Option<&Unit> {
    blocks.iter().flat_map(|b| &b.units).find_map(|u| {
        if u.line == line {
            Some(u)
        } else {
            unit_at(&u.nested, line)
        }
    })
}
