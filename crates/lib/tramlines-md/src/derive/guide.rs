//! The guidance of each part of a schema that applies at some states, as a
//! host asks for it: the text of each guidance line that the scaffold
//! writes, and the limit of each part. The scaffold takes the text of each
//! block's and each frontmatter key's guidance line from here, so a host
//! that prints the guide and the scaffold print the same words, the
//! host's words for each limit among them. A text holds the words alone:
//! a guidance line also spells a `-->` in them as `- ->`, so that the
//! comment stays one comment.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer};

use super::{
    deriver::{Examples, address},
    scaffold,
};
use crate::{
    check::{self, Counts, Host, LimitFacts, LimitSubject, RuleName},
    grammar::{
        Block, FrontmatterConstraint, Label, Limit, LimitKind, Schema, Section, States, Unit,
        squash,
    },
    markdown::{self, BlockKind},
};

// sokf:begin guide
/// The guidance of a schema at some states: the schema's own, then each
/// frontmatter key, the preamble, and each section, in the schema's order,
/// and only the parts that those states admit. A block whose hard `max` of
/// zero bans it is left out, as the scaffold leaves it out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Guide {
    /// The schema's top-level `guidance`.
    pub guidance: Option<String>,
    /// The message of each ban of the whole body, in order.
    pub bans: Vec<String>,
    /// Each label of the schema's `labels`, in order, with the words that
    /// a document writes for it.
    pub labels: Vec<LabelGuide>,
    /// Each declared frontmatter key that the states admit, in the
    /// schema's order, then each lifecycle dimension.
    pub frontmatter: Vec<KeyGuide>,
    /// The preamble, where the schema declares one that the states admit.
    pub preamble: Option<PreambleGuide>,
    /// Each section that the states admit, in the schema's order, a
    /// section that declares `parent` after its parent.
    pub sections: Vec<SectionGuide>,
}

/// One label of a schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LabelGuide {
    /// The label's key.
    pub key: String,
    /// The words that a document writes between the label's brackets: the
    /// host's words in the language of the schema, or else the key.
    pub words: String,
}

/// The guidance of one frontmatter key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyGuide {
    /// The key.
    pub key: String,
    /// Whether the key is a lifecycle dimension rather than a declared key.
    pub lifecycle: bool,
    /// Whether the key can be absent.
    pub optional: bool,
    /// The key's own `guidance`.
    pub guidance: Option<String>,
    /// The limit on the elements of a list.
    pub limit: Option<Limit>,
    /// The text of the guidance line that the scaffold writes before the
    /// key, where it writes the key, or `None` where the key has no
    /// guidance and no limit.
    pub text: Option<String>,
    /// The line that the scaffold writes for the key, such as
    /// `priority: medium`, without its `only` marker, or `None` where it
    /// writes none.
    pub example: Option<String>,
    /// The line of the key's rule in the schema.
    pub line: usize,
}

/// The guidance of the preamble.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreambleGuide {
    /// The preamble's own `guidance`.
    pub guidance: Option<String>,
    /// The text of the guidance line that the scaffold writes before the
    /// preamble, or `None` where it writes none.
    pub text: Option<String>,
    /// The text that the scaffold writes for the preamble, or `None` where
    /// it writes none.
    pub example: Option<String>,
    /// Each block that the states admit, in order.
    pub blocks: Vec<BlockGuide>,
    /// The line of the preamble in the schema.
    pub line: usize,
}

/// The guidance of one section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionGuide {
    /// The heading as a finding names it: its text, its template or
    /// pattern, or `<key>` for a heading that takes the value of the
    /// frontmatter key `key`.
    pub heading: String,
    /// The heading's level.
    pub level: u8,
    /// The section's `rule-id`.
    pub rule_id: Option<String>,
    /// The `rule-id` of the section that this one lies inside.
    pub parent: Option<String>,
    /// Whether the section can be absent at the states.
    pub optional: bool,
    /// Whether the heading can occur more than once.
    pub repeat: bool,
    /// The section's own `guidance`.
    pub guidance: Option<String>,
    /// The text of the guidance line that the scaffold writes before the
    /// heading, or `None` where it writes none.
    pub text: Option<String>,
    /// The text that the scaffold writes for the section: each occurrence,
    /// from its heading, with the sections inside it and the section's
    /// `examples`, or `None` where it writes none. A section inside a
    /// repeating one gives the text of its first occurrence.
    pub example: Option<String>,
    /// Each block of its body that the states admit, in order.
    pub blocks: Vec<BlockGuide>,
    /// The line of the section's rule in the schema.
    pub line: usize,
}

