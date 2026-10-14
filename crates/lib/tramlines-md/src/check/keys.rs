//! Captured keys: document-scope uniqueness, `for-each-key-of`,
//! `key-derives-from`, and keys that collide with heading slugs.

use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};

use super::{At, Capture, Checker, Paired, blocks::unit_of, rules_named};
use crate::{
    grammar::{Block, DerivationSource, KeyDerivation, RuleCommon, Section},
    markdown::slug,
};

/// Where a pairing is read: the lines, and the parent heading's line and
/// text where the scope is one occurrence of a parent.
type Scope = (Range<usize>, Option<(usize, String)>);

/// Which pairing a capture belongs to: the pairing its rule declares, and
/// the scope it lies in.
type Pairing = (Paired, usize);

/// One rule that pairs its keys with another rule's: a section by its
/// headings, or a unit rule by its units.
struct Pairer<'a> {
    /// The declared section it lies in.
    section: usize,
    /// The rules whose keys it pairs with.
    with: &'a [String],
    /// Whether it pairs headings.
    heading: bool,
    /// What one of its captures is, for a finding.
    noun: &'static str,
    /// Its own `rule-id`, which names it in a finding.
    rule_id: Option<&'a str>,
    /// Its rule, whose `severity` a finding takes.
    common: &'a RuleCommon,
}

/// Every unit rule under `blocks` that pairs, with what its unit is called.
fn unit_pairers<'a>(blocks: Option<&'a [Block]>, section: usize, out: &mut Vec<Pairer<'a>>) {
    for b in blocks.unwrap_or(&[]) {
        unit_pairers(b.blocks.as_deref(), section, out);
        for u in b.units() {
            let lines = u.lines.iter().flatten().map(|l| (l, "line"));
            for (rule, noun) in std::iter::once((u, unit_of(b.kind))).chain(lines) {
                if let Some(with) = &rule.for_each_key_of {
                    out.push(Pairer {
                        section,
                        with,
                        heading: false,
                        noun,
                        rule_id: rule.rule_id.as_deref(),
                        common: &rule.common,
                    });
                }
            }
            unit_pairers(u.blocks.as_deref(), section, out);
        }
    }
}

/// The key checks a single document settles.
// sokf-implements: TLMD_101
// sokf-implements: TL_32
pub(super) fn check(c: &mut Checker<'_>) {
    // The checks read the captures and add only findings.
    let captures = std::mem::take(&mut c.out.captures);
    uniqueness(c, &captures);
    pairings(c, &captures);
    derivations(c, &captures);
    c.out.captures = captures;
}

/// The scopes a section's pairing is read in: each occurrence of its
/// parent, with the parent's heading, or the whole document.
// sokf-implements: TLMD_97
fn scopes(c: &Checker<'_>, section: usize) -> Vec<Scope> {
    if c.sections[section].parent.is_none() {
        return vec![(0..usize::MAX, None)];
    }
    // A parent the document's states exclude has no occurrence to read in.
    match c.parent_of[section] {
        None => Vec::new(),
        Some(pj) => c
            .occurrences
            .iter()
            .filter(|o| o.section == pj)
            .map(|o| {
                let heading = format!("{} {}", "#".repeat(usize::from(o.level)), o.text);
                (c.span(o), Some((o.line, heading)))
            })
            .collect(),
    }
}

