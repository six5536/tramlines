//! Sections: headings matched to declared entries, order, levels, open
//! regions, prohibited headings, bound headings, `renamed-from`, setext.

use super::{
    At, Capture, Checker, Occurrence, Paired, Repair, Spot, claim,
    headings::{Kind, inside, walk},
    keys,
};
use crate::{
    grammar::{Section, Shape, Squashed, Text, captured, read},
    markdown::{Block, Heading},
    text,
};

fn label(level: u8, text: &str) -> String {
    format!("{} {text}", "#".repeat(usize::from(level)))
}

/// The number a heading begins with: the text before the first space,
/// digits separated by dots.
fn number(text: &str) -> Option<&str> {
    let first = text.split(' ').next()?;
    first
        .split('.')
        .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        .then_some(first)
}

/// A heading of a section that lies inside a parent: after an occurrence
/// of the parent, as often as the section admits there, and numbered from
/// the parent's heading where the schema says so. Whether it repeats
/// under one occurrence of the parent.
// sokf-implements: TLMD_31
// sokf-implements: TLMD_32
// sokf-implements: TLMD_151
// sokf-implements: TLMD_166
// sokf-implements: TLMD_35
// sokf-implements: TLMD_36
fn placed(
    c: &mut Checker<'_>,
    declared: &[&Section],
    parent_of: &[Option<usize>],
    above: &[Heading],
    j: usize,
    h: &Heading,
) -> bool {
    let section = declared[j];
    // A parent the document's states exclude has no occurrence to lie in.
    let Some(pj) = parent_of[j] else {
        c.emit(
            &section.common,
            Some(h.line),
            "TLMD_166",
            format!(
                "heading {} lies inside `{}`, which does not apply in this document's states",
                text::code(&label(h.level, &h.text)),
                section.parent.clone().unwrap_or_default()
            ),
        );
        return false;
    };
    let parent = declared[pj];
    let Some(host) = host_of(c, parent, pj, above) else {
        c.emit(
            &section.common,
            Some(h.line),
            "TLMD_31",
            format!(
                "heading {} lies outside every {} section; it belongs inside one",
                text::code(&label(h.level, &h.text)),
                text::code(&label(parent.level, &parent.label()))
            ),
        );
        return false;
    };
    let host_label = label(host.level, &host.text);
    let span = c.span(&host);
    let before = c
        .occurrences
        .iter()
        .filter(|o| o.section == j && span.contains(&o.line))
        .count();
    let repeated = before > 0 && !section.repeat;
    if repeated {
        c.emit(
            &section.common,
            Some(h.line),
            "TLMD_151",
            format!(
                "heading {} occurs more than once under {}",
                text::code(&label(h.level, &h.text)),
                text::code(&host_label)
            ),
        );
    }
    if !section.number_from_parent {
        return repeated;
    }
    let Some(want) = number(&host.text) else {
        c.emit(
            &section.common,
            Some(host.line),
            "TLMD_36",
            format!(
                "heading {} carries no number for {} to extend",
                text::code(&host_label),
                text::code(&label(h.level, &h.text))
            ),
        );
        return repeated;
    };
    let extends = number(&h.text)
        .and_then(|n| n.strip_prefix(want))
        .and_then(|rest| rest.strip_prefix('.'))
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()));
    if !extends {
        c.emit(
            &section.common,
            Some(h.line),
            "TLMD_35",
            format!(
                "heading {} must be numbered `{want}.<n>`, under {}",
                text::code(&label(h.level, &h.text)),
                text::code(&host_label)
            ),
        );
    }
    repeated
}

/// The occurrence of the parent `pj` that a heading after `above` lies
/// inside: the nearest heading above of the parent's level or higher,
/// where it is an occurrence of the parent.
fn host_of(c: &Checker<'_>, parent: &Section, pj: usize, above: &[Heading]) -> Option<Occurrence> {
    above
        .iter()
        .rev()
        .find(|o| o.level <= parent.level)
        .and_then(|o| {
            c.occurrences
                .iter()
                .find(|occ| occ.line == o.line && occ.section == pj)
                .cloned()
        })
}

