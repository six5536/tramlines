//! The match of each heading of a body to a declared section, which the
//! section walk and a host share (`TLMD_23`): by the reading of its exact
//! text, its shape, a `renamed-from`, the value of the frontmatter, or its
//! place in the place of a heading bound to the frontmatter.

use serde::{Deserialize, Serialize};

use super::{
    DocView,
    checker::{parents, possible},
};
use crate::{
    grammar::{Schema, Section, Shape, States, Text, read, squash},
    markdown::{self, Heading},
};

// sokf:begin match-headings
/// How the check matched a heading to its section.
///
/// A later release can add a way of matching, so a `match` on it has a
/// wildcard arm. A variant never changes what it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MatchedBy {
    /// Its reading equals the reading of the exact heading of the section.
    Text,
    /// It has the template or the pattern of the section's heading.
    Shape,
    /// Its reading equals the reading of a `renamed-from` of the section,
    /// which the check repairs to the section's heading.
    Renamed,
    /// The section's heading is the value of a frontmatter key, and the
    /// heading's reading equals the reading of the value.
    Frontmatter,
    /// The section's heading is the value of a frontmatter key, and the
    /// heading stands in its place, at the section's level, but reads
    /// otherwise, which the check repairs to the value (`TLMD_165`), or
    /// the document gives the key no scalar value, such as where the key
    /// is absent. The heading holds the section by its place alone.
    Place,
}

/// A heading of a document matched to a section of its schema.
///
/// The section can be of another level than the heading, which the check
/// reports (`TLMD_144`): a host that needs the heading at its section's
/// level compares `Heading::level` with `Section::level`. A later release
/// can add a fact of the match, so a host builds one with
/// [`HeadingMatch::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HeadingMatch {
    /// The index of the section in `Schema::sections`.
    pub section: usize,
    /// How the check matched the heading.
    pub by: MatchedBy,
}

impl HeadingMatch {
    /// The match of a heading to the section at index `section` of
    /// `Schema::sections`, matched `by`, for a host's own tests.
    pub fn new(section: usize, by: MatchedBy) -> Self {
        HeadingMatch { section, by }
    }
}

/// Each heading of the body of `doc`, in order, matched to the section of
/// `schema` that the check matches it to at `states`, or `None` where no
/// declared section takes it.
///
/// The match can give a section a heading at another level: by its exact
/// text, a `renamed-from`, or the value of the frontmatter at any level,
/// and by its shape where no section of the heading's level takes it, and
/// the check reports such a heading (`TLMD_144`). It says nothing of a heading out of place or repeated, which the section
/// walk of the check reports.
// sokf-implements: MD-15_AC-1
// sokf-implements: MD-15_AC-3
pub fn match_headings(
    doc: DocView<'_>,
    schema: &Schema,
    states: &States,
) -> Vec<Option<HeadingMatch>> {
    // sokf:end match-headings
    let possible = possible(states, &schema.core.lifecycle);
    let admitted: Vec<usize> = (0..schema.sections.len())
        .filter(|&j| schema.sections[j].common.only.admits(&possible))
        .collect();
    let declared: Vec<&Section> = admitted.iter().map(|&j| &schema.sections[j]).collect();
    let parent_of = parents(&declared);
    let value = |key: &str| {
        doc.frontmatter
            .and_then(|f| f.get(key))
            .and_then(tramlines_core::Spanned::scalar_text)
    };
    walk(
        &doc.body.headings,
        &declared,
        &parent_of,
        schema.sections_ordered,
        &value,
    )
    .into_iter()
    .map(|m| m.map(|(j, kind)| HeadingMatch::new(admitted[j], kind.by())))
    .collect()
}

/// How a heading matched its section, with what a repair of the heading
/// writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Kind {
    /// Its reading equals the section's text, the literal or the value of
    /// the frontmatter.
    Exact(MatchedBy),
    /// It has the section's template or pattern.
    Shaped,
    /// Its reading equals a `renamed-from`: the section's current text.
    Renamed(String),
    /// It stands for a heading bound to the frontmatter, and reads
    /// otherwise: the value as the heading holds it.
    From(String),
    /// It stands for a heading bound to a frontmatter key that the
    /// document does not set.
    Unbound(String),
}

impl Kind {
    /// How the match is named to a host.
    fn by(&self) -> MatchedBy {
        match self {
            Kind::Exact(by) => *by,
            Kind::Shaped => MatchedBy::Shape,
            Kind::Renamed(_) => MatchedBy::Renamed,
            Kind::From(_) | Kind::Unbound(_) => MatchedBy::Place,
        }
    }
}

