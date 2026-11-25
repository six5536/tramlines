//! Captured keys: document-scope uniqueness, `for-each-key-of`,
//! `key-derives-from`, and keys that collide with heading slugs.

use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};

use super::{At, Capture, Checker, Paired, Spot, anchors, blocks::unit_of, rules_named};
use crate::{
    grammar::{
        self, Block, Captured, DerivationSource, KeyDerivation, KeyText, RuleCommon, Section,
        Shape, Squashed,
    },
    markdown::{self, BlockKind, Unit},
    text,
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
    /// Whether it demands a pair for each key: not where the part that
    /// holds it is declared only by a state that the document could be in
    /// (`TLMD_230`), though a pair that it holds is still checked.
    demands: bool,
}

/// Every unit rule under `blocks` that pairs, with what its unit is called.
fn unit_pairers<'a>(
    blocks: Option<&'a [Block]>,
    section: usize,
    demands: bool,
    c: &Checker<'_>,
    out: &mut Vec<Pairer<'a>>,
) {
    for b in blocks
        .unwrap_or(&[])
        .iter()
        .filter(|b| b.common.only.admits(&c.possible))
    {
        let demands = demands && b.common.only.admits(c.states);
        unit_pairers(b.blocks.as_deref(), section, demands, c, out);
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
                        demands,
                    });
                }
            }
            unit_pairers(u.blocks.as_deref(), section, demands, c, out);
        }
    }
}