/// Unique within the document, where a pairing is checked on its own: a
/// paired capture is the rule working, not a clash with the key it pairs
/// with, and two captures of one pairing are still two.
// sokf-implements: TLMD_28
// sokf-implements: TLMD_29
// sokf-implements: TLMD_30
fn uniqueness(c: &mut Checker<'_>, captures: &[Capture]) {
    let sections = c.sections.clone();
    let at_capture = |cap: &Capture| At::capture(cap, &sections);
    // The group a capture is checked in: plain keys together, and each
    // pairing, in each scope it is read in, on its own.
    let group_of = |cap: &Capture| -> Option<Pairing> {
        let paired = cap.pairs.clone()?;
        let scope = scopes(c, cap.section?)
            .iter()
            .position(|(span, _)| span.contains(&cap.line))
            .unwrap_or(usize::MAX);
        Some((paired, scope))
    };
    let mut groups: BTreeMap<Option<Pairing>, Vec<&Capture>> = BTreeMap::new();
    for cap in captures {
        groups.entry(group_of(cap)).or_default().push(cap);
    }
    for (pairing, group) in &groups {
        // A key paired to a rule repeats its pairing; any other repeats
        // its scope.
        // sokf-implements: TLMD_29
        // sokf-implements: TLMD_148
        let criterion = if pairing.is_some() {
            "TLMD_29"
        } else {
            "TLMD_148"
        };
        let mut by_key: BTreeMap<&str, Vec<&Capture>> = BTreeMap::new();
        for cap in group.iter() {
            by_key.entry(cap.key.as_str()).or_default().push(cap);
        }
        for (key, caps) in &by_key {
            // A rule that declares `key-repeats` names keys that may repeat,
            // so none of its captures clashes (TLMD_198).
            // sokf-implements: TLMD_198
            let caps: Vec<&Capture> = caps.iter().filter(|cap| !cap.repeats).copied().collect();
            let lines: Vec<usize> = caps.iter().map(|cap| cap.line).collect();
            if lines.len() > 1 {
                let at = group
                    .iter()
                    .find(|cap| cap.key == **key)
                    .map(|cap| at_capture(cap))
                    .unwrap_or_default();
                // A body that repeats with its heading (TLMD_151) captures
                // again the keys of the body it repeats: that is the
                // heading's fault, and any other repeat is the key's own.
                let again: Vec<&usize> = lines
                    .windows(2)
                    .filter(|w| {
                        !c.repeats
                            .iter()
                            .any(|(first, body)| body.contains(&w[1]) && first.contains(&w[0]))
                    })
                    .map(|w| &w[1])
                    .collect();
                for line in again {
                    c.located(at.clone(), |c| {
                        c.error(
                            Some(*line),
                            criterion,
                            format!(
                                "key `{key}` is captured more than once; it was first captured at line {}",
                                lines[0]
                            ),
                        );
                    });
                }
            }
        }
    }
    // A key's slug names its anchor, so it shares one namespace with the
    // headings and the other keys of the document (TLMD_101): two keys, or a
    // key and a heading, with one slug would give one id twice.
    let headings = &c.doc.body.headings;
    let mut first: BTreeMap<String, (&str, usize)> = BTreeMap::new();
    // A reference other than a heading holds no anchor (TLMD_103), so its
    // slug names nothing that could clash.
    for cap in captures
        .iter()
        .filter(|cap| cap.pairs.is_none() || cap.heading_slug.is_some())
    {
        let s = slug(&cap.key);
        if let Some(line) = headings
            .iter()
            .find(|h| h.slug == s && h.line != cap.line)
            .map(|h| h.line)
        {
            c.located(at_capture(cap), |c| {
                c.error(
                    Some(cap.line),
                    "TLMD_101",
                    format!(
                        "key `{}` has the slug `{s}` of the heading at line {line}; keys and headings share one namespace",
                        cap.key
                    ),
                );
            });
        }
        match first.get(&s) {
            Some(&(other, line)) if other != cap.key => {
                c.located(at_capture(cap), |c| {
                    c.error(
                        Some(cap.line),
                        "TLMD_101",
                        format!(
                            "key `{}` has the slug `{s}` of key `{other}` at line {line}; each key needs a slug of its own",
                            cap.key
                        ),
                    );
                });
            }
            Some(_) => {}
            None => {
                first.insert(s, (&cap.key, cap.line));
            }
        }
    }
}

/// One capture per key of the rule each pairing names, in each scope it is
/// read in: a missing key and an orphan are each named. A key is missing
/// from its scope even where the block or section that would hold it is
/// absent.
// sokf-implements: TLMD_95
// sokf-implements: TLMD_96
fn pairings(c: &mut Checker<'_>, captures: &[Capture]) {
    let declared = c.sections.clone();
    let mut pairers = Vec::new();
    for (j, section) in declared.iter().enumerate() {
        // sokf-implements: TLMD_103
        if let Some(with) = &section.for_each_key_of {
            pairers.push(Pairer {
                section: j,
                with,
                heading: true,
                noun: "section",
                rule_id: section.rule_id.as_deref(),
                common: &section.common,
            });
        }
        unit_pairers(section.blocks.as_deref(), j, &mut pairers);
    }
    for p in &pairers {
        let section = declared[p.section];
        let at = At {
            rule_id: p.rule_id.map(str::to_string).or(section.rule_id.clone()),
            line: Some(p.common.line),
            ..At::section(section)
        };
        c.located(at, |c| pairing(c, captures, p, section));
    }
}