/// A section demanded inside its parent is missing from an occurrence of
/// the parent that does not hold it.
// sokf-implements: TLMD_18
// sokf-implements: TLMD_32
fn missing_inside(c: &mut Checker<'_>, section: &Section, j: usize, pj: usize) {
    let hosts: Vec<Occurrence> = c
        .occurrences
        .iter()
        .filter(|o| o.section == pj)
        .cloned()
        .collect();
    for host in hosts {
        let span = c.span(&host);
        if c.occurrences
            .iter()
            .any(|o| o.section == j && span.contains(&o.line))
        {
            continue;
        }
        c.located(At::section(section), |c| {
            c.emit(
                &section.common,
                Some(host.line),
                "TLMD_18",
                format!(
                    "section {} is missing under {}",
                    text::code(&label(section.level, &section.label())),
                    text::code(&label(host.level, &host.text))
                ),
            );
        });
    }
}

/// Record that `h` breaks its rule where the match of the heading, from
/// the finding `reported` on, reports a finding on its line other than
/// those at the indices `corrected`, which the repair that rewrites the
/// heading corrects. A finding of the match on another line, such as on a
/// parent's heading, is not about `h`'s text, and a heading that captures
/// its key holds that key's anchor, so it holds no other. Whether it
/// recorded the heading.
// sokf-implements: TLMD_109
fn broke_heading(c: &mut Checker<'_>, h: &Heading, reported: usize, corrected: &[usize]) -> bool {
    let captures = c
        .out
        .captures
        .iter()
        .any(|cap| cap.line == h.line && cap.heading_slug.is_some());
    let on_heading = c.out.findings[reported..]
        .iter()
        .enumerate()
        .any(|(i, f)| f.line == Some(h.line) && !corrected.contains(&(reported + i)));
    let broke = on_heading && !captures;
    if broke {
        c.broke_heading(h);
    }
    broke
}

/// Report that `h` is to be rewritten to `text`, the text of its section,
/// under `criterion`, and record the repair that writes it so at the
/// section's level. The findings that the repair corrects, by their
/// indices: its own, and `level`, the finding of the heading's level,
/// where there is one, because the repair writes the section's level too.
// sokf-implements: TLMD_137
fn rewrite_heading(
    c: &mut Checker<'_>,
    section: &Section,
    h: &Heading,
    level: Option<usize>,
    (criterion, message): (&str, String),
    text: String,
) -> Vec<usize> {
    let own = c.out.findings.len();
    c.emit(&section.common, Some(h.line), criterion, message);
    c.repair_heading(Repair::heading(h.line, h.end_line, section.level, text));
    std::iter::once(own).chain(level).collect()
}

