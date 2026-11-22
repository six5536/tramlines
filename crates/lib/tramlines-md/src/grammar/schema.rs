//! `Schema` and the rule structs, by reference to the grammar: one field
//! per key, in snake case. `lower` (in `lower.rs`) builds a `Schema` from
//! a valid YAML.

use std::collections::{BTreeMap, HashSet};

use regex::Regex;
use tramlines_core::grammar::Scope;

use super::{
    Condition, CoreSchema, DerivationSource, Format, FormatRegistry, KeyDerivation, Label,
    LifecycleDimension, Limit, Only, RuleCommon, RuleLookup, Shape, States, Words, captured,
    squash,
};
use crate::{Spanned, markdown::BlockKind};

/// Grammar `heading`: a section's heading as a unit rule, its shape and
/// the key it captures (`TLMD_27`).
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
    /// `optional`: the condition in which the section can be absent, which
    /// admits every state for `true`; `None` where it is always required.
    pub optional: Option<Condition>,
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

/// Whether a part whose `optional` is `optional` can be absent from a
/// document in `states`: never for `None`, and otherwise where the
/// condition admits the states, as [`Condition::admits_known`] reads them.
// sokf-implements: TLMD_229
// sokf-implements: TLMD_230
fn optional_at(
    optional: Option<&Condition>,
    states: &States,
    lifecycle: &[LifecycleDimension],
) -> bool {
    optional.is_some_and(|c| c.admits_known(states, lifecycle))
}

impl Section {
    /// Whether the section can be absent from a document in `states`: where
    /// the condition of its `optional` admits them.
    pub fn optional_at(&self, states: &States, lifecycle: &[LifecycleDimension]) -> bool {
        optional_at(self.optional.as_ref(), states, lifecycle)
    }

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
    /// `optional`: the condition in which the block can be absent, which
    /// admits every state for `true`; `None` where it is always required.
    pub optional: Option<Condition>,
    /// `repeat`.
    pub repeat: bool,
    /// `min`; absent means one.
    pub min: usize,
    /// `max`: the limit on the units; absent means unbounded.
    pub max: Option<Limit>,
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
    /// `example-columns`: the header row of the scaffold of a table with
    /// no declared columns.
    pub example_columns: Option<Vec<String>>,
    /// `source-pattern`.
    pub source_pattern: Option<Regex>,
    /// `inline-html`.
    pub inline_html: bool,
    /// `label`: whether each unit carries a label, where its unit rule
    /// declares no `label` of its own.
    pub label: Option<Label>,
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
    /// `rule-id`: the rule's name, which no reference can use, and under
    /// which no key is filed (`TLMD_236`).
    pub rule_id: Option<String>,
    /// `key-of`: the name that the keys this rule captures belong to, in
    /// place of the id of its block, its group, or its section (`TLMD_89`).
    pub key_of: Option<String>,
    /// `for-each-key-of`: the names whose keys this rule's keys pair with,
    /// one name when the schema gives one.
    pub for_each_key_of: Option<Vec<String>>,
    /// `prohibited-patterns`: what the unit's text may not match.
    pub prohibited_patterns: Vec<tramlines_core::grammar::Prohibition>,
    /// `label`: whether each unit the rule governs carries a label, in
    /// place of the `label` of its block.
    pub label: Option<Label>,
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
    /// `max`: the limit on the selected units or lines; absent means
    /// unbounded.
    pub max: Option<Limit>,
    /// `example`: the texts of the units this rule governs, in order. A
    /// single string is a list of one.
    pub examples: Vec<String>,
    /// Whether `example` is written as a list rather than as one string.
    pub example_list: bool,
    /// The keys a host's extension adds to the rule, as written, for the
    /// host to lower.
    pub extensions: BTreeMap<String, Spanned>,
    /// The rule's place among the unit rules of its schema, counted from
    /// zero in the order the schema writes them, which `Schema::unit` finds
    /// the rule by. Two rules on one line of the schema have two places.
    pub index: usize,
}

