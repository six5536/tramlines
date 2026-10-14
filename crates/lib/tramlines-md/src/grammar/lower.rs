//! `lower` builds a typed `Schema` from a schema YAML that passed the
//! generic check: the core keys through tramlines-core, the markdown keys
//! here. `compile` does both.

use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use tramlines_core::grammar::{
    CORE_SCHEMA_KEYS, Lowerer, Scope, bool_of, derivation, extensions, list_of, lower_formats,
    str_of, usize_of, validate_schema,
};

use super::{
    Block, FormatRegistry, Grammar, Lang, Preamble, ProhibitedHeading, RegexCache, RuleCommon,
    Schema, Section, Unit, squash,
};
use crate::{
    Finding, RepoPath, Severity, Spanned,
    markdown::{BlockKind, Dialect},
};

/// The keys of `schema` the markdown format grammar adds.
const MD_SCHEMA_KEYS: &[&str] = &[
    "preamble",
    "sections",
    "sections-ordered",
    "sections-prohibited",
];

/// Lower a schema that passed `validate_schema`: the schema, with every
/// semantic refusal the generic check cannot see, whether or not there is
/// one. `dialect` names the host's region kind.
// sokf-implements: TL_11
pub fn lower(
    yaml: &Spanned,
    grammar: &Grammar,
    registry: &FormatRegistry,
    regexes: &RegexCache,
    schema: &RepoPath,
    schema_id: &str,
    dialect: &Dialect,
) -> (Schema, Vec<Finding>) {
    let mut findings = Vec::new();
    // sokf-implements: MD-2_AC-5
    // A schema's own formats and types are its nearest layer.
    let formats = lower_formats(
        yaml,
        grammar,
        registry,
        regexes,
        schema,
        schema_id,
        &mut findings,
    );
    // A format the schema declares and that did not load has its own
    // finding already.
    let unloaded = yaml
        .get("formats")
        .and_then(|f| f.as_map())
        .unwrap_or(&[])
        .iter()
        .map(|(name, _)| name.value.clone())
        .filter(|name| formats.get(name).is_none())
        .collect();
    let mut l = MdLowerer {
        core: Lowerer::new(regexes, schema, formats.types_seen()),
        dialect: *dialect,
        severity: Severity::Error,
        unloaded,
    };
    l.findings = findings;
    let schema = l.schema(yaml, formats);
    (schema, l.core.findings)
}

/// The lowering of a markdown schema: the core lowering, the dialect that
/// names a host's region kind, the `severity` of the rule being lowered,
/// which the rules inside it take unless they declare their own, and the
/// formats the schema declares that did not load.
struct MdLowerer<'a> {
    core: Lowerer<'a>,
    dialect: Dialect,
    severity: Severity,
    unloaded: HashSet<String>,
}

impl<'a> Deref for MdLowerer<'a> {
    type Target = Lowerer<'a>;

    fn deref(&self) -> &Lowerer<'a> {
        &self.core
    }
}

impl DerefMut for MdLowerer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

/// Where a unit rule sits, which decides the keys it may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitPos {
    /// `each`, or one position of a `sequence`: no guard, no bounds.
    Plain,
    /// One entry of a block's `rules`: a guard, its bounds, and nesting.
    Guarded,
    /// One entry of a unit's `lines`: a guard and its bounds, no nesting.
    Line,
}