/// The text that `section` fixes for its heading: its literal as written,
/// or the value of the frontmatter key that it is bound to as a heading
/// holds it, through `markdown::heading_text_of`.
pub(super) fn expected_text(
    section: &Section,
    value: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    match &section.heading.shape {
        Shape::Text(Text::Literal(t)) => Some(t.clone()),
        Shape::Text(Text::From(key)) => value(key).map(|v| markdown::heading_text_of(&v)),
        _ => None,
    }
}

/// Whether section `k` lies inside section `j`, at any depth.
pub(super) fn inside(parent_of: &[Option<usize>], mut k: usize, j: usize) -> bool {
    while let Some(p) = parent_of[k] {
        if p == j {
            return true;
        }
        k = p;
    }
    false
}

/// The match of each of `headings` to one of `declared`, the sections that
/// the states admit, each of whose parent `parent_of` gives: the walk of
/// the section check, in which each match depends on the matches before
/// it. `value` gives the value of a frontmatter key.
pub(super) fn walk(
    headings: &[Heading],
    declared: &[&Section],
    parent_of: &[Option<usize>],
    ordered: bool,
    value: &dyn Fn(&str) -> Option<String>,
) -> Vec<Option<(usize, Kind)>> {
    let mut seen = vec![0usize; declared.len()];
    let mut pos = 0usize;
    let mut out = Vec::with_capacity(headings.len());
    for h in headings {
        let found = find_match(h, declared, &seen, pos, ordered, value);
        if let Some((j, _)) = &found {
            let j = *j;
            let section = declared[j];
            // A repeating section is met again after its own subsections,
            // which the schema lists between it and its next sibling.
            // sokf-implements: TLMD_33
            // Every entry after it that the walk has passed lies inside it;
            // a repeating entry the walk rests on counts once it has been
            // seen.
            let passed = if seen.get(pos).is_some_and(|&n| n > 0) {
                pos + 1
            } else {
                pos
            };
            let returns =
                section.repeat && j < pos && (j + 1..passed).all(|k| inside(parent_of, k, j));
            seen[j] += 1;
            if ordered {
                let next = if section.repeat { j } else { j + 1 };
                pos = if returns { next } else { pos.max(next) };
            }
        }
        out.push(found);
    }
    out
}

/// The section that `h` matches, of `declared`, after the walk has seen
/// each section `seen` times and rests on `pos`: by the reading of its
/// exact text, then by the shape of a section of its level, then by a
/// `renamed-from`, then as the heading bound to the frontmatter that
/// stands in its place, then by the shape of a section of another level.
// sokf-implements: TLMD_23
// sokf-implements: TLMD_258
fn find_match(
    h: &Heading,
    declared: &[&Section],
    seen: &[usize],
    pos: usize,
    ordered: bool,
    value: &dyn Fn(&str) -> Option<String>,
) -> Option<(usize, Kind)> {
    let order: Vec<usize> = (pos..declared.len()).chain(0..pos).collect();
    let read_heading = h.reading.or_written(&h.text);
    let reading = read_heading.text.as_str();
    for &j in &order {
        let (expected, by) = match &declared[j].heading.read {
            Shape::Text(Text::Literal(t)) => (Some(t.clone()), MatchedBy::Text),
            Shape::Text(Text::From(key)) => (
                value(key).map(|v| read(&markdown::heading_text_of(&v))),
                MatchedBy::Frontmatter,
            ),
            _ => (None, MatchedBy::Text),
        };
        if expected.as_deref() == Some(reading) {
            return Some((j, Kind::Exact(by)));
        }
    }
    // A template such as `${}` matches any heading, so only a section of the
    // heading's own level is tried here. A section of another level is tried
    // last, so that a heading at the wrong level is still claimed, and
    // reported as being at the wrong level.
    let shaped = |j: usize| {
        !declared[j].is_exact() && declared[j].heading.read.matches(&squash(reading), |_| None)
    };
    for &j in &order {
        if declared[j].level == h.level && shaped(j) {
            return Some((j, Kind::Shaped));
        }
    }
    for &j in &order {
        if declared[j].renamed_read.iter().any(|r| r == reading) {
            let current = expected_text(declared[j], value).unwrap_or_else(|| declared[j].label());
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
            let kind = match expected_text(declared[j], value) {
                Some(expected) => Kind::From(expected),
                None => Kind::Unbound(key.clone()),
            };
            return Some((j, kind));
        }
    }
    order
        .iter()
        .find(|&&j| declared[j].level != h.level && shaped(j))
        .map(|&j| (j, Kind::Shaped))
}