/// The guidance of one block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockGuide {
    /// The kind of the block, as the grammar names it.
    #[serde(serialize_with = "kind_name")]
    pub kind: BlockKind,
    /// The block's `rule-id`.
    pub rule_id: Option<String>,
    /// The block's place in the list that declares it, from one.
    pub pos: usize,
    /// Whether the block can be absent at the states.
    pub optional: bool,
    /// The block's own `guidance`.
    pub guidance: Option<String>,
    /// The block's own limit, then the limit of each of its unit rules and
    /// line rules, in the schema's order.
    pub limits: Vec<RuleLimit>,
    /// The block's own `label`, then the `label` of each of its unit rules
    /// that declares one, in the schema's order. A unit rule that declares
    /// none takes the block's, and a position of a `sequence` that declares
    /// none takes the `label` of the block's `each` before the block's.
    pub labels: Vec<RuleLabel>,
    /// The message of each ban of its unit rules, of the formats that they
    /// bind, and of their line rules, each once, in order.
    pub bans: Vec<String>,
    /// The text of the guidance line that the scaffold writes before the
    /// block, or `None` where it writes none.
    pub text: Option<String>,
    /// The text that the scaffold writes for the block, every unit of it,
    /// or `None` where it writes none. A block of a section that repeats,
    /// or of an item, gives the text of its first occurrence, without the
    /// indent of the item.
    pub example: Option<String>,
    /// The blocks of a group, and the blocks inside the items of a list,
    /// that the states admit, in order.
    pub blocks: Vec<BlockGuide>,
    /// The line of the block's rule in the schema.
    pub line: usize,
}

/// Whether the units of a block, or those of one of its unit rules,
/// require a label or allow one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleLabel {
    /// The `Unit::index` of the unit rule whose `label` this is, which
    /// `Schema::unit` finds, or `None` for the block's own `label`.
    pub rule: Option<usize>,
    /// The `rule-id` of the unit rule, or `None` for the block's own
    /// `label` and a rule that declares none.
    pub rule_id: Option<String>,
    /// The line of the rule in the schema.
    pub line: usize,
    /// Whether the units require a label or allow one.
    pub label: Label,
}

/// One limit of a block: its own, or one of a unit rule's or a line
/// rule's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleLimit {
    /// The `Unit::index` of the unit rule whose limit this is, which
    /// `Schema::unit` finds, or `None` for the block's own limit.
    pub rule: Option<usize>,
    /// The name of the format that the unit rule or the line rule binds,
    /// or `None` for the block's own limit and a rule that binds none.
    pub format: Option<String>,
    /// The `rule-id` of the unit rule or the line rule, or `None` for the
    /// block's own limit and a rule that declares none.
    pub rule_id: Option<String>,
    /// What the rule counts, the units of the block or the content lines
    /// of a code block or an HTML block, or `None` for the block's own
    /// limit.
    pub counts: Option<Counts>,
    /// The line of the rule in the schema.
    pub line: usize,
    /// The limit.
    pub limit: Limit,
}
// sokf:end guide

// sokf-implements: MD-13_AC-3
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's `serialize_with` fixes the signature `fn(&T, S)`"
)]
fn kind_name<S: Serializer>(kind: &BlockKind, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(kind.name())
}

/// The guidance and the limits of each part of `schema` that `states`
/// admit, which need not be a document's own states: a host can ask for
/// the guidance of a state that a document is moving to. Each text holds
/// the words that `host` gives for each limit, as the scaffold derived
/// with `host` does.
// sokf-implements: TLMD_233
// sokf-implements: MD-13_AC-1
// sokf:begin guide
pub fn guide(schema: &Schema, states: &States, host: &dyn Host) -> Guide {
    // sokf:end guide
    guide_for(schema, states, host, &BTreeMap::new())
}

