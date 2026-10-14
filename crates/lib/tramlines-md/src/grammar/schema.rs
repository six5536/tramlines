//! `Schema` and the rule structs, by reference to the grammar: one field
//! per key, in snake case. `lower` (in `lower.rs`) builds a `Schema` from
//! a valid YAML.

use std::collections::{BTreeMap, HashSet};

use regex::Regex;
use tramlines_core::grammar::Scope;

use super::{CoreSchema, Format, FormatRegistry, KeyDerivation, Only, RuleCommon, Shape, States};
use crate::{Spanned, markdown::BlockKind};

/// Grammar `heading`: a section's heading as a unit rule, its shape and
/// the key it captures (TLMD_27).
#[derive(Debug, Clone)]
pub struct Heading {
    /// The shape: exact text, the frontmatter value, a template or a regex.
    pub shape: Shape,
    /// The hole or named group whose capture is the section's key.
    pub key: Option<String>,
    /// `unique`: the scope of the key.
    pub unique: Scope,
    /// `key-derives-from`: the prefix the key takes.
    pub key_derives_from: Option<KeyDerivation>,
    /// `example`: a heading of the kind a template or a regex demands.
    pub example: Option<String>,
}

/// Grammar `section`.
#[derive(Debug, Clone)]
pub struct Section {
    /// The common keys.
    pub common: RuleCommon,
    /// The heading.
    pub heading: Heading,
    /// `renamed-from`.
    pub renamed_from: Vec<String>,
    /// `rule-id`.
    pub rule_id: Option<String>,
    /// `level`.
    pub level: u8,
    /// `optional`.
    pub optional: bool,
    /// `repeat`.
    pub repeat: bool,
    /// `headings-open`.
    pub headings_open: bool,
    /// `subheadings-open`.
    pub subheadings_open: bool,
    /// `for-each-key-of`: the rules whose keys the headings pair with, one
    /// rule when the schema names one.
    pub for_each_key_of: Option<Vec<String>>,
    /// `parent`: the `rule-id` of the section this one lies inside.
    pub parent: Option<String>,
    /// `number-from-parent`.
    pub number_from_parent: bool,
    /// `blocks`: absent means one prose block; `[]` an empty body.
    pub blocks: Option<Vec<Block>>,
    /// `blocks-ordered`.
    pub blocks_ordered: bool,
    /// `guidance`: what the example cannot say, written before the heading.
    pub guidance: Option<String>,
    /// `examples`: the further occurrences of a repeating section, each the
    /// whole markdown text of one occurrence, from its heading line on.
    pub examples: Vec<String>,
}

impl Section {
    /// The heading as the schema would print it.
    pub fn label(&self) -> String {
        self.heading.shape.label()
    }

    /// Whether the heading is written as the text itself, a literal or
    /// the frontmatter value, rather than a shape a heading may vary in.
    pub fn is_exact(&self) -> bool {
        matches!(self.heading.shape, Shape::Text(_))
    }
}

/// Grammar `preamble`.
#[derive(Debug, Clone)]
pub struct Preamble {
    /// The common keys.
    pub common: RuleCommon,
    /// `blocks`.
    pub blocks: Option<Vec<Block>>,
    /// `blocks-ordered`.
    pub blocks_ordered: bool,
    /// `guidance`: what the example cannot say, written before the preamble.
    pub guidance: Option<String>,
}

/// `lang` of a code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lang {
    /// Any tag; an untagged fence is still an error.
    Any,
    /// One of these tags.
    Tags(Vec<String>),
}