impl Unit {
    /// The name that a key this rule captures is filed under: its
    /// `key-of`, else the first of `around`, the `rule-id`s of its block,
    /// its group, and its section, in that order. The rule's own
    /// `rule-id` files nothing. The check, the derivation, and the
    /// lookup of a reference all file a key through this.
    // sokf-implements: TLMD_89
    // sokf-implements: TLMD_236
    pub(crate) fn files_under<'a>(&'a self, around: &[Option<&'a str>]) -> Option<&'a str> {
        self.key_of.as_deref().or_else(|| nearest(around))
    }

    /// The `rule-id` that names the rule in a finding on its capture: its
    /// own, else the first of `around`, as `files_under` reads them.
    pub(crate) fn named_by<'a>(&'a self, around: &[Option<&'a str>]) -> Option<&'a str> {
        self.rule_id.as_deref().or_else(|| nearest(around))
    }

    /// The most units or lines the rule may take or count: the count of a
    /// hard `max`, and `None` for a soft `max` or no `max`, neither of
    /// which bounds anything.
    pub fn hard_max(&self) -> Option<usize> {
        self.max.as_ref().and_then(Limit::hard)
    }

    /// The format the rule binds, as `formats` defines it.
    pub(crate) fn format_in<'f>(&self, formats: &'f FormatRegistry) -> Option<&'f Format> {
        formats.get(self.format.as_deref()?)
    }

    /// Whether the rule, or the format it binds as `formats` defines it,
    /// bans any text.
    pub(crate) fn bans_in(&self, formats: &FormatRegistry) -> bool {
        !self.prohibited_patterns.is_empty()
            || self
                .format_in(formats)
                .is_some_and(|f| !f.prohibited.is_empty())
    }

    /// The key the rule captures from the squashed `text`: its own `key` of
    /// its own shape, or else the key of the shape of its format that
    /// `text` takes.
    pub(crate) fn key_in(&self, formats: &FormatRegistry, text: &str) -> Option<String> {
        if let Some(shape) = &self.shape {
            return Some(captured(shape, self.key.as_deref()?, text)?.key);
        }
        let format = self.format_in(formats)?;
        let name = self.key.as_ref().or(format.key.as_ref())?;
        let (shape, _) = format.shape_for(text)?;
        Some(captured(shape, name, text)?.key)
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
    /// count, and nothing that would check a unit (`TLMD_52` to `TLMD_57`).
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
    /// units its shape matches (`TLMD_52` to `TLMD_57`).
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
    /// `labels`: the key of each label that a unit can carry, in order.
    pub labels: Vec<String>,
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
            labels: vec![],
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
        self.each_name(&mut |kind, id, line, _| {
            if kind != Name::KeyOf {
                ids.push((id.to_string(), line));
            }
        });
        ids
    }

    /// Call `f` with each name that the schema declares, in schema order:
    /// what declares it, the name, the line of its rule, and the unit rule
    /// where a unit rule declares it.
    fn each_name<'s>(&'s self, f: &mut impl FnMut(Name, &str, usize, Option<&'s Unit>)) {
        for s in &self.sections {
            if let Some(id) = &s.rule_id {
                f(Name::Referable, id, s.common.line, None);
            }
            each_block_name(s.blocks.as_deref(), f);
        }
        if let Some(p) = &self.preamble {
            each_block_name(p.blocks.as_deref(), f);
        }
    }