/// The guide of `schema` at `states`, as `guide` returns it, for a document
/// whose frontmatter holds `values`: each `example` is the text that
/// `derive_for` writes for the part with those values, so that a key that
/// derives from a value, such as a prefix of the frontmatter `code`, derives
/// from the document's. Every `example` is `None` where the schema derives
/// no scaffold at `states`.
// sokf-implements: MD-13_AC-5
// sokf-implements: MD-13_AC-6
// sokf-implements: TLMD_310
// sokf:begin guide
pub fn guide_for(
    schema: &Schema,
    states: &States,
    host: &dyn Host,
    values: &BTreeMap<String, String>,
) -> Guide {
    // sokf:end guide
    let ex = scaffold::examples(schema, states, host, values).unwrap_or_default();
    let mut frontmatter: Vec<KeyGuide> = schema
        .core
        .frontmatter
        .iter()
        .filter(|(_, c)| c.common.only.admits(states))
        .map(|(key, c)| KeyGuide {
            key: key.clone(),
            lifecycle: false,
            optional: c.optional,
            guidance: c.guidance.clone(),
            limit: c.max.clone(),
            text: key_text(host, schema.core.language.as_deref(), key, c),
            example: ex.keys.get(key).cloned(),
            line: c.common.line,
        })
        .collect();
    frontmatter.extend(schema.core.lifecycle.iter().map(|d| KeyGuide {
        key: d.key.clone(),
        lifecycle: true,
        optional: false,
        guidance: d.guidance.clone(),
        limit: None,
        text: d.guidance.as_deref().map(own_line),
        example: ex.keys.get(&d.key).cloned(),
        line: d.line,
    }));
    Guide {
        guidance: schema.core.guidance.clone(),
        bans: schema
            .core
            .prohibited_patterns
            .iter()
            .map(|b| b.message.clone())
            .collect(),
        // sokf-implements: MD-13_AC-4
        labels: check::words_of(schema, host)
            .iter()
            .map(|(key, words)| LabelGuide {
                key: key.to_string(),
                words: words.to_string(),
            })
            .collect(),
        frontmatter,
        preamble: schema
            .preamble
            .as_ref()
            .filter(|p| p.common.only.admits(states))
            .map(|p| PreambleGuide {
                guidance: p.guidance.clone(),
                text: p.guidance.as_deref().map(own_line),
                example: ex.preamble.clone(),
                blocks: blocks(schema, p.blocks.as_deref(), states, host, &ex),
                line: p.common.line,
            }),
        sections: schema
            .sections
            .iter()
            .filter(|s| s.common.only.admits(states))
            .map(|s| section(schema, s, states, host, &ex))
            .collect(),
    }
}

fn section(
    schema: &Schema,
    s: &Section,
    states: &States,
    host: &dyn Host,
    ex: &Examples,
) -> SectionGuide {
    SectionGuide {
        heading: s.label(),
        level: s.level,
        rule_id: s.rule_id.clone(),
        parent: s.parent.clone(),
        optional: s.optional_at(states, &schema.core.lifecycle),
        repeat: s.repeat,
        guidance: s.guidance.clone(),
        text: s.guidance.as_deref().map(own_line),
        example: ex.sections.get(&address(s)).cloned(),
        blocks: blocks(schema, s.blocks.as_deref(), states, host, ex),
        line: s.common.line,
    }
}

/// The blocks of `list` that `states` admit, less a block whose hard
/// `max` of zero bans it, as the scaffold leaves it out.
fn blocks(
    schema: &Schema,
    list: Option<&[Block]>,
    states: &States,
    host: &dyn Host,
    ex: &Examples,
) -> Vec<BlockGuide> {
    list.unwrap_or(&[])
        .iter()
        .filter(|b| b.common.only.admits(states) && b.hard_max() != Some(0))
        .map(|b| block(schema, b, states, host, ex))
        .collect()
}