/// Grammar `block`.
#[derive(Debug, Clone)]
pub struct Block {
    /// The common keys.
    pub common: RuleCommon,
    /// `kind`.
    pub kind: BlockKind,
    /// `optional`.
    pub optional: bool,
    /// `repeat`.
    pub repeat: bool,
    /// `min`; absent means one.
    pub min: usize,
    /// `max`; absent means unbounded.
    pub max: Option<usize>,
    /// `each`.
    pub each: Option<Unit>,
    /// `sequence`.
    pub sequence: Option<Vec<Unit>>,
    /// `rules`: guarded rules over the block's units.
    pub rules: Option<Vec<Unit>>,
    /// `units-open`: a unit no rule of `rules` selects is accepted.
    pub units_open: bool,
    /// `lang`.
    pub lang: Option<Lang>,
    /// `fence`; absent means three.
    pub fence: usize,
    /// `columns`.
    pub columns: Option<Vec<String>>,
    /// `columns-open`.
    pub columns_open: bool,
    /// `source-pattern`.
    pub source_pattern: Option<Regex>,
    /// `inline-html`.
    pub inline_html: bool,
    /// `blocks`, of a group.
    pub blocks: Option<Vec<Block>>,
    /// `rule-id`.
    pub rule_id: Option<String>,
    /// `example`: the texts of this block's units, in order. A single string
    /// is a list of one.
    pub examples: Vec<String>,
    /// Whether `example` is written as a list rather than as one string.
    pub example_list: bool,
    /// `guidance`: what the example cannot say, written before the block.
    pub guidance: Option<String>,
    /// The block's position in the `blocks` list that declares it,
    /// one-based, which a finding names where the block has no `rule-id`.
    pub pos: usize,
}

/// Grammar `unit`.
#[derive(Debug, Clone)]
pub struct Unit {
    /// The common keys.
    pub common: RuleCommon,
    /// `text`, `template` or `pattern`: the unit's shape.
    pub shape: Option<Shape>,
    /// `format`: the name.
    pub format: Option<String>,
    /// `key`: the hole or named group of the shape, or of the format's,
    /// whose capture is the unit's key.
    pub key: Option<String>,
    /// `key-optional`.
    pub key_optional: bool,
    /// `unique`.
    pub unique: Scope,
    /// `key-repeats`: a later capture of a key the rule captured is a
    /// reference to it, not a clash.
    pub key_repeats: bool,
    /// `key-derives-from`: the prefix the key takes.
    pub key_derives_from: Option<KeyDerivation>,
    /// `rule-id`: the name the keys this rule captures belong to.
    pub rule_id: Option<String>,
    /// `for-each-key-of`: the rules whose keys this rule's keys pair with,
    /// one rule when the schema names one.
    pub for_each_key_of: Option<Vec<String>>,
    /// `prohibited-patterns`: what the unit's text may not match.
    pub prohibited_patterns: Vec<tramlines_core::grammar::Prohibition>,
    /// `optional`, in a sequence.
    pub optional: bool,
    /// `repeat`, in a sequence.
    pub repeat: bool,
    /// `blocks` of a list item.
    pub blocks: Option<Vec<Block>>,
    /// `lines`: rules over the content lines of a code block.
    pub lines: Option<Vec<Unit>>,
    /// `lines-open`: a line no rule of `lines` takes is accepted.
    pub lines_open: bool,
    /// `select-pattern`: the units or lines this rule governs, or counts.
    pub select: Option<Regex>,
    /// `min` selected lines; a guarded rule states it, else one.
    pub min: usize,
    /// `max` selected lines; absent means unbounded.
    pub max: Option<usize>,
    /// `example`: the texts of the units this rule governs, in order. A
    /// single string is a list of one.
    pub examples: Vec<String>,
    /// Whether `example` is written as a list rather than as one string.
    pub example_list: bool,
}

