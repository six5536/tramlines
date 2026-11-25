//! The anchor each key needs (`TLMD_104` to `TLMD_109`), against the anchors the
//! body holds.
//! A key has one anchor, named by its slug: on the heading that captures it,
//! where one does, else on the capture that defines it. An anchor that names
//! no key stays where it stands while a unit or a heading that breaks its
//! rule holds it.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::{Broken, Capture, Checker, Placement, Repair, Spot};
use crate::{
    Finding,
    grammar::KeyText,
    markdown::{Anchor, Body, render_anchor},
    text,
};

/// The anchor each key of `captures` needs, in the order the keys were
/// first captured, named by the `Capture::anchor` that the check gives
/// its captures, so `captures` are those of one check, as it returns them.
/// A key whose heading already carries its slug, a key whose slug another
/// key took first, and a key that only references capture, need none.
// sokf-implements: TLMD_103
// sokf-implements: TLMD_104
// sokf-implements: TLMD_105
// sokf-implements: TLMD_106
// sokf-implements: MD-4_AC-8
pub fn placements(captures: &[Capture]) -> Vec<Placement> {
    // The captures of one key, a capture that pairs with a key of another
    // reading among them (`TLMD_287`), by the first capture of the key.
    let firsts = identities(captures);
    let mut by_key: BTreeMap<usize, Vec<&Capture>> = BTreeMap::new();
    let mut order: Vec<usize> = Vec::new();
    for (cap, &first) in captures.iter().zip(&firsts) {
        let entry = by_key.entry(first).or_default();
        if entry.is_empty() {
            order.push(first);
        }
        entry.push(cap);
    }
    let mut ids: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for key in order {
        let caps = &by_key[&key];
        let heading = caps.iter().find(|c| c.heading_slug.is_some());
        let holder = holder(caps);
        // The id that the check gives each capture of the key, which a host
        // reads too (`MD-4_AC-22`), where the check gave it to every capture
        // of the key, and else the id of the capture that holds the anchor.
        let anchor = caps[0].anchor.as_str();
        let id = if !anchor.is_empty() && caps.iter().all(|c| c.anchor == anchor) {
            anchor.to_string()
        } else {
            holder.unwrap_or(caps[0]).id()
        };
        if id.is_empty() || ids.contains(&id) {
            continue;
        }
        // A heading whose own slug is the key's is its anchor already.
        let spot = match (heading, holder) {
            (Some(h), _) if h.heading_slug.as_deref() == Some(id.as_str()) => None,
            (_, holder) => holder.map(|c| c.spot),
        };
        ids.insert(id.clone());
        // The key as the capture that holds the anchor reads it.
        if let (Some(spot), Some(holder)) = (spot, holder) {
            out.push(Placement {
                id,
                key: holder.key.clone(),
                spot,
            });
        }
    }
    out
}

/// The capture of one key, among all its `caps`, that holds its anchor:
/// where the key is described, the heading that captures it, else where it
/// is defined. A paired capture refers to a key that another rule defines,
/// and holds no anchor of its own. A rule that declares `key-repeats`
/// refers to its keys, so a capture by another rule defines the key where
/// there is one. The holder names the anchor, because the id reads the key
/// as that capture writes it.
pub(super) fn holder<'a>(caps: &[&'a Capture]) -> Option<&'a Capture> {
    caps.iter()
        .find(|c| c.heading_slug.is_some())
        .or_else(|| caps.iter().find(|c| c.pairs.is_none() && !c.repeats))
        .or_else(|| caps.iter().find(|c| c.pairs.is_none()))
        .copied()
}

/// For each of `captures`, the index of the capture that stands for its
/// key: the first capture of its key, and for a capture that pairs, the
/// first capture of the key of the set that it pairs with, as the check of
/// the pairing reads the set, which a key of
/// another reading can be by its plain text (`TLMD_287`). So a capture that
/// pairs refers to that key, holds no anchor, and shares its slug.
// sokf-implements: TLMD_287
pub(super) fn identities(captures: &[Capture]) -> Vec<usize> {
    let mut first: HashMap<&str, usize> = HashMap::new();
    for (i, cap) in captures.iter().enumerate() {
        first.entry(cap.key.as_str()).or_insert(i);
    }
    let own = |cap: &Capture| first[cap.key.as_str()];
    if captures.iter().all(|cap| cap.pairs.is_none()) {
        return captures.iter().map(own).collect();
    }
    let texts: Vec<KeyText> = captures.iter().map(Capture::text).collect();
    captures
        .iter()
        .enumerate()
        .map(|(i, cap)| {
            let Some(paired) = &cap.pairs else {
                return own(cap);
            };
            captures
                .iter()
                .enumerate()
                .find(|(j, other)| {
                    other
                        .pairs
                        .as_ref()
                        .is_none_or(|q| !q.with.iter().any(|w| paired.with.contains(w)))
                        && other
                            .rule_id
                            .as_ref()
                            .is_some_and(|r| paired.with.contains(r))
                        && texts[*j].same(&texts[i])
                })
                .map_or_else(|| own(cap), |(_, other)| own(other))
        })
        .collect()
}