fn block(
    schema: &Schema,
    b: &Block,
    states: &States,
    host: &dyn Host,
    ex: &Examples,
) -> BlockGuide {
    let mut inner = blocks(schema, b.blocks.as_deref(), states, host, ex);
    for unit in b.units() {
        inner.extend(blocks(schema, unit.blocks.as_deref(), states, host, ex));
    }
    let limits = block_limits(b);
    let bans = block_bans(schema, b);
    BlockGuide {
        kind: b.kind,
        rule_id: b.rule_id.clone(),
        pos: b.pos,
        optional: b.optional_at(states, &schema.core.lifecycle),
        guidance: b.guidance.clone(),
        text: line_text(host, schema.core.language.as_deref(), b, &limits, &bans),
        example: ex.blocks.get(&address(b)).cloned(),
        limits,
        labels: block_labels(b),
        bans,
        blocks: inner,
        line: b.common.line,
    }
}

/// The block's own `label`, then the `label` of each of its unit rules that
/// declares one, in the schema's order.
// sokf-implements: MD-13_AC-4
fn block_labels(b: &Block) -> Vec<RuleLabel> {
    let own = b.label.map(|label| RuleLabel {
        rule: None,
        rule_id: None,
        line: b.common.line,
        label,
    });
    own.into_iter()
        .chain(b.units().filter_map(|u| {
            Some(RuleLabel {
                rule: Some(u.index),
                rule_id: u.rule_id.clone(),
                line: u.common.line,
                label: u.label?,
            })
        }))
        .collect()
}

/// The block's own limit, then each limit of its unit rules and of their
/// line rules, in the schema's order.
fn block_limits(b: &Block) -> Vec<RuleLimit> {
    fn unit_limits(u: &Unit, counts: Counts, out: &mut Vec<RuleLimit>) {
        if let Some(limit) = &u.max {
            out.push(RuleLimit {
                rule: Some(u.index),
                format: u.format.clone(),
                rule_id: u.rule_id.clone(),
                counts: Some(counts),
                line: u.common.line,
                limit: limit.clone(),
            });
        }
        for line in u.lines.iter().flatten() {
            unit_limits(line, Counts::Lines, out);
        }
    }
    let mut out: Vec<RuleLimit> = b
        .max
        .iter()
        .map(|limit| RuleLimit {
            rule: None,
            format: None,
            rule_id: None,
            counts: None,
            line: b.common.line,
            limit: limit.clone(),
        })
        .collect();
    for u in b.units() {
        unit_limits(u, Counts::Units, &mut out);
    }
    out
}

/// The messages of the bans that govern the units of `b`: the bans of its
/// unit rules, of the formats that they bind, and of their line rules,
/// each message once, in order.
// sokf-implements: TLMD_195
// sokf-implements: TLMD_271
pub(super) fn block_bans(schema: &Schema, b: &Block) -> Vec<String> {
    fn unit_bans(schema: &Schema, u: &Unit, out: &mut Vec<String>) {
        let format = u.format_in(&schema.core.formats);
        let bans = u
            .prohibited_patterns
            .iter()
            .chain(format.into_iter().flat_map(|f| f.prohibited.iter()));
        push_messages(out, bans.map(|ban| ban.message.clone()));
        for line in u.lines.iter().flatten() {
            unit_bans(schema, line, out);
        }
    }
    let mut out = Vec::new();
    for u in b.units() {
        unit_bans(schema, u, &mut out);
    }
    out
}

/// `messages` added to `out`, each but one whose reading, squashed, is the
/// reading of a message that `out` holds, because a writer reads such two
/// as one rule (`TLMD_271`).
pub(super) fn push_messages(out: &mut Vec<String>, messages: impl IntoIterator<Item = String>) {
    let read = |m: &str| squash(&markdown::reading(&squash(m)).text).clone();
    for m in messages {
        let r = read(&m);
        if !out.iter().any(|o| read(o) == r) {
            out.push(m);
        }
    }
}

/// The text of a block's guidance line: its guidance, then the lead and
/// the `when-at-limit`, unchanged, of each of its limits, then the
/// messages of its bans, or `None` where there is none of them.
// sokf-implements: TLMD_195
// sokf-implements: TLMD_231
// sokf-implements: MD-13_AC-2
pub(super) fn block_text(schema: &Schema, host: &dyn Host, b: &Block) -> Option<String> {
    line_text(
        host,
        schema.core.language.as_deref(),
        b,
        &block_limits(b),
        &block_bans(schema, b),
    )
}