/// Match every heading and check every section body.
// sokf-implements: TLMD_18
// sokf-implements: TLMD_151
// sokf-implements: TLMD_19
// sokf-implements: TLMD_26
// sokf-implements: TLMD_37
// sokf-implements: TLMD_38
#[expect(
    clippy::too_many_lines,
    reason = "one pass of the check, a split of which is a refactor of its own"
)]
pub(super) fn check(c: &mut Checker<'_>) {
    let headings = &c.doc.body.headings;
    for h in headings {
        if h.setext {
            c.error(
                Some(h.line),
                "TLMD_138",
                format!("heading {} is setext; write it as ATX", text::code(&h.text)),
            );
            c.out
                .repairs
                .push(Repair::heading(h.line, h.end_line, h.level, h.text.clone()));
        }
    }

    let schema = c.schema;
    let states = c.states;
    let declared = c.sections.clone();
    let ordered = schema.sections_ordered;
    let parent_of = c.parent_of.clone();
    let mut seen = vec![0usize; declared.len()];
    // Each heading's section, from the walk that a host shares (TLMD_23).
    // sokf-implements: MD-15_AC-2
    let value = |key: &str| c.front_value(key);
    let matches = walk(headings, &declared, &parent_of, ordered, &value);
    let mut region: Option<(u8, bool)> = None;
    // The section of the last heading that a declared section took.
    let mut last: Option<usize> = None;

    for (hi, h) in headings.iter().enumerate() {
        let next_line = headings.get(hi + 1).map_or(usize::MAX, |n| n.line);
        // The findings of the match of the heading start here.
        let reported = c.out.findings.len();
        let found = matches[hi].clone();
        // A heading at another level than its section declares is that
        // fault (TLMD_144), whatever level it skips to, and a subsection
        // outside every occurrence of its parent (TLMD_31) skips the level
        // of the parent it lacks.
        // sokf-implements: TLMD_20
        let outside = found.as_ref().is_some_and(|(j, _)| {
            parent_of[*j].is_some_and(|pj| host_of(c, declared[pj], pj, &headings[..hi]).is_none())
        });
        if let Some(before) = hi.checked_sub(1).map(|b| &headings[b])
            && h.level > before.level + 1
            && !outside
            && found
                .as_ref()
                .is_none_or(|(j, _)| declared[*j].level == h.level)
        {
            c.error(
                Some(h.line),
                "TLMD_20",
                format!(
                    "heading {} is more than one level deeper than {}",
                    text::code(&label(h.level, &h.text)),
                    text::code(&label(before.level, &before.text))
                ),
            );
        }
        let Some((j, kind)) = found else {
            let admitted = match region {
                Some((level, deeper)) => {
                    if deeper {
                        h.level > level
                    } else {
                        h.level >= level
                    }
                }
                None => false,
            };
            if admitted {
                for banned in &schema.sections_prohibited {
                    // sokf-implements: TLMD_258
                    if banned.read == h.reading.or_written(&h.text).text
                        && banned.common.only.admits(states)
                    {
                        c.emit(
                            &banned.common,
                            Some(h.line),
                            "TLMD_26",
                            format!(
                                "heading {} is prohibited",
                                text::code(&label(h.level, &h.text))
                            ),
                        );
                    }
                }
            } else {
                // A heading that the region does not accept ends it: one of
                // a higher level after `headings-open`, or one of the
                // section's level or higher after `subheadings-open`, as such
                // a heading ends a section in the outline, so the headings
                // under it are undeclared too.
                // sokf-implements: TLMD_24
                // sokf-implements: TLMD_25
                region = None;
                c.error(
                    Some(h.line),
                    "TLMD_19",
                    format!(
                        "heading {} is not declared",
                        text::code(&label(h.level, &h.text))
                    ),
                );
            }
            // The units of the body break their rules too while the
            // heading does, because the check judges none of them.
            if broke_heading(c, h, reported, &[]) {
                for block in c.doc.body.blocks_between(h.line, next_line) {
                    c.broke_block(block);
                }
            }
            continue;
        };
        let section = declared[j];
        // Every finding of this section and of its blocks names where it
        // came from, so the schema the authority sends a reader to is
        // reachable at the rule and not at the file.
        // sokf-implements: TL_73
        c.at = At::section(section);
        // sokf-implements: MD-4_AC-20
        c.gate(&section.common, h.line);
        seen[j] += 1;
        let previous = last.replace(j);
        let repeated = if section.parent.is_some() {
            placed(c, &declared, &parent_of, &headings[..hi], j, h)
        } else {
            seen[j] > 1 && !section.repeat
        };
        if repeated && section.parent.is_none() {
            c.emit(
                &section.common,
                Some(h.line),
                "TLMD_151",
                format!(
                    "heading {} occurs more than once",
                    text::code(&label(h.level, &h.text))
                ),
            );
        }
        // A heading that occurs again stands after the sections that follow
        // its first occurrence, which TLMD_151 has reported. A heading is
        // out of place against the heading before it, so the subsections of
        // a parent out of place, which follow the parent in order, are not,
        // and neither is a repeating section met again after its own.
        // sokf-implements: TLMD_22
        let back = section.repeat
            && previous.is_some_and(|l| (j + 1..=l).all(|k| inside(&parent_of, k, j)));
        if ordered && previous.is_some_and(|l| j < l) && !back && !repeated {
            c.error(
                Some(h.line),
                "TLMD_22",
                format!(
                    "heading {} is out of place; the sections are in the order the schema lists them",
                    text::code(&label(h.level, &h.text))
                ),
            );
        }
        let level = (h.level != section.level).then_some(c.out.findings.len());
        if level.is_some() {
            // sokf-implements: TLMD_144
            c.emit(
                &section.common,
                Some(h.line),
                "TLMD_144",
                format!(
                    "heading {} must be level {}",
                    text::code(&label(h.level, &h.text)),
                    section.level
                ),
            );
        }
        // The findings that the repair of the heading corrects: it rewrites
        // a renamed heading, or one bound to the frontmatter, to the text
        // of its section, at the section's level.
        let mut corrected = Vec::new();
        // Whether the heading is not surely the bound heading, so that the
        // blocks under it are not surely its section's either.
        let mut doubtful = false;
        match kind {
            Kind::Exact(_) | Kind::Shaped => {}
            Kind::Renamed(current) => {
                let message = format!(
                    "heading {} was renamed; it is now {}",
                    text::code(&h.text),
                    text::code(&current)
                );
                corrected = rewrite_heading(c, section, h, level, ("TLMD_137", message), current);
            }
            Kind::From(expected) => {
                let key = match &section.heading.shape {
                    Shape::Text(Text::From(k)) => k.clone(),
                    _ => String::new(),
                };
                // A required value that is empty is missing (`TL_143`), the
                // one fault, and the heading stays as it stands.
                // sokf-implements: TLMD_300
                let missing = c.value_missing(&key);
                let message = format!(
                    "heading {} must equal the frontmatter `{key}`: {}",
                    text::code(&h.text),
                    text::code(&expected)
                );
                // A heading that an open region admits, or that stands beside
                // another heading which reads as the value, can be other than
                // the bound heading, so the check reports it and writes
                // nothing.
                // sokf-implements: TLMD_285
                let admitted =
                    region.is_some_and(
                        |(at, deeper)| {
                            if deeper { h.level > at } else { h.level >= at }
                        },
                    );
                let value = read(&expected);
                let elsewhere = headings
                    .iter()
                    .any(|o| o.line != h.line && o.reading.or_written(&o.text).text == value);
                doubtful = admitted || elsewhere;
                if doubtful && !missing {
                    // sokf-implements: TLMD_165
                    c.emit(&section.common, Some(h.line), "TLMD_165", message);
                } else if !missing {
                    // sokf-implements: TLMD_165
                    // sokf-implements: TLMD_283
                    corrected =
                        rewrite_heading(c, section, h, level, ("TLMD_165", message), expected);
                }
            }
            Kind::Unbound(key) => {
                if c.unbound_is_own_fault(&key) {
                    c.emit(
                        &section.common,
                        Some(h.line),
                        "TLMD_168",
                        format!(
                            "heading {} is bound to the frontmatter `{key}`, which is absent",
                            text::code(&label(h.level, &h.text))
                        ),
                    );
                }
            }
        }
        // sokf-implements: TLMD_24
        // sokf-implements: TLMD_25
        region = if section.headings_open {
            Some((section.level, false))
        } else if section.subheadings_open {
            Some((section.level, true))
        } else {
            None
        };
        let reading = h.reading.or_written(&h.text);
        let squashed = Squashed::mapped(&reading.text, &reading.map);
        if let Some(name) = &section.heading.key
            && let Some(got) = captured(&section.heading.read, name, &squashed.text)
        {
            let at = keys::key_at(&section.heading.read, name, &squashed.text);
            // sokf-implements: MD-4_AC-11
            // sokf-implements: MD-4_AC-18
            c.out.captures.push(Capture {
                written: crate::grammar::written_key(&h.text, &reading, &squashed, at, &got.key),
                key: got.key,
                line: h.line,
                heading_slug: Some(h.slug.clone()),
                anchor: String::new(),
                scope: section.heading.unique.clone(),
                rule_id: section.rule_id.clone(),
                rule_name: section.rule_id.clone(),
                section: Some(j),
                pairs: section.for_each_key_of.clone().map(|with| Paired {
                    with,
                    by: section.common.line,
                }),
                repeats: false,
                spot: Spot::Heading(h.line),
                derives: section.heading.key_derives_from.clone(),
            });
        }
        broke_heading(c, h, reported, &corrected);
        let occurrence = Occurrence {
            section: j,
            line: h.line,
            level: h.level,
            text: h.text.clone(),
        };
        // A body that repeats with its heading repeats the keys of the
        // body it repeats. Each body runs to the next heading of its level
        // or higher, the level the section declares where the heading is
        // written higher.
        if repeated && let Some(first) = c.occurrences.iter().rev().find(|o| o.section == j) {
            let original = c.span_from(first.line, first.level.max(section.level));
            let again = c.span_from(h.line, h.level.max(section.level));
            c.repeats.push((original, again));
        }
        c.occurrences.push(occurrence);
        let body: Vec<&Block> = c.doc.body.blocks_between(h.line, next_line);
        // A repair of a fixed text writes only where the check is sure of
        // each claim of the body (TLMD_279, TLMD_280).
        c.settled.clear();
        let start = c.save();
        c.body = section.blocks.as_deref().unwrap_or_default();
        claim::check_blocks(
            c,
            section.blocks.as_deref(),
            &body,
            section.blocks_ordered,
            Some(j),
            h.line,
        );
        c.keep_sure_repairs(&start, &body, doubtful);
        c.body = &[];
        c.at = At::default();
    }

    // A section's `rule-id` names the keys captured within it, and the
    // rule that captured each where nothing nearer names it.
    for cap in &mut c.out.captures {
        if let Some(j) = cap.section {
            if cap.rule_id.is_none() {
                cap.rule_id.clone_from(&declared[j].rule_id);
            }
            if cap.rule_name.is_none() {
                cap.rule_name.clone_from(&declared[j].rule_id);
            }
        }
    }

    for (j, section) in declared.iter().enumerate() {
        if section.parent.is_some() {
            if let Some(pj) = parent_of[j]
                && !c.section_optional(section)
                && section.for_each_key_of.is_none()
            {
                missing_inside(c, section, j, pj);
            }
            continue;
        }
        if seen[j] == 0 && !c.section_optional(section) && section.for_each_key_of.is_none() {
            c.located(At::section(section), |c| {
                c.emit(
                    &section.common,
                    None,
                    "TLMD_18",
                    format!(
                        "section {} is missing",
                        text::code(&label(section.level, &section.label()))
                    ),
                );
            });
        }
    }

    // The preamble: everything before the first heading.
    let first = headings.first().map_or(usize::MAX, |h| h.line);
    let pre: Vec<&Block> = c
        .doc
        .body
        .blocks
        .iter()
        .filter(|b| b.line < first)
        .collect();
    match &schema.preamble {
        Some(p) if p.common.only.admits(states) => {
            c.at = At {
                region: Some("preamble".to_string()),
                line: Some(p.common.line),
                ..At::default()
            };
            c.settled.clear();
            let start = c.save();
            c.body = p.blocks.as_deref().unwrap_or_default();
            claim::check_blocks(
                c,
                p.blocks.as_deref(),
                &pre,
                p.blocks_ordered,
                None,
                c.doc.body_line,
            );
            c.keep_sure_repairs(&start, &pre, false);
            c.body = &[];
            c.at = At::default();
        }
        _ => {
            if let Some(b) = pre.first() {
                c.error(
                    Some(b.line),
                    "TLMD_38",
                    "content before the first heading, with no preamble declared",
                );
            }
        }
    }
}
