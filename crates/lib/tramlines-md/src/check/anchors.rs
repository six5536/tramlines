//! The anchor each key needs (TLMD_104 to TLMD_109), against the anchors the
//! body holds.
//! A key has one anchor, named by its slug: on the heading that captures it,
//! where one does, else on the capture that defines it.

use std::collections::{BTreeMap, HashSet};

use super::{Capture, Checker, Placement, Spot};
use crate::{
    Finding,
    markdown::{Anchor, Body, render_anchor, slug},
};

/// The anchor each key of `captures` needs, in the order the keys were
/// first captured. A key whose heading already carries its slug, a key
/// whose slug another key took first, and a key that only references
/// capture, need none.
// sokf-implements: TLMD_103
// sokf-implements: TLMD_104
// sokf-implements: TLMD_105
// sokf-implements: TLMD_106
// sokf-implements: MD-4_AC-8
pub fn placements(captures: &[Capture]) -> Vec<Placement> {
    let mut by_key: BTreeMap<&str, Vec<&Capture>> = BTreeMap::new();
    let mut order: Vec<&str> = Vec::new();
    for cap in captures {
        let entry = by_key.entry(&cap.key).or_default();
        if entry.is_empty() {
            order.push(&cap.key);
        }
        entry.push(cap);
    }
    let mut ids: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for key in order {
        let caps = &by_key[key];
        let id = slug(key);
        if id.is_empty() || ids.contains(&id) {
            continue;
        }
        // Where the key is described: the heading that captures it. A
        // heading whose own slug is the key's is its anchor already.
        // Else where it is defined: a paired capture refers to a key
        // that another rule defines, and holds no anchor of its own.
        let spot = match caps.iter().find(|c| c.heading_slug.is_some()) {
            Some(h) if h.heading_slug.as_deref() == Some(id.as_str()) => None,
            Some(h) => Some(h.spot),
            // A rule that declares `key-repeats` refers to its keys, so a
            // capture by another rule defines the key where there is one.
            None => caps
                .iter()
                .find(|c| c.pairs.is_none() && !c.repeats)
                .or_else(|| caps.iter().find(|c| c.pairs.is_none()))
                .map(|c| c.spot),
        };
        ids.insert(id.clone());
        if let Some(spot) = spot {
            out.push(Placement {
                id,
                key: key.to_string(),
                spot,
            });
        }
    }
    out
}

/// The byte column where the text of the heading on `line` starts, in the
/// line with its anchors removed.
pub fn heading_col(body: &Body, text: &str, line: usize) -> usize {
    let raw = text.lines().nth(line.saturating_sub(1)).unwrap_or("");
    let clean = body.clean_line(line, raw);
    let setext = body.headings.iter().any(|h| h.line == line && h.setext);
    let lead = clean.len() - clean.trim_start().len();
    if setext {
        return lead;
    }
    let after_hashes = clean[lead..].trim_start_matches('#');
    let hashes_end = clean.len() - after_hashes.len();
    hashes_end + (after_hashes.len() - after_hashes.trim_start().len())
}

/// Whether `anchor` sits where `spot` puts it.
fn at_spot(anchor: &Anchor, spot: Spot, body: &Body, text: &str) -> bool {
    match spot {
        Spot::Heading(line) => {
            !anchor.alone && anchor.line == line && anchor.col == heading_col(body, text, line)
        }
        Spot::Inline(line, col) => !anchor.alone && anchor.line == line && anchor.col == col,
        // A line of anchors, then blank lines, then the block.
        Spot::Before(line) => {
            anchor.alone
                && anchor.line < line
                && text
                    .lines()
                    .enumerate()
                    .skip(anchor.line)
                    .take(line.saturating_sub(anchor.line + 1))
                    .all(|(i, l)| body.clean_line(i + 1, l).trim().is_empty())
        }
    }
}

/// Compare the anchors the body holds with those its keys need, report
/// each difference, and keep what the repair writes.
// sokf-implements: TLMD_109
pub(super) fn check(c: &mut Checker<'_>) {
    let wanted = placements(&c.out.captures);
    let body = c.doc.body;
    let text = c.doc.text;
    let mut findings = Vec::new();
    let mut stale = false;
    let mut claimed: HashSet<usize> = HashSet::new();
    let fail = |findings: &mut Vec<Finding>, line: usize, message: String| {
        findings.push(Finding::error(
            c.doc.path.clone(),
            Some(line),
            "TLMD_109",
            message,
        ));
    };
    for p in &wanted {
        let found: Vec<usize> = (0..body.anchors.len())
            .filter(|&i| body.anchors[i].id == p.id)
            .collect();
        match found
            .iter()
            .find(|&&i| at_spot(&body.anchors[i], p.spot, body, text))
        {
            Some(&i) => {
                claimed.insert(i);
            }
            None if found.is_empty() => fail(
                &mut findings,
                p.spot.line(),
                format!(
                    "key `{}` has no anchor; its repair writes `{}` before it",
                    p.key,
                    render_anchor(&p.id)
                ),
            ),
            None => fail(
                &mut findings,
                body.anchors[found[0]].line,
                format!(
                    "the anchor `#{}` is not where its key `{}` is; its repair moves it",
                    p.id, p.key
                ),
            ),
        }
    }
    let wanted_ids: HashSet<&str> = wanted.iter().map(|p| p.id.as_str()).collect();
    for (i, a) in body.anchors.iter().enumerate() {
        if claimed.contains(&i) {
            continue;
        }
        // A capture of a key with the anchor's slug on the anchor's line.
        let captured_here = |paired: bool| {
            c.out.captures.iter().any(|cap| {
                cap.pairs.is_some() == paired && cap.line == a.line && slug(&cap.key) == a.id
            })
        };
        if !wanted_ids.contains(a.id.as_str()) {
            // A paired capture refers to its key and holds no anchor, so an
            // anchor beside one is what an earlier repair wrote there: the
            // fault is the key the pairing reports, and the repair removes
            // the anchor.
            if captured_here(true) {
                stale = true;
                continue;
            }
            fail(
                &mut findings,
                a.line,
                format!(
                    "the anchor `#{}` names no key of this document; its repair removes it",
                    a.id
                ),
            );
        } else if body.anchors.iter().filter(|o| o.id == a.id).count() > 1 {
            // A copy beside a second capture of a key with its slug, where
            // the anchor belongs to the first, is that capture's fault,
            // which TLMD_148 or TLMD_101 reports; the repair still keeps one.
            let belongs_here = wanted
                .iter()
                .any(|p| p.id == a.id && p.spot.line() == a.line);
            if captured_here(false) && !belongs_here {
                stale = true;
                continue;
            }
            fail(
                &mut findings,
                a.line,
                format!(
                    "the anchor `#{}` occurs more than once; its repair keeps one",
                    a.id
                ),
            );
        }
    }
    // sokf-implements: MD-4_AC-9
    c.out.anchors_stale = stale || !findings.is_empty();
    c.out.anchors = wanted;
    // A scaffold carries no anchors until a new document is written from it.
    // sokf-implements: MD-4_AC-5
    if !c.options.scaffold {
        c.out.findings.extend(findings);
    }
}