impl MdLowerer<'_> {
    /// The keys every rule map carries, a `severity` the map does not
    /// declare taken from the rule it sits in.
    // sokf-implements: TL_18
    fn rule_common(&mut self, node: &Spanned) -> RuleCommon {
        let mut common = self.common(node);
        if node.get("severity").is_none() {
            common.severity = self.severity;
        }
        common
    }

    /// Lower what lies inside a rule with `common`, at its severity.
    fn inside<T>(&mut self, common: &RuleCommon, f: impl FnOnce(&mut Self) -> T) -> T {
        let outer = std::mem::replace(&mut self.severity, common.severity);
        let out = f(self);
        self.severity = outer;
        out
    }

    fn schema(&mut self, yaml: &Spanned, formats: FormatRegistry) -> Schema {
        let core = self.core.lower_core(yaml, formats);
        let preamble = yaml.get("preamble").map(|p| {
            let common = self.rule_common(p);
            Preamble {
                blocks: self.inside(&common, |l| l.blocks(p)),
                common,
                blocks_ordered: ordered(p, "blocks-ordered"),
                guidance: str_of(p, "guidance"),
            }
        });
        let sections: Vec<Section> = seq(yaml, "sections")
            .map(|items| items.iter().map(|s| self.section(s)).collect())
            .unwrap_or_default();
        let sections_ordered = ordered(yaml, "sections-ordered");
        // sokf-implements: TLMD_21
        if sections_ordered {
            for w in sections.windows(2) {
                if w[1].level > w[0].level + 1 {
                    self.fail(
                        w[1].common.line,
                        "TLMD_21",
                        format!(
                            "section `{}` skips a level after `{}`",
                            w[1].label(),
                            w[0].label()
                        ),
                    );
                }
            }
        }
        let sections_prohibited: Vec<ProhibitedHeading> = seq(yaml, "sections-prohibited")
            .map(|items| {
                items
                    .iter()
                    .map(|p| ProhibitedHeading {
                        common: self.rule_common(p),
                        heading: str_of(p, "heading").unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        // A heading a section declares is matched as that section, even in
        // an open region, so a ban on its text could never apply.
        // sokf-implements: TLMD_170
        for banned in &sections_prohibited {
            let declared = sections.iter().any(|s| {
                s.common.only.overlaps(&banned.common.only)
                    && s.heading.shape.matches(&squash(&banned.heading), |_| None)
            });
            if declared {
                self.fail(
                    banned.common.line,
                    "TLMD_170",
                    format!(
                        "`sections-prohibited` names `{}`, which a section declares",
                        banned.heading
                    ),
                );
            }
        }
        // References resolve last, once every rule id is known.
        let mut known = CORE_SCHEMA_KEYS.to_vec();
        known.extend(MD_SCHEMA_KEYS);
        let schema = Schema {
            core,
            preamble,
            sections,
            sections_ordered,
            sections_prohibited,
            // sokf-implements: TL_9
            // sokf-implements: MD-2_AC-6
            extensions: extensions(yaml, &known),
        };
        self.check_references(&schema);
        schema
    }

    fn section(&mut self, node: &Spanned) -> Section {
        let heading = self.heading(node);
        // Guidance and the example are one voice each, the example now on
        // the heading the section's guidance precedes.
        if let Some(g) = str_of(node, "guidance") {
            self.one_voice_in(node.line, &g, heading.example.clone());
        }
        if node.get("for-each-key-of").is_some() && heading.key.is_none() {
            self.fail(
                node.line,
                "TLMD_98",
                "`for-each-key-of` pairs headings by their key: the heading declares `key`",
            );
        }
        if let Some(id) = str_of(node, "rule-id") {
            self.rule_ids.insert(id);
        }
        let common = self.rule_common(node);
        let blocks = self.inside(&common, |l| l.blocks(node));
        let level = node
            .get("level")
            .and_then(|l| l.as_int())
            .and_then(|l| u8::try_from(l).ok())
            .unwrap_or(1);
        let examples = self.occurrences(node, usize::from(level));
        Section {
            common,
            heading,
            renamed_from: list_of(node, "renamed-from"),
            rule_id: str_of(node, "rule-id"),
            level,
            optional: bool_of(node, "optional"),
            repeat: bool_of(node, "repeat"),
            headings_open: bool_of(node, "headings-open"),
            subheadings_open: bool_of(node, "subheadings-open"),
            for_each_key_of: node
                .get("for-each-key-of")
                .and_then(Spanned::as_str_or_list),
            parent: str_of(node, "parent"),
            number_from_parent: bool_of(node, "number-from-parent"),
            blocks,
            blocks_ordered: ordered(node, "blocks-ordered"),
            guidance: str_of(node, "guidance"),
            examples,
        }
    }

    /// The `examples` of a section: the further occurrences that the
    /// scaffold writes after the one it derives. Only a section that repeats
    /// has further occurrences, so one that doesn't refuses `examples`. Each
    /// text begins with a heading of the section's level,
    /// because the scaffold writes the text as the whole occurrence.
    fn occurrences(&mut self, node: &Spanned, level: usize) -> Vec<String> {
        let Some(list) = node.get("examples") else {
            return Vec::new();
        };
        if !bool_of(node, "repeat") {
            self.fail(
                list.line,
                "TLMD_180",
                "`examples` holds further occurrences of a section, so the section declares `repeat: true`",
            );
        }
        let mut out = Vec::new();
        for (i, item) in list.as_seq().unwrap_or(&[]).iter().enumerate() {
            let text = item.as_str().unwrap_or_default();
            let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            let hashes = first.len() - first.trim_start_matches('#').len();
            if hashes != level || !first[hashes..].starts_with(' ') {
                self.fail(
                    item.line,
                    "TLMD_181",
                    format!(
                        "item {} of `examples` begins with `{first}`, but each occurrence begins with a heading of level {level}",
                        i + 1
                    ),
                );
            }
            out.push(text.to_string());
        }
        out
    }

    /// The texts of the `example` of a block or a unit rule, and whether the
    /// schema writes them as a list. A single string is a list of one text.
    /// An empty list gives the scaffold nothing to write, so it is refused.
    fn examples_of(&mut self, node: &Spanned) -> (Vec<String>, bool) {
        let Some(value) = node.get("example") else {
            return (Vec::new(), false);
        };
        let listed = value.as_seq().is_some();
        let texts = list_of(node, "example");
        if listed && texts.is_empty() {
            self.fail(
                value.line,
                "TLMD_182",
                "`example` is an empty list: write at least one text, or leave `example` out",
            );
        }
        (texts, listed)
    }

    fn blocks(&mut self, node: &Spanned) -> Option<Vec<Block>> {
        let items = node.get("blocks")?.as_seq()?;
        Some(
            items
                .iter()
                .enumerate()
                .map(|(i, b)| self.block(b, i + 1))
                .collect(),
        )
    }

    fn block(&mut self, node: &Spanned, pos: usize) -> Block {
        self.explained(node, &["source-pattern"], true);
        // The scaffold writes a block's guidance before its units, so the
        // guidance is one voice with each unit's example as well as its own.
        // sokf-implements: TL_68
        if let Some(g) = str_of(node, "guidance") {
            let units = node.get("each").into_iter().chain(
                ["sequence", "rules"]
                    .into_iter()
                    .filter_map(|k| seq(node, k))
                    .flatten(),
            );
            let examples = list_of(node, "example")
                .into_iter()
                .chain(units.flat_map(|u| list_of(u, "example")));
            self.one_voice_in(node.line, &g, examples);
        }
        let kind = str_of(node, "kind")
            .and_then(|k| BlockKind::parse_in(&k, &self.dialect))
            .unwrap_or(BlockKind::Prose);
        let lang = node
            .get("lang")
            .and_then(|l| l.as_str_or_list())
            .map(|tags| {
                if tags.len() == 1 && tags[0] == "any" {
                    Lang::Any
                } else {
                    Lang::Tags(tags)
                }
            });
        let common = self.rule_common(node);
        let (examples, example_list) = self.examples_of(node);
        let (each, sequence, rules, blocks) = self.inside(&common, |l| {
            (
                node.get("each").map(|u| l.unit(u, UnitPos::Plain)),
                seq(node, "sequence")
                    .map(|items| items.iter().map(|u| l.unit(u, UnitPos::Plain)).collect()),
                seq(node, "rules")
                    .map(|items| items.iter().map(|u| l.unit(u, UnitPos::Guarded)).collect()),
                l.blocks(node),
            )
        });
        Block {
            common,
            kind,
            optional: bool_of(node, "optional"),
            repeat: bool_of(node, "repeat"),
            min: usize_of(node, "min").unwrap_or(1),
            max: usize_of(node, "max"),
            each,
            sequence,
            rules,
            units_open: bool_of(node, "units-open"),
            lang,
            fence: usize_of(node, "fence").unwrap_or(3),
            columns: node.get("columns").and_then(|c| c.as_str_or_list()),
            columns_open: bool_of(node, "columns-open"),
            source_pattern: self.regex(node, "source-pattern"),
            inline_html: bool_of(node, "inline-html"),
            blocks,
            rule_id: str_of(node, "rule-id"),
            examples,
            example_list,
            guidance: str_of(node, "guidance"),
            pos,
        }
    }

    /// One unit rule, at the position that decides which keys it may carry.
    fn unit(&mut self, node: &Spanned, pos: UnitPos) -> Unit {
        let select = self.regex(node, "select-pattern");
        let guarded = matches!(pos, UnitPos::Guarded | UnitPos::Line);
        // sokf-implements: TLMD_91
        let refusals: [(&str, bool, &'static str, &str); 5] = [
            (
                "select-pattern",
                !guarded,
                "TLMD_91",
                "`select-pattern` applies to a rule under `rules` or `lines`, which chooses what it governs",
            ),
            (
                "min",
                !guarded,
                "TLMD_91",
                "`min` applies to a rule under `rules` or `lines`; elsewhere the block bounds its units",
            ),
            (
                "max",
                !guarded,
                "TLMD_91",
                "`max` applies to a rule under `rules` or `lines`; elsewhere the block bounds its units",
            ),
            (
                "lines",
                matches!(pos, UnitPos::Line),
                "TLMD_90",
                "`lines` does not nest",
            ),
            (
                "blocks",
                matches!(pos, UnitPos::Line),
                "TLMD_42",
                "`blocks` applies to an item, not to a line",
            ),
        ];
        for (key, refused, criterion, message) in refusals {
            if refused && node.get(key).is_some() {
                self.fail(node.line, criterion, message);
            }
        }
        // A guard says which units a rule governs, not that there is one of
        // them, so the rule states for itself whether it demands. One key
        // settled by a sibling key reads as a demand it does not make.
        // sokf-implements: TLMD_92
        // A rule under `lines` is held too: its example is what the failure
        // of a line shows. A guard alone declares no shape.
        self.explained(node, &["pattern", "template"], true);
        let unstated_guard = guarded && node.get("select-pattern").is_some();
        if unstated_guard && node.get("min").is_none() {
            self.fail(
                node.line,
                "TLMD_92",
                "a guarded rule states `min`: `min: 0` governs such units without demanding one, and a number demands that many",
            );
        }
        let min = match usize_of(node, "min") {
            Some(n) => n,
            // One default, whatever sits beside it: a rule demands one unit.
            // A guarded rule that stated none was reported above and the
            // schema will not compile; it takes zero here so that one
            // omission raises one finding and not a bound nobody wrote.
            // sokf-implements: TLMD_85
            None if unstated_guard => 0,
            None => 1,
        };
        // A guard selects by what a unit starts with; `text` replaces the
        // unit whole, so the repair would drop what the guard read.
        if select.is_some() && node.get("text").is_some() {
            // sokf-implements: TLMD_142
            self.fail(
                node.line,
                "TLMD_142",
                "`text` beside `select-pattern`: the repair rewrites the unit whole and would drop what the guard selected on",
            );
        }
        let common = self.rule_common(node);
        let (examples, example_list) = self.examples_of(node);
        let (blocks, lines) = self.inside(&common, |l| {
            (
                l.blocks(node),
                seq(node, "lines")
                    .map(|items| items.iter().map(|u| l.unit(u, UnitPos::Line)).collect()),
            )
        });
        Unit {
            common,
            shape: self.shape(node),
            format: str_of(node, "format"),
            key: str_of(node, "key"),
            key_optional: bool_of(node, "key-optional"),
            unique: Scope::parse(str_of(node, "unique").as_deref()),
            key_repeats: bool_of(node, "key-repeats"),
            key_derives_from: node.get("key-derives-from").map(derivation),
            rule_id: str_of(node, "rule-id"),
            for_each_key_of: node
                .get("for-each-key-of")
                .and_then(Spanned::as_str_or_list),
            prohibited_patterns: self.prohibitions(node),
            optional: bool_of(node, "optional"),
            repeat: bool_of(node, "repeat"),
            blocks,
            lines,
            lines_open: bool_of(node, "lines-open"),
            min,
            max: usize_of(node, "max"),
            select,
            examples,
            example_list,
        }
    }
}

/// Validate and lower in one step.
// sokf:begin compile
pub fn compile(
    yaml: &Spanned,
    grammar: &Grammar,
    registry: &FormatRegistry,
    regexes: &RegexCache,
    schema: &RepoPath,
    schema_id: &str,
    dialect: &Dialect,
) -> std::result::Result<Schema, Vec<Finding>> {
    // sokf:end compile
    // sokf-implements: MD-2_AC-1
    // sokf-implements: MD-2_AC-2
    let problems = validate_schema(yaml, grammar, regexes, schema);
    if !problems.is_empty() {
        return Err(problems);
    }
    let (schema, findings) = lower(yaml, grammar, registry, regexes, schema, schema_id, dialect);
    if findings.is_empty() {
        // sokf-implements: MD-2_AC-4
        Ok(schema)
    } else {
        // sokf-implements: MD-2_AC-3
        Err(findings)
    }
}

mod checks;
mod shapes;

/// Whether a `*-ordered` key of `node` holds its parts in order, which it
/// does unless it says otherwise.
fn ordered(node: &Spanned, key: &str) -> bool {
    node.get(key).and_then(|b| b.as_bool()).unwrap_or(true)
}

/// The items of the sequence `key` of `node`, where it holds one.
fn seq<'a>(node: &'a Spanned, key: &str) -> Option<&'a [Spanned]> {
    node.get(key).and_then(|s| s.as_seq())
}