impl Unit {
    /// The format the rule binds, as `formats` defines it.
    pub(crate) fn format_in<'f>(&self, formats: &'f FormatRegistry) -> Option<&'f Format> {
        formats.get(self.format.as_deref()?)
    }

    /// Whether the squashed `text` has the rule's shape, its own or else its
    /// format's. `value` gives the frontmatter value a `text` is bound to.
    pub(crate) fn fits(
        &self,
        formats: &FormatRegistry,
        text: &str,
        value: impl Fn(&str) -> Option<String>,
    ) -> bool {
        match &self.shape {
            Some(shape) => shape.matches(text, value),
            None => self
                .format_in(formats)
                .is_some_and(|f| f.shape_for(text).is_some()),
        }
    }

    /// How far `text` follows the rule's shape, its own or else its
    /// format's, before it fails.
    pub(crate) fn reach(&self, formats: &FormatRegistry, text: &str) -> usize {
        match &self.shape {
            Some(shape) => shape.reach(text),
            None => self.format_in(formats).map_or(0, |f| f.reach(text)),
        }
    }

    /// The first text of the rule's `example`, which a failure quotes as the
    /// text of a valid unit.
    pub fn example(&self) -> Option<&str> {
        self.examples.first().map(String::as_str)
    }

    /// Whether the rule declares what a unit looks like, by its own shape
    /// or a format's.
    pub fn has_shape(&self) -> bool {
        self.shape.is_some() || self.format.is_some()
    }

    /// Whether the rule only counts what its guard selects: a guard and a
    /// count, and nothing that would check a unit (TLMD_52 to TLMD_57).
    pub fn is_counter(&self) -> bool {
        self.select.is_some()
            && !self.has_shape()
            && self.key.is_none()
            && self.prohibited_patterns.is_empty()
            && self.blocks.is_none()
            && self.lines.is_none()
            && self.for_each_key_of.is_none()
    }

    /// Whether the rule is a production: a shape and no guard, taking the
    /// units its shape matches (TLMD_52 to TLMD_57).
    pub fn is_production(&self) -> bool {
        self.select.is_none() && self.has_shape()
    }
}

/// Grammar `prohibited-heading`.
#[derive(Debug, Clone)]
pub struct ProhibitedHeading {
    /// The common keys.
    pub common: RuleCommon,
    /// `heading`.
    pub heading: String,
}

/// The scaffold a schema derives, the document a new one starts from: the
/// only kind of template a schema has.
#[derive(Debug, Clone)]
pub struct Scaffold {
    /// `document`.
    pub document: String,
    /// `lifecycle`; absent means every state.
    pub lifecycle: Option<Only>,
    /// The line in the schema.
    pub line: usize,
}

// sokf:begin schema
/// A lowered schema: grammar `schema`, regexes compiled, formats
/// resolved. The keys every format shares are in `core`.
#[derive(Debug, Clone)]
pub struct Schema {
    /// The keys of the core grammar.
    pub core: CoreSchema,
    /// `preamble`.
    pub preamble: Option<Preamble>,
    /// `sections`.
    pub sections: Vec<Section>,
    /// `sections-ordered`; absent means ordered.
    pub sections_ordered: bool,
    /// `sections-prohibited`.
    pub sections_prohibited: Vec<ProhibitedHeading>,
    /// The keys a host's extension adds, as written, for the host to lower.
    pub extensions: BTreeMap<String, Spanned>,
}
// sokf:end schema

impl Schema {
    /// A schema that demands nothing: the fallback for a schema that did
    /// not compile, and a base for tests.
    pub fn demand_nothing() -> Schema {
        Schema {
            core: CoreSchema::demand_nothing(),
            preamble: None,
            sections: vec![],
            sections_ordered: true,
            sections_prohibited: vec![],
            extensions: BTreeMap::new(),
        }
    }

    /// A document's lifecycle states, one per dimension its frontmatter
    /// carries.
    // sokf-implements: MD-2_AC-7
    pub fn states(&self, frontmatter: Option<&Spanned>) -> States {
        self.core.states(frontmatter)
    }

    /// Every rule id declared in the schema.
    pub fn rule_ids(&self) -> Vec<String> {
        self.rule_id_lines().into_iter().map(|(id, _)| id).collect()
    }

    /// Each `rule-id`, in schema order, with the line of its rule.
    pub fn rule_id_lines(&self) -> Vec<(String, usize)> {
        let mut ids = Vec::new();
        for s in &self.sections {
            ids.extend(s.rule_id.clone().map(|id| (id, s.common.line)));
            collect_block_ids(s.blocks.as_deref(), &mut ids);
        }
        if let Some(p) = &self.preamble {
            collect_block_ids(p.blocks.as_deref(), &mut ids);
        }
        ids
    }

    /// Whether unit rule `u` captures a key: it declares `key`, or binds a
    /// format that declares one.
    pub fn unit_captures_key(&self, u: &Unit) -> bool {
        u.key.is_some()
            || u.format
                .as_ref()
                .and_then(|name| self.core.formats.get(name))
                .is_some_and(|f| f.key.is_some())
    }

