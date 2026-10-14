//! Sections: headings matched to declared entries, order, levels, open
//! regions, prohibited headings, bound headings, `renamed-from`, setext.

use super::{At, Capture, Checker, Occurrence, Paired, Repair, Spot, claim};
use crate::{
    grammar::{Section, Shape, Text, captured, squash},
    markdown::{Block, Heading},
};

enum Kind {
    Exact,
    Renamed(String),
    From(String),
    /// Bound to a frontmatter key the document does not set.
    Unbound(String),
}

/// The heading a section expects, as text, when it is exact.
fn expected_text(section: &Section, c: &Checker<'_>) -> Option<String> {
    match &section.heading.shape {
        Shape::Text(t) => c.text_of(t),
        _ => None,
    }
}

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

/// Whether section `k` lies inside section `j`, at any depth.
fn inside(parent_of: &[Option<usize>], mut k: usize, j: usize) -> bool {
    while let Some(p) = parent_of[k] {
        if p == j {
            return true;
        }
        k = p;
    }
    false
}

// sokf-implements: TLMD_23
fn find_match(
    h: &Heading,
    declared: &[&Section],
    seen: &[usize],
    pos: usize,
    ordered: bool,
    c: &Checker<'_>,
) -> Option<(usize, Kind)> {
    let order: Vec<usize> = (pos..declared.len()).chain(0..pos).collect();
    for &j in &order {
        if expected_text(declared[j], c).as_deref() == Some(h.text.as_str()) {
            return Some((j, Kind::Exact));
        }
    }
    // A template such as `${}` matches any heading, so only a section of the
    // heading's own level is tried here. A section of another level is tried
    // last, so that a heading at the wrong level is still claimed, and
    // reported as being at the wrong level.
    let shaped = |j: usize| {
        !declared[j].is_exact()
            && declared[j]
                .heading
                .shape
                .matches(&squash(&h.text), |_| None)
    };
    for &j in &order {
        if declared[j].level == h.level && shaped(j) {
            return Some((j, Kind::Exact));
        }
    }
    for &j in &order {
        if declared[j].renamed_from.iter().any(|r| r == &h.text) {
            let current = expected_text(declared[j], c).unwrap_or_else(|| declared[j].label());
            return Some((j, Kind::Renamed(current)));
        }
    }
    let candidates: Vec<usize> = if ordered {
        pos.min(declared.len().saturating_sub(1))..pos + 1
    } else {
        0..declared.len()
    }
    .filter(|&j| j < declared.len())
    .collect();
    for j in candidates {
        if let Shape::Text(Text::From(key)) = &declared[j].heading.shape
            && seen[j] == 0
            && declared[j].level == h.level
        {
            // A key the document does not set leaves the heading nothing
            // to equal, and the heading still stands in its section's place.
            let kind = match expected_text(declared[j], c) {
                Some(expected) => Kind::From(expected),
                None => Kind::Unbound(key.clone()),
            };
            return Some((j, kind));
        }
    }
    order
        .iter()
        .find(|&&j| declared[j].level != h.level && shaped(j))
        .map(|&j| (j, Kind::Exact))
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
                "heading `{}` lies inside `{}`, which does not apply in this document's states",
                label(h.level, &h.text),
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
                "heading `{}` lies outside every `{}` section; it belongs inside one",
                label(h.level, &h.text),
                label(parent.level, &parent.label())
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
                "heading `{}` occurs more than once under `{host_label}`",
                label(h.level, &h.text)
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
                "heading `{host_label}` carries no number for `{}` to extend",
                label(h.level, &h.text)
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
                "heading `{}` must be numbered `{want}.<n>`, under `{host_label}`",
                label(h.level, &h.text)
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
                    "section `{}` is missing under `{}`",
                    label(section.level, &section.label()),
                    label(host.level, &host.text)
                ),
            );
        });
    }
}