    /// The unit rule whose `Unit::index` is `index`: a rule of a block, of
    /// an item's blocks, or of a fence's `lines`.
    // sokf-implements: MD-2_AC-10
    pub fn unit(&self, index: usize) -> Option<&Unit> {
        let blocks = self
            .sections
            .iter()
            .filter_map(|s| s.blocks.as_deref())
            .chain(self.preamble.iter().filter_map(|p| p.blocks.as_deref()));
        let mut pending: Vec<&Block> = blocks.flatten().collect();
        while let Some(b) = pending.pop() {
            pending.extend(b.blocks.iter().flatten());
            let mut units: Vec<&Unit> = b.units().collect();
            while let Some(u) = units.pop() {
                if u.index == index {
                    return Some(u);
                }
                units.extend(u.lines.iter().flatten());
                pending.extend(u.blocks.iter().flatten());
            }
        }
        None
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

    /// Whether the name `id` holds a key. A key belongs to the `key-of` of
    /// the unit rule that captures it, else to the nearest `rule-id`
    /// around the rule: its block's, else the enclosing group's, else its
    /// section's; a section's heading key is the section's. A unit rule's
    /// own `rule-id` holds no key (`TLMD_236`). This is the one definition
    /// every reference to a set of keys (`for-each-key-of`,
    /// `key-derives-from`, a host's rule) resolves through, and it mirrors
    /// how the check attributes a capture (`TLMD_238`).
    // sokf-implements: TLMD_238
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
            return Ok(());
        }
        // A unit rule's `rule-id` on a rule that captures a key is the
        // likeliest meaning of the name, as a schema written before
        // `key-of` means it.
        let (mut unit, mut declared) = (false, false);
        self.each_name(&mut |kind, name, _, rule| {
            if name == id {
                // A pairing rule's keys are references, so they make no set.
                unit |= kind == Name::Unit
                    && rule
                        .is_some_and(|u| u.for_each_key_of.is_none() && self.unit_captures_key(u));
                declared = true;
            }
        });
        Err(if unit {
            RuleLookup::NameOnly
        } else if declared {
            RuleLookup::NoKey
        } else {
            RuleLookup::NoRule
        })
    }

    /// Every name that the schema declares: each `rule-id` and each
    /// `key-of`.
    pub(crate) fn declared_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        self.each_name(&mut |_, id, _, _| {
            names.insert(id.to_string());
        });
        names
    }

    /// Each `rule-id` that a reference can name, of a section or a block,
    /// in schema order, with the line of its rule (`TL_15`).
    pub(crate) fn referable_id_lines(&self) -> Vec<(String, usize)> {
        let mut ids = Vec::new();
        self.each_name(&mut |kind, id, line, _| {
            if kind == Name::Referable {
                ids.push((id.to_string(), line));
            }
        });
        ids
    }

    /// The keys that the examples of the rule `id` names capture, owned as
    /// `rule_captures_key` owns them: the heading example of the section,
    /// or the example of each unit rule, else its format's, else its
    /// block's. These are the parents that the example of a key deriving
    /// from the rule starts with (`TL_127`, `TLMD_205`).
    pub(crate) fn example_keys(&self, id: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(p) = &self.preamble {
            self.example_keys_in(p.blocks.as_deref(), id, None, None, &mut out);
        }
        for s in &self.sections {
            if s.rule_id.as_deref() == Some(id)
                && let (Some(key), Some(example)) = (&s.heading.key, &s.heading.example)
                && let Some(got) = captured(&s.heading.shape, key, &squash(example))
            {
                out.push(got.key);
            }
            self.example_keys_in(
                s.blocks.as_deref(),
                id,
                None,
                s.rule_id.as_deref(),
                &mut out,
            );
        }
        out
    }

    /// The examples of the parent that `kd` names, which the example of a
    /// key deriving through it starts with, in schema order and each once:
    /// the value that the scaffold at the first states writes for the
    /// frontmatter key from the schema, or the keys of the examples of the
    /// rule. Empty where the schema states no value of the parent, as for a
    /// key whose value only the host seeds.
    // sokf-implements: TL_129
    pub(crate) fn parent_examples(&self, kd: &KeyDerivation) -> Vec<String> {
        let mut all = match &kd.source {
            DerivationSource::Rule(r) => self.example_keys(r),
            DerivationSource::From(k) => self
                .core
                .constraint(k)
                .and_then(|c| c.scaffold_value(&States::default()))
                .into_iter()
                .collect(),
        };
        let mut seen = HashSet::new();
        all.retain(|p| seen.insert(p.clone()));
        all
    }

    /// `example_keys` under `blocks`, `group` and `section` being the ids
    /// that own a key no nearer `rule-id` owns.
    fn example_keys_in(
        &self,
        blocks: Option<&[Block]>,
        id: &str,
        group: Option<&str>,
        section: Option<&str>,
        out: &mut Vec<String>,
    ) {
        let words = Words::keys_of(self);
        for b in blocks.unwrap_or(&[]) {
            if b.kind == BlockKind::Group {
                let group = b.rule_id.as_deref().or(group);
                self.example_keys_in(b.blocks.as_deref(), id, group, section, out);
                continue;
            }
            // The rules in schema order, each rule's line rules after it.
            let mut units: Vec<&Unit> = b.units().collect();
            let mut next = 0;
            while let Some(&u) = units.get(next) {
                next += 1;
                units.extend(u.lines.iter().flatten());
                self.example_keys_in(u.blocks.as_deref(), id, group, section, out);
                let owner = u.files_under(&[b.rule_id.as_deref(), group, section]);
                // A rule that derives from the name gives it no parent.
                let derives_here = u
                    .key_derives_from
                    .as_ref()
                    .is_some_and(|d| matches!(&d.source, DerivationSource::Rule(r) if r == id));
                if owner != Some(id) || derives_here {
                    continue;
                }
                let formats = &self.core.formats;
                let texts: Vec<&str> = if !u.examples.is_empty() {
                    u.examples.iter().map(String::as_str).collect()
                } else if let Some(f) = u.format_in(formats) {
                    vec![f.first_example()]
                } else {
                    b.examples.iter().map(String::as_str).collect()
                };
                // An example's key is read without its label (TLMD_248).
                out.extend(
                    texts.into_iter().filter_map(|t| {
                        u.key_in(formats, &self.unlabelled(&words, b, Some(u), t).0)
                    }),
                );
            }
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
            && let Some(owner) = u.files_under(&[block.rule_id.as_deref(), group, section])
        {
            owners.insert(owner.to_string());
        }
        self.key_owners(u.blocks.as_deref(), group, section, owners);
        for line in u.lines.iter().flatten() {
            self.unit_key_owners(line, block, group, section, owners);
        }
    }
}