    /// Whether the rule `id` names captures a key. A key belongs to the
    /// nearest `rule-id` around the rule that captures it: the unit rule's
    /// own, else its block's, else the enclosing group's, else its
    /// section's; a section's heading key is the section's. This is the
    /// one definition every reference to a rule's keys (`for-each-key-of`,
    /// a host's rule) resolves through, and it mirrors how the check
    /// attributes a capture.
    pub fn rule_captures_key(&self, id: &str) -> Result<(), RuleLookup> {
        let mut owners = HashSet::new();
        for s in &self.sections {
            if s.heading.key.is_some()
                && let Some(r) = &s.rule_id
            {
                owners.insert(r.clone());
            }
            self.key_owners(s.blocks.as_deref(), None, s.rule_id.as_deref(), &mut owners);
        }
        if let Some(p) = &self.preamble {
            self.key_owners(p.blocks.as_deref(), None, None, &mut owners);
        }
        if owners.contains(id) {
            Ok(())
        } else if self.rule_ids().iter().any(|r| r == id) {
            Err(RuleLookup::NoKey)
        } else {
            Err(RuleLookup::NoRule)
        }
    }

    /// The rule ids that own a key captured under `blocks`, `group` being
    /// the enclosing group's id and `section` the section's.
    fn key_owners(
        &self,
        blocks: Option<&[Block]>,
        group: Option<&str>,
        section: Option<&str>,
        owners: &mut HashSet<String>,
    ) {
        for b in blocks.unwrap_or(&[]) {
            if b.kind == BlockKind::Group {
                let group = b.rule_id.as_deref().or(group);
                self.key_owners(b.blocks.as_deref(), group, section, owners);
                continue;
            }
            for u in b.units() {
                self.unit_key_owners(u, b, group, section, owners);
            }
        }
    }

    fn unit_key_owners(
        &self,
        u: &Unit,
        block: &Block,
        group: Option<&str>,
        section: Option<&str>,
        owners: &mut HashSet<String>,
    ) {
        if self.unit_captures_key(u)
            && let Some(owner) = u
                .rule_id
                .as_deref()
                .or(block.rule_id.as_deref())
                .or(group)
                .or(section)
        {
            owners.insert(owner.to_string());
        }
        self.key_owners(u.blocks.as_deref(), group, section, owners);
        for line in u.lines.iter().flatten() {
            self.unit_key_owners(line, block, group, section, owners);
        }
    }
}

/// Why `Schema::rule_captures_key` found no rule that captures a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleLookup {
    /// No rule of the schema carries the id.
    NoRule,
    /// A rule carries the id, and captures no key.
    NoKey,
}

impl RuleLookup {
    /// The refusal, for the key `what` that named rule `id`.
    pub fn message(self, what: &str, id: &str) -> String {
        match self {
            RuleLookup::NoRule => format!("`{what}` names no rule `{id}`"),
            RuleLookup::NoKey => format!("`{what}` names rule `{id}`, which captures no key"),
        }
    }
}

fn collect_block_ids(blocks: Option<&[Block]>, ids: &mut Vec<(String, usize)>) {
    for b in blocks.unwrap_or(&[]) {
        ids.extend(b.rule_id.clone().map(|id| (id, b.common.line)));
        collect_block_ids(b.blocks.as_deref(), ids);
        for u in b.units() {
            collect_unit_ids(u, ids);
        }
    }
}

fn collect_unit_ids(u: &Unit, ids: &mut Vec<(String, usize)>) {
    ids.extend(u.rule_id.clone().map(|id| (id, u.common.line)));
    collect_block_ids(u.blocks.as_deref(), ids);
    for line in u.lines.iter().flatten() {
        collect_unit_ids(line, ids);
    }
}

impl Block {
    /// The first text of the block's `example`.
    pub fn example(&self) -> Option<&str> {
        self.examples.first().map(String::as_str)
    }

    /// The block's unit rules: `each`, the positions of `sequence`, and
    /// `rules`. The rules under a unit's `lines` are the unit's own.
    pub fn units(&self) -> impl Iterator<Item = &Unit> {
        self.each
            .iter()
            .chain(self.sequence.iter().flatten())
            .chain(self.rules.iter().flatten())
    }
}