/// The id of the anchor of the key of each of `captures`, whose key is
/// that of the capture that `firsts` names: the id of the capture that
/// holds the anchor, else of the key's first capture.
pub(super) fn anchor_ids(captures: &[Capture], firsts: &[usize]) -> Vec<String> {
    let mut by_key: BTreeMap<usize, Vec<&Capture>> = BTreeMap::new();
    for (cap, &first) in captures.iter().zip(firsts) {
        by_key.entry(first).or_default().push(cap);
    }
    let ids: BTreeMap<usize, String> = by_key
        .into_iter()
        .map(|(first, caps)| (first, holder(&caps).unwrap_or(caps[0]).id()))
        .collect();
    firsts.iter().map(|first| ids[first].clone()).collect()
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

/// Where `anchor` stands, when a unit or a heading that breaks its rule
/// holds it: on a line of the unit's or the heading's text, or on the line
/// of anchors before the block of a line of a fence. None holds the slug of
/// a heading, which already names the heading.
fn held(anchor: &Anchor, broken: &[&Broken], body: &Body, text: &str) -> Option<Spot> {
    if body.headings.iter().any(|h| h.slug == anchor.id) {
        return None;
    }
    broken.iter().find_map(|b| match b {
        Broken::Text(lines) | Broken::Beyond(lines) | Broken::Heading(lines)
            if !anchor.alone && lines.contains(&anchor.line) =>
        {
            Some(Spot::Inline(anchor.line, anchor.col))
        }
        Broken::Before(line) if at_spot(anchor, Spot::Before(*line), body, text) => {
            Some(Spot::Before(*line))
        }
        Broken::Block(lines) if !anchor.alone && lines.contains(&anchor.line) => {
            Some(Spot::Inline(anchor.line, anchor.col))
        }
        _ => None,
    })
}

/// The records of the units that break their rules, less those of the
/// units that a repair rewrites to the text of their rule, which section 7
/// of the specification says break no rule. The final repairs decide,
/// whether the check recorded a unit before or after its repair. A repair
/// rewrites a heading, a paragraph, or an item, and the record of a
/// paragraph or an item holds the lines of its text. Content beyond the
/// text outlasts the repair, and so does the finding on a heading that
/// breaks its rule, so their records stay.
fn unrepaired<'a>(broken: &'a [Broken], repairs: &[Repair]) -> Vec<&'a Broken> {
    let rewritten: HashSet<usize> = repairs.iter().map(|r| r.line).collect();
    broken
        .iter()
        .filter(|b| !matches!(b, Broken::Text(lines) if rewritten.contains(lines.start())))
        .collect()
}

/// Compare the anchors the body holds with those its keys need, report
/// each difference, and keep what the repair writes.
// sokf-implements: TLMD_109
// sokf-implements: TLMD_139
#[expect(
    clippy::too_many_lines,
    reason = "one pass of the check, a split of which is a refactor of its own"
)]
pub(super) fn check(c: &mut Checker<'_>) {
    let mut wanted = placements(&c.out.captures);
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
                    "key {} has no anchor; its repair writes `{}` before it",
                    text::code(&p.key),
                    render_anchor(&p.id)
                ),
            ),
            None => fail(
                &mut findings,
                body.anchors[found[0]].line,
                format!(
                    "the anchor {} is not where its key {} is; its repair moves it",
                    text::code(&format!("#{}", p.id)),
                    text::code(&p.key)
                ),
            ),
        }
    }
    let wanted_ids: HashSet<&str> = wanted.iter().map(|p| p.id.as_str()).collect();
    let broken = unrepaired(&c.broken, &c.out.repairs);
    // The anchors that units which break their rules hold, which the
    // repair writes again where they stand.
    let mut kept = Vec::new();
    for (i, a) in body.anchors.iter().enumerate() {
        if claimed.contains(&i) {
            continue;
        }
        // A capture of a key with the anchor's slug on the anchor's line.
        let captured_here = |paired: bool| {
            c.out
                .captures
                .iter()
                .any(|cap| cap.pairs.is_some() == paired && cap.line == a.line && cap.id() == a.id)
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
            // A unit that breaks its rule captures no key until its text
            // passes, so the anchor waits for the key where it stands.
            if let Some(spot) = held(a, &broken, body, text) {
                // One copy stays, as for an anchor whose key is captured.
                if kept.iter().any(|k: &Placement| k.id == a.id) {
                    fail(
                        &mut findings,
                        a.line,
                        format!(
                            "the anchor {} occurs more than once; its repair keeps one",
                            text::code(&format!("#{}", a.id))
                        ),
                    );
                    continue;
                }
                kept.push(Placement {
                    id: a.id.clone(),
                    key: a.id.clone(),
                    spot,
                });
                continue;
            }
            fail(
                &mut findings,
                a.line,
                format!(
                    "the anchor {} names no key of this document; its repair removes it",
                    text::code(&format!("#{}", a.id))
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
                    "the anchor {} occurs more than once; its repair keeps one",
                    text::code(&format!("#{}", a.id))
                ),
            );
        }
    }
    // sokf-implements: MD-4_AC-9
    c.out.anchors_stale = stale || !findings.is_empty();
    // sokf-implements: MD-4_AC-8
    wanted.extend(kept);
    c.out.anchors = wanted;
    // A scaffold carries no anchors until a new document is written from it.
    // sokf-implements: MD-4_AC-5
    if !c.options.scaffold {
        c.out.findings.extend(findings);
    }
}