/// The first id of `around`, the ids around a rule, nearest first.
fn nearest<'a>(around: &[Option<&'a str>]) -> Option<&'a str> {
    around.iter().find_map(|a| *a)
}

/// What declares a name of a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Name {
    /// The `rule-id` of a section or a block, which a reference can name.
    Referable,
    /// The `rule-id` of a unit rule or a line rule, which names the rule.
    Unit,
    /// The `key-of` of a unit rule or a line rule.
    KeyOf,
}

/// Call `f` with each name declared under `blocks`, at any depth.
fn each_block_name<'s>(
    blocks: Option<&'s [Block]>,
    f: &mut impl FnMut(Name, &str, usize, Option<&'s Unit>),
) {
    for b in blocks.unwrap_or(&[]) {
        if let Some(id) = &b.rule_id {
            f(Name::Referable, id, b.common.line, None);
        }
        each_block_name(b.blocks.as_deref(), f);
        for u in b.units() {
            each_unit_name(u, f);
        }
    }
}

fn each_unit_name<'s>(u: &'s Unit, f: &mut impl FnMut(Name, &str, usize, Option<&'s Unit>)) {
    if let Some(id) = &u.rule_id {
        f(Name::Unit, id, u.common.line, Some(u));
    }
    if let Some(id) = &u.key_of {
        f(Name::KeyOf, id, u.common.line, Some(u));
    }
    each_block_name(u.blocks.as_deref(), f);
    for line in u.lines.iter().flatten() {
        each_unit_name(line, f);
    }
}

impl Block {
    /// Whether the block can be absent from a document in `states`: where
    /// the condition of its `optional` admits them.
    pub fn optional_at(&self, states: &States, lifecycle: &[LifecycleDimension]) -> bool {
        optional_at(self.optional.as_ref(), states, lifecycle)
    }

    /// The inner blocks of a group that `states` admit, in order.
    pub(crate) fn admitted<'a>(
        &'a self,
        states: &'a States,
    ) -> impl DoubleEndedIterator<Item = &'a Block> + 'a {
        self.blocks
            .iter()
            .flatten()
            .filter(move |b| b.common.only.admits(states))
    }

    /// The position that the block opens with at `states`: a group's first
    /// admitted inner position, followed into a group that opens a group,
    /// else the block itself.
    pub(crate) fn first_position<'a>(&'a self, states: &'a States) -> Option<&'a Block> {
        if self.kind != BlockKind::Group {
            return Some(self);
        }
        self.admitted(states).next()?.first_position(states)
    }

    /// The most units the block may hold: the count of a hard `max`, and
    /// `None` for a soft `max` or no `max`, neither of which bounds
    /// anything.
    pub fn hard_max(&self) -> Option<usize> {
        self.max.as_ref().and_then(Limit::hard)
    }

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