/// One pairing, in each scope it is read in.
fn pairing(c: &mut Checker<'_>, captures: &[Capture], p: &Pairer<'_>, section: &Section) {
    for (span, host) in scopes(c, p.section) {
        let under = host
            .as_ref()
            .map_or_else(String::new, |(_, h)| format!(" under `{h}`"));
        // A key captured twice is wanted once; TLMD_148 reports the
        // second capture.
        let mut named = HashSet::new();
        // sokf-implements: TLMD_197
        let wanted: Vec<(&str, usize)> = captures
            .iter()
            .filter(|cap| {
                cap.rule_id.as_ref().is_some_and(|r| p.with.contains(r)) && span.contains(&cap.line)
            })
            .map(|cap| (cap.key.as_str(), cap.line))
            .filter(|(key, _)| named.insert(*key))
            .collect();
        let have: Vec<(&str, usize)> = captures
            .iter()
            .filter(|cap| {
                cap.pairs
                    .as_ref()
                    .is_some_and(|q| q.with == p.with && q.by == p.common.line)
                    && span.contains(&cap.line)
            })
            .map(|cap| (cap.key.as_str(), cap.line))
            .collect();
        // A key with no pair is reported where it is captured.
        for (key, at) in &wanted {
            if !have.iter().any(|(k, _)| k == key) {
                let message = if p.heading {
                    format!("no `{}` section for key `{key}`{under}", section.label())
                } else {
                    format!(
                        "no {} names key `{key}` of {}{under}",
                        p.noun,
                        rules_named(p.with)
                    )
                };
                c.emit(p.common, Some(*at), "TLMD_95", message);
            }
        }
        for (key, line) in &have {
            if !wanted.iter().any(|(k, _)| k == key) {
                let message = if p.heading {
                    format!(
                        "section names key `{key}`, which {} did not capture{under}",
                        rules_named(p.with)
                    )
                } else {
                    format!(
                        "key `{key}` is not one {} captured{under}",
                        rules_named(p.with)
                    )
                };
                c.emit(p.common, Some(*line), "TLMD_96", message);
            }
        }
    }
}

/// Each key whose rule derives it has a prefix the named rule captured or
/// the named frontmatter key holds (TL_32 and TL_33:
/// the derivation sits beside the key it constrains).
// sokf-implements: TL_33
// sokf-implements: TL_32
fn derivations(c: &mut Checker<'_>, captures: &[Capture]) {
    let sections = c.sections.clone();
    for cap in captures {
        let Some(derivation) = &cap.derives else {
            continue;
        };
        let prefixes = prefixes(c, captures, derivation);
        let prefix = cap.key.split_once(&derivation.separator).map(|(p, _)| p);
        let message = match prefix {
            Some(p) if prefixes.iter().any(|q| q == p) => None,
            Some(p) => Some(match &derivation.source {
                DerivationSource::Rule(r) => format!(
                    "key `{}` derives from `{p}`, which rule `{r}` did not capture",
                    cap.key
                ),
                DerivationSource::From(k) => match prefixes.first() {
                    Some(v) => format!(
                        "key `{}` derives from `{p}`, which is not `{v}`, the value of the frontmatter `{k}`",
                        cap.key
                    ),
                    // A value that is no scalar is the value's fault (TL_57).
                    None if c.doc.frontmatter.and_then(|f| f.get(k)).is_some() => continue,
                    None => format!(
                        "key `{}` derives from `{p}`, and the frontmatter `{k}` that it derives from is absent",
                        cap.key
                    ),
                },
            }),
            None => Some(format!(
                "key `{}` has no `{}` separator to derive from",
                cap.key, derivation.separator
            )),
        };
        if let Some(m) = message {
            c.located(At::capture(cap, &sections), |c| {
                c.error(Some(cap.line), "TL_33", m);
            });
        }
    }
}

/// The prefixes a derivation allows: the keys its rule captured, or the
/// value of its frontmatter key.
fn prefixes(c: &Checker<'_>, captures: &[Capture], derivation: &KeyDerivation) -> Vec<String> {
    match &derivation.source {
        DerivationSource::Rule(r) => captures
            .iter()
            .filter(|cap| cap.rule_id.as_deref() == Some(r.as_str()))
            .map(|cap| cap.key.clone())
            .collect(),
        DerivationSource::From(k) => c.front_value(k).into_iter().collect(),
    }
}