/// The key checks a single document settles.
// sokf-implements: TLMD_101
// sokf-implements: TL_32
pub(super) fn check(c: &mut Checker<'_>) {
    // The checks read the captures and add only findings.
    let mut captures = std::mem::take(&mut c.out.captures);
    // Each capture names its key by the capture that stands for it, which
    // for a capture that pairs is a key of its set (TLMD_287), and names the
    // anchor of that key, which a host files it under.
    // sokf-implements: MD-4_AC-22
    let firsts = anchors::identities(&captures);
    let ids = anchors::anchor_ids(&captures, &firsts);
    for (cap, id) in captures.iter_mut().zip(ids) {
        cap.anchor = id;
    }
    uniqueness(c, &captures, &firsts);
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
fn uniqueness(c: &mut Checker<'_>, captures: &[Capture], firsts: &[usize]) {
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
    let mut groups: BTreeMap<Option<Pairing>, Vec<usize>> = BTreeMap::new();
    for (i, cap) in captures.iter().enumerate() {
        groups.entry(group_of(cap)).or_default().push(i);
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
        // The captures of each key, in the order of the key's text, as
        // the first capture of the key reads it.
        let mut by_key: BTreeMap<(&str, usize), Vec<&Capture>> = BTreeMap::new();
        for &i in group {
            let first = firsts[i];
            by_key
                .entry((captures[first].key.as_str(), first))
                .or_default()
                .push(&captures[i]);
        }
        for all in by_key.values() {
            // A rule that declares `key-repeats` names keys that may repeat,
            // so none of its captures clashes (TLMD_198).
            // sokf-implements: TLMD_198
            let caps: Vec<&Capture> = all.iter().filter(|cap| !cap.repeats).copied().collect();
            if caps.len() > 1 {
                let at = at_capture(all[0]);
                // A body that repeats with its heading (TLMD_151) captures
                // again the keys of the body it repeats: that is the
                // heading's fault, and any other repeat is the key's own.
                let again: Vec<&[&Capture]> = caps
                    .windows(2)
                    .filter(|w| {
                        !c.repeats.iter().any(|(first, body)| {
                            body.contains(&w[1].line) && first.contains(&w[0].line)
                        })
                    })
                    .collect();
                for w in again {
                    let (cap, first) = (w[1], caps[0].line);
                    c.located(at.clone(), |c| {
                        c.error(
                            Some(cap.line),
                            criterion,
                            format!(
                                "key {} is captured more than once; it was first captured at line {first}",
                                text::code(&cap.key)
                            ),
                        );
                    });
                }
            }
        }
    }
    slugs(c, captures, firsts);
}

/// A key's slug names its anchor, so it shares one namespace with the
/// headings and the other keys of the document (`TLMD_101`): two keys, or a
/// key and a heading, with one slug would give one id twice. Each capture
/// names its key by the capture that `firsts` gives it.
fn slugs(c: &mut Checker<'_>, captures: &[Capture], firsts: &[usize]) {
    let sections = c.sections.clone();
    let at_capture = |cap: &Capture| At::capture(cap, &sections);
    let headings = &c.doc.body.headings;
    let mut first: BTreeMap<String, usize> = BTreeMap::new();
    // A reference other than a heading holds no anchor (TLMD_103), so its
    // slug names nothing that could clash.
    for (i, cap) in captures
        .iter()
        .enumerate()
        .filter(|(_, cap)| cap.pairs.is_none() || cap.heading_slug.is_some())
    {
        // Each capture of a key names the key by the id of its anchor.
        let s = cap.anchor.clone();
        // A heading that captures the same key, such as a heading that pairs
        // with it, holds the key's anchor and is no other name.
        let own = |line: usize| {
            captures.iter().zip(firsts).any(|(other, &f)| {
                f == firsts[i] && other.heading_slug.is_some() && other.line == line
            })
        };
        if let Some(line) = headings
            .iter()
            .find(|h| h.slug == s && h.line != cap.line && !own(h.line))
            .map(|h| h.line)
        {
            c.located(at_capture(cap), |c| {
                c.error(
                    Some(cap.line),
                    "TLMD_101",
                    format!(
                        "key {} has the slug `{s}` of the heading at line {line}; keys and headings share one namespace",
                        text::code(&cap.key)
                    ),
                );
            });
        }
        match first.get(&s) {
            Some(&j) if firsts[j] != firsts[i] => {
                let other = &captures[j];
                c.located(at_capture(cap), |c| {
                    c.error(
                        Some(cap.line),
                        "TLMD_101",
                        format!(
                            "key {} has the slug `{s}` of key {} at line {}; each key needs a slug of its own",
                            text::code(&cap.key),
                            text::code(&other.key),
                            other.line
                        ),
                    );
                });
            }
            Some(_) => {}
            None => {
                first.insert(s, i);
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
        let demands = section.common.only.admits(c.states);
        // sokf-implements: TLMD_103
        if let Some(with) = &section.for_each_key_of {
            pairers.push(Pairer {
                section: j,
                with,
                heading: true,
                noun: "section",
                rule_id: section.rule_id.as_deref(),
                common: &section.common,
                demands,
            });
        }
        unit_pairers(section.blocks.as_deref(), j, demands, c, &mut pairers);
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
        let under = host.as_ref().map_or_else(String::new, |(_, h)| {
            format!(" under {h}", h = text::code(h))
        });
        // A key captured twice is wanted once; TLMD_148 reports the
        // second capture. A capture that pairs with a name of this pairing,
        // its own among them, refers to a key of the set and defines none,
        // though it can be filed in the set, as under the `rule-id` of its
        // block or its section.
        // Each key is wanted once, with every capture of its text, and it has
        // its pair where a pair is the same key as any of them, because a key
        // of another reading with its plain text is the same key (TLMD_287),
        // and two captures of one text can render otherwise, as `a*b*` and
        // `a_b_` do.
        // sokf-implements: TLMD_197
        // sokf-implements: TLMD_238
        let mut wanted: Vec<(Vec<KeyText>, &str, usize)> = Vec::new();
        for cap in captures.iter().filter(|cap| {
            cap.rule_id.as_ref().is_some_and(|r| p.with.contains(r))
                && cap
                    .pairs
                    .as_ref()
                    .is_none_or(|q| !q.with.iter().any(|w| p.with.contains(w)))
                && span.contains(&cap.line)
        }) {
            match wanted.iter_mut().find(|(_, k, _)| *k == cap.key) {
                Some((texts, ..)) => texts.push(cap.text()),
                None => wanted.push((vec![cap.text()], &cap.key, cap.line)),
            }
        }
        let names = |texts: &[KeyText], got: &KeyText| texts.iter().any(|k| k.same(got));
        let have: Vec<(KeyText, &str, usize)> = captures
            .iter()
            .filter(|cap| {
                cap.pairs
                    .as_ref()
                    .is_some_and(|q| q.with == p.with && q.by == p.common.line)
                    && span.contains(&cap.line)
            })
            .map(|cap| (cap.text(), cap.key.as_str(), cap.line))
            .collect();
        // Each pair names the first key wanted that it is the same key as,
        // as `anchors::identities` gives it the key, so that two keys of the
        // set that are one key in its reading each want a pair of their own.
        let named: Vec<Option<usize>> = have
            .iter()
            .map(|(got, ..)| wanted.iter().position(|(texts, ..)| names(texts, got)))
            .collect();
        // A key with no pair is reported where it is captured.
        for (w, (_, key, at)) in wanted.iter().enumerate().filter(|_| p.demands) {
            if !named.contains(&Some(w)) {
                let message = if p.heading {
                    format!(
                        "no {} section for key {}{under}",
                        text::code(&section.label()),
                        text::code(key)
                    )
                } else {
                    format!(
                        "no {} names key {} of {}{under}",
                        p.noun,
                        text::code(key),
                        rules_named(p.with)
                    )
                };
                c.emit(p.common, Some(*at), "TLMD_95", message);
            }
        }
        for ((_, key, line), w) in have.iter().zip(&named) {
            if w.is_none() {
                let message = if p.heading {
                    format!(
                        "section names key {}, which is not a key of {}{under}",
                        text::code(key),
                        rules_named(p.with)
                    )
                } else {
                    format!(
                        "key {} is not a key of {}{under}",
                        text::code(key),
                        rules_named(p.with)
                    )
                };
                c.emit(p.common, Some(*line), "TLMD_96", message);
            }
        }
    }
}

/// Each key whose rule derives it starts with a prefix the named rule
/// captured or the named frontmatter key holds, followed by the separator
/// (`TL_32` and `TL_33`: the derivation sits beside the key it constrains). A
/// prefix can contain the separator, so the key is matched against each
/// prefix rather than split; a key that no prefix starts is described by
/// the text before its last separator, because a prefix can contain the
/// separator and the key's own tail, such as `1`, rarely does.
// sokf-implements: TL_33
// sokf-implements: TL_32
// sokf-implements: TL_126
fn derivations(c: &mut Checker<'_>, captures: &[Capture]) {
    let sections = c.sections.clone();
    let mut rows = HashSet::new();
    rows_in(&c.doc.body.blocks, &mut rows);
    for cap in captures {
        let Some(derivation) = &cap.derives else {
            continue;
        };
        let key = cap.text();
        // A key of another reading is compared by its plain text
        // (`TLMD_287`), and a frontmatter value reads as the key that
        // derives from it reads, and the finding quotes it as written
        // (`TLMD_259`).
        let values = match &derivation.source {
            DerivationSource::Rule(_) => Vec::new(),
            DerivationSource::From(k) => c.front_value(k).into_iter().collect(),
        };
        let parents: Vec<KeyText> = match &derivation.source {
            DerivationSource::Rule(r) => parents(captures, r, derivation),
            DerivationSource::From(_) => values
                .iter()
                .map(|v| KeyText::new(&read_as(cap, &rows, v), v, key.verbatim))
                .collect(),
        };
        if key.parent_in(derivation, &parents).is_some() {
            continue;
        }
        let prefix = cap.key.rsplit_once(&derivation.separator).map(|(p, _)| p);
        let message = match prefix {
            Some(p) => match &derivation.source {
                DerivationSource::Rule(r) => format!(
                    "key {} derives from {}, which is not a key of set `{r}`",
                    text::code(&cap.key),
                    text::code(p)
                ),
                DerivationSource::From(k) => match values.first() {
                    Some(v) => format!(
                        "key {} derives from {}, which is not {v}, the value of the frontmatter `{k}`",
                        text::code(&cap.key),
                        text::code(p),
                        v = text::code(v)
                    ),
                    // A value that is no scalar is the value's fault (TL_57).
                    None if c.doc.frontmatter.and_then(|f| f.get(k)).is_some() => continue,
                    None => format!(
                        "key {} derives from {}, and the frontmatter `{k}` that it derives from is absent",
                        text::code(&cap.key),
                        text::code(p)
                    ),
                },
            },
            None => format!(
                "key {} has no {} separator to derive from",
                text::code(&cap.key),
                text::code(&derivation.separator)
            ),
        };
        c.located(At::capture(cap, &sections), |c| {
            c.error(Some(cap.line), "TL_33", message);
        });
    }
}

/// `value`, a frontmatter value that the key of `cap` derives from, as the
/// rule that captured the key reads it (`TLMD_259`), squashed as the key is
/// (`TL_23`): as written where a line of a fence or of an HTML block
/// captured it, cell by cell where a table row, one of `rows`, did, and else
/// as a paragraph alone.
// sokf-implements: TLMD_259
fn read_as(cap: &Capture, rows: &HashSet<usize>, value: &str) -> String {
    let read = match cap.spot {
        Spot::Heading(_) => grammar::read(value),
        _ if rows.contains(&cap.line) => grammar::row_reading(value).text,
        Spot::Before(_) => value.to_string(),
        Spot::Inline(..) => grammar::read(value),
    };
    grammar::squash(&read)
}

/// The line of each row of a table among `blocks`, at any depth.
fn rows_in(blocks: &[markdown::Block], out: &mut HashSet<usize>) {
    for b in blocks {
        if b.kind == BlockKind::Table {
            out.extend(b.units.iter().map(|u| u.line));
        }
        for u in &b.units {
            rows_in(&u.nested, out);
        }
    }
}

/// The keys of `rule`, the rule that a derivation names, which a key of the rule
/// that derives can take as its prefix. A key that derives from the same
/// name is no prefix, though it can be filed under the name (`TLMD_238`).
fn parents(captures: &[Capture], rule: &str, derivation: &KeyDerivation) -> Vec<KeyText> {
    captures
        .iter()
        .filter(|cap| {
            cap.rule_id.as_deref() == Some(rule)
                && cap
                    .derives
                    .as_ref()
                    .is_none_or(|d| d.source != derivation.source)
        })
        .map(Capture::text)
        .collect()
}

/// The bytes of `text`, squashed, that the key `name` of `shape` covers.
pub(super) fn key_at(shape: &Shape, name: &str, text: &str) -> Option<Range<usize>> {
    Some(shape.captures(text)?.name(name)?.range())
}

/// The capture a unit's key makes, filed as the checker files every key:
/// `got`, captured from `squashed`, whose bytes `at` hold the key.
// sokf-implements: MD-4_AC-11
// sokf-implements: MD-4_AC-18
pub(super) fn capture(
    c: &Checker<'_>,
    unit: &Unit,
    squashed: &Squashed,
    (got, at): (Captured, Option<Range<usize>>),
    rule: &grammar::Unit,
    block: &grammar::Block,
    section: Option<usize>,
) -> Capture {
    let reading = unit.reading.or_written(&unit.text);
    Capture {
        spot: spot(c, (unit, block.kind), squashed.original(got.start)),
        written: grammar::written_key(&unit.text, &reading, squashed, at, &got.key),
        key: got.key,
        line: unit.line,
        heading_slug: None,
        anchor: String::new(),
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

/// Where the anchor of a key whose match starts at byte `offset` of the
/// unit's text goes (`TLMD_105`): before the block when a line of a code
/// block carries it, else before the match, or before the code span or
/// link the match starts inside. A backslash that `reading` drops before
/// the match belongs to the character after it, so the anchor goes before
/// the backslash, and so does the last of an odd run of backslashes that
/// the reading keeps, which would escape the anchor's `<`. A match that starts where no piece of the
/// text stands, as in a row whose first cell is empty or on the ` | ` that
/// joins two cells, starts at the next piece. The anchor of a key of a
/// task item goes after the item's box (`TLMD_105`).
// sokf-implements: TLMD_105
pub(super) fn spot(c: &Checker<'_>, (unit, kind): (&Unit, BlockKind), offset: usize) -> Spot {
    if let Some(line) = c.lines_block {
        return Spot::Before(line);
    }
    let reading = unit.reading.or_written(&unit.text);
    let mut offset = reading.span_of(&unit.text, offset, offset).start;
    // The box of a task item belongs to its marker, so the anchor goes after
    // it, and the item stays a task.
    if matches!(kind, BlockKind::BulletList | BlockKind::NumberedList) {
        offset = offset.max(grammar::task_box(&unit.text));
    }
    // A backslash that the reading keeps would escape the anchor's `<`.
    let before = &unit.text.as_bytes()[..offset.min(unit.text.len())];
    if before.iter().rev().take_while(|&&b| b == b'\\').count() % 2 == 1 {
        offset -= 1;
    }
    match unit
        .position(offset)
        .filter(|_| !unit.in_joint(offset))
        .or_else(|| unit.position_after(offset))
    {
        Some((line, col)) => Spot::Inline(line, c.doc.body.span_around(line, col).unwrap_or(col)),
        None => Spot::Before(unit.line),
    }
}