/// Match every heading and check every section body.
// sokf-implements: TLMD_18
// sokf-implements: TLMD_151
// sokf-implements: TLMD_19
// sokf-implements: TLMD_26
// sokf-implements: TLMD_37
// sokf-implements: TLMD_38
pub(super) fn check(c: &mut Checker<'_>) {
    let headings = &c.doc.body.headings;
    for h in headings {
        if h.setext {
            c.error(
                Some(h.line),
                "TLMD_138",
                format!("heading `{}` is setext; write it as ATX", h.text),
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
    let mut pos = 0usize;
    let mut region: Option<(u8, bool)> = None;
    let mut out_of_place = false;

    for (hi, h) in headings.iter().enumerate() {
        let next_line = headings.get(hi + 1).map(|n| n.line).unwrap_or(usize::MAX);
        let found = find_match(h, &declared, &seen, pos, ordered, c);
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
                    "heading `{}` is more than one level deeper than `{}`",
                    label(h.level, &h.text),
                    label(before.level, &before.text)
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
                    if banned.heading == h.text && banned.common.only.admits(states) {
                        c.emit(
                            &banned.common,
                            Some(h.line),
                            "TLMD_26",
                            format!("heading `{}` is prohibited", label(h.level, &h.text)),
                        );
                    }
                }
            } else {
                c.error(
                    Some(h.line),
                    "TLMD_19",
                    format!("heading `{}` is not declared", label(h.level, &h.text)),
                );
            }
            continue;
        };
        let section = declared[j];
        // Every finding of this section and of its blocks names where it
        // came from, so the schema the authority sends a reader to is
        // reachable at the rule and not at the file.
        // sokf-implements: TL_73
        c.at = At::section(section);
        c.gate(&section.common, h.line);
        // A repeating section is met again after its own subsections, which
        // the schema lists between it and its next sibling.
        // sokf-implements: TLMD_33
        // Every entry after it that the walk has passed lies inside it; a
        // repeating entry the walk rests on counts once it has been seen.
        let passed = if seen.get(pos).is_some_and(|&n| n > 0) {
            pos + 1
        } else {
            pos
        };
        let returns =
            section.repeat && j < pos && (j + 1..passed).all(|k| inside(&parent_of, k, j));
        seen[j] += 1;
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
                    "heading `{}` occurs more than once",
                    label(h.level, &h.text)
                ),
            );
        }
        // A heading that occurs again stands after the sections that follow
        // its first occurrence, which TLMD_151 has reported.
        // sokf-implements: TLMD_22
        if ordered && j < pos && !returns && !repeated && !out_of_place {
            out_of_place = true;
            c.error(
                Some(h.line),
                "TLMD_22",
                format!("heading `{}` is out of place; the sections are in the order the schema lists them", label(h.level, &h.text)),
            );
        }
        if h.level != section.level {
            // sokf-implements: TLMD_144
            c.emit(
                &section.common,
                Some(h.line),
                "TLMD_144",
                format!(
                    "heading `{}` must be level {}",
                    label(h.level, &h.text),
                    section.level
                ),
            );
        }
        if ordered {
            let next = if section.repeat { j } else { j + 1 };
            pos = if returns { next } else { pos.max(next) };
        }
        match kind {
            Kind::Exact => {}
            Kind::Renamed(current) => {
                c.emit(
                    &section.common,
                    Some(h.line),
                    "TLMD_137",
                    format!("heading `{}` was renamed; it is now `{}`", h.text, current),
                );
                c.repair_heading(Repair::heading(h.line, h.end_line, section.level, current));
            }
            Kind::From(expected) => {
                let key = match &section.heading.shape {
                    Shape::Text(Text::From(k)) => k.clone(),
                    _ => String::new(),
                };
                // sokf-implements: TLMD_165
                c.emit(
                    &section.common,
                    Some(h.line),
                    "TLMD_165",
                    format!(
                        "heading `{}` must equal the frontmatter `{key}`: `{expected}`",
                        h.text
                    ),
                );
                c.repair_heading(Repair::heading(h.line, h.end_line, section.level, expected));
            }
            Kind::Unbound(key) => {
                if c.unbound_is_own_fault(&key) {
                    c.emit(
                        &section.common,
                        Some(h.line),
                        "TLMD_168",
                        format!(
                            "heading `{}` is bound to the frontmatter `{key}`, which is absent",
                            label(h.level, &h.text)
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
        if let Some(name) = &section.heading.key
            && let Some(got) = captured(&section.heading.shape, name, &squash(&h.text))
        {
            // sokf-implements: MD-4_AC-11
            c.out.captures.push(Capture {
                key: got.key,
                line: h.line,
                heading_slug: Some(h.slug.clone()),
                scope: section.heading.unique.clone(),
                rule_id: section.rule_id.clone(),
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
        claim::check_blocks(
            c,
            section.blocks.as_deref(),
            &body,
            section.blocks_ordered,
            Some(j),
            h.line,
        );
        c.at = At::default();
    }

    // A section's `rule-id` names the keys captured within it.
    for cap in &mut c.out.captures {
        if cap.rule_id.is_none()
            && let Some(j) = cap.section
        {
            cap.rule_id = declared[j].rule_id.clone();
        }
    }

    for (j, section) in declared.iter().enumerate() {
        if section.parent.is_some() {
            if let Some(pj) = parent_of[j]
                && !section.optional
                && section.for_each_key_of.is_none()
            {
                missing_inside(c, section, j, pj);
            }
            continue;
        }
        if seen[j] == 0 && !section.optional && section.for_each_key_of.is_none() {
            c.located(At::section(section), |c| {
                c.emit(
                    &section.common,
                    None,
                    "TLMD_18",
                    format!(
                        "section `{}` is missing",
                        label(section.level, &section.label())
                    ),
                );
            });
        }
    }

    // The preamble: everything before the first heading.
    let first = headings.first().map(|h| h.line).unwrap_or(usize::MAX);
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
            claim::check_blocks(
                c,
                p.blocks.as_deref(),
                &pre,
                p.blocks_ordered,
                None,
                c.doc.body_line,
            );
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