/// The text of a block's guidance line from its limits and its bans.
fn line_text(
    host: &dyn Host,
    language: Option<&str>,
    b: &Block,
    limits: &[RuleLimit],
    bans: &[String],
) -> Option<String> {
    joined(
        b.guidance
            .iter()
            .cloned()
            .chain(
                limits
                    .iter()
                    .flat_map(|l| limit_texts(host, &l.limit, subject(b, l), language)),
            )
            .chain(bans.iter().cloned()),
    )
}

/// The text of a frontmatter key's guidance line: its guidance, then the
/// lead and the `when-at-limit`, unchanged, of its limit, or `None` where
/// it has neither.
// sokf-implements: TLMD_232
// sokf-implements: MD-13_AC-2
// sokf-implements: MD-5_AC-17
pub(super) fn key_text(
    host: &dyn Host,
    language: Option<&str>,
    key: &str,
    c: &FrontmatterConstraint,
) -> Option<String> {
    joined(
        c.guidance.iter().cloned().chain(
            c.max
                .iter()
                .flat_map(|l| limit_texts(host, l, LimitSubject::Key(key), language)),
        ),
    )
}

/// What the limit `l` of block `b` is applied to: the kind of the block
/// for its own limit, and else the rule, with the kind of the block, what
/// the rule counts, and the format that it binds or else its `rule-id`.
// sokf-implements: MD-5_AC-17
// sokf-implements: MD-5_AC-18
fn subject<'a>(b: &Block, l: &'a RuleLimit) -> LimitSubject<'a> {
    let Some(counts) = l.counts else {
        return LimitSubject::Block(b.kind);
    };
    let name = match (&l.format, &l.rule_id) {
        (Some(format), _) => Some(RuleName::Format(format)),
        (None, Some(id)) => Some(RuleName::RuleId(id)),
        (None, None) => None,
    };
    LimitSubject::Rule {
        block: b.kind,
        counts,
        name,
    }
}

/// The lead of `limit` on `subject`, then its `when-at-limit`. The lead
/// is the words that `host` gives, where it gives words that aren't
/// blank, and else the limit in the syntax of the schema, such as
/// `` `hard: 9`, `when-at-limit`: ``. The engine adds no words of its own.
// sokf-implements: MD-5_AC-15
// sokf-implements: MD-5_AC-16
// sokf-implements: MD-5_AC-19
// sokf-implements: TLMD_234
// sokf-implements: TLMD_231
// sokf-implements: TLMD_232
fn limit_texts(
    host: &dyn Host,
    limit: &Limit,
    subject: LimitSubject<'_>,
    language: Option<&str>,
) -> impl Iterator<Item = String> {
    let facts = LimitFacts::new(limit.kind, limit.count, subject, language);
    let lead = host
        .limit_lead(&facts)
        .filter(|words| !squash(words).is_empty())
        .unwrap_or_else(|| {
            let kind = match limit.kind {
                LimitKind::Soft => "soft",
                LimitKind::Hard => "hard",
            };
            format!("`{kind}: {}`, `when-at-limit`:", limit.count)
        });
    [lead, limit.when_at_limit.clone()].into_iter()
}

/// The words of a guidance line that holds one guidance as the scaffold
/// writes it, whatever its words, for a section, a preamble, or a
/// lifecycle dimension.
fn own_line(guidance: &str) -> String {
    squash(guidance).clone()
}

/// The texts joined by one space, each run of whitespace read as one
/// space, as a guidance line holds them, or `None` where there are none:
/// the text of a block's guidance line, a key's, and the opening's.
pub(super) fn joined(texts: impl Iterator<Item = String>) -> Option<String> {
    let texts: Vec<String> = texts
        .map(|t| squash(&t).clone())
        .filter(|t| !t.is_empty())
        .collect();
    (!texts.is_empty()).then(|| texts.join(" "))
}
