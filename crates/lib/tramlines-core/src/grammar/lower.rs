//! The lowering of the keys every format shares: a typed [`CoreSchema`]
//! from a schema YAML that passed the generic check. A format crate wraps
//! [`Lowerer`] and lowers its own keys beside these.

use std::collections::{BTreeMap, HashSet};

use regex::Regex;

use super::{
    CoreSchema, DerivationSource, Example, FormatRegistry, FormatSource, Grammar, KeyDerivation,
    LifecycleDimension, Only, RegexCache, RuleCommon, TypeRegistry,
};
use crate::{Finding, Problem, RepoPath, Severity, Spanned, Yaml};

// sokf-implements: CORE-6_AC-1
// sokf:begin lowering
/// The keys of `schema` the core grammar lowers.
pub const CORE_SCHEMA_KEYS: &[&str] = &[
    "line-limit",
    "guidance",
    "lifecycle",
    "frontmatter",
    "frontmatter-open",
    "prohibited-patterns",
    "formats",
    "types",
    "examples",
];
// sokf:end lowering

/// The keys of `lifecycle-dimension` the engine lowers.
const DIMENSION_KEYS: &[&str] = &["key", "states", "guidance"];

/// The entries of `node` whose keys `known` does not name, as written.
// sokf-implements: CORE-3_AC-3
pub fn extensions(node: &Spanned, known: &[&str]) -> BTreeMap<String, Spanned> {
    node.as_map()
        .unwrap_or(&[])
        .iter()
        .filter(|(k, _)| !known.contains(&k.value.as_str()))
        .map(|(k, v)| (k.value.clone(), v.clone()))
        .collect()
}

/// The registry a schema sees: its own `formats` and `types` as the
/// nearest layer on `registry`. A refusal is added to `findings`, and the
/// schema then sees `registry` alone.
// sokf-implements: TL_34
// sokf-implements: TL_36
// sokf-implements: CORE-6_AC-5
// sokf:begin lowering
pub fn lower_formats(
    yaml: &Spanned,
    grammar: &Grammar,
    registry: &FormatRegistry,
    regexes: &RegexCache,
    schema: &RepoPath,
    schema_id: &str,
    findings: &mut Vec<Finding>,
) -> FormatRegistry {
    // sokf:end lowering
    if yaml.get("formats").is_none() && yaml.get("types").is_none() {
        return registry.clone();
    }
    let none = Spanned::new(Yaml::Map(vec![]), yaml.line);
    // What loads is kept, so a type or a format that fails is reported
    // once, and not again by each rule that uses it.
    let (own, refused) = FormatRegistry::layer(
        yaml.get("formats").unwrap_or(&none),
        yaml.get("types"),
        FormatSource::Schema(schema_id.to_string()),
        grammar,
        regexes,
        schema,
        Some(registry),
    );
    // sokf-implements: CORE-6_AC-6
    findings.extend(refused);
    registry.layered(&own)
}

/// The keys every rule map accepts, `severity` and `only`, lowered for a
/// rule map a host's extension adds: `only` resolves against the schema's
/// lifecycle.
// sokf-implements: CORE-6_AC-7
// sokf:begin lowering
pub fn lower_common(
    node: &Spanned,
    schema: &CoreSchema,
    path: &RepoPath,
) -> (RuleCommon, Vec<Finding>) {
    // sokf:end lowering
    let regexes = RegexCache::new();
    let mut l = Lowerer::new(&regexes, path, TypeRegistry::default());
    l.lifecycle = schema.lifecycle.clone();
    let common = l.common(node);
    (common, l.findings)
}

// sokf:begin lowerer
/// The state of one lowering: the schema it reports on, what it found,
/// and what it has read so far that later keys resolve against.
pub struct Lowerer<'a> {
    /// The cache the schema's regexes compile through.
    pub regexes: &'a RegexCache,
    /// The schema that every finding is reported on.
    pub schema: &'a RepoPath,
    /// The findings so far.
    pub findings: Vec<Finding>,
    /// The `rule-id`s declared so far.
    pub rule_ids: HashSet<String>,
    /// The lifecycle, which `only` resolves against.
    pub lifecycle: Vec<LifecycleDimension>,
    /// The types a template of this schema is written with.
    pub types: TypeRegistry,
}
// sokf:end lowerer

/// The string at `key` of `node`.
pub fn str_of(node: &Spanned, key: &str) -> Option<String> {
    node.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

/// A grammar `key-derivation`: a rule's keys or a frontmatter value.
pub fn derivation(node: &Spanned) -> KeyDerivation {
    let source = match str_of(node, "from") {
        Some(key) => DerivationSource::From(key),
        None => DerivationSource::Rule(str_of(node, "rule").unwrap_or_default()),
    };
    KeyDerivation {
        source,
        separator: str_of(node, "separator").unwrap_or_default(),
    }
}

/// The boolean at `key` of `node`; absent means false.
pub fn bool_of(node: &Spanned, key: &str) -> bool {
    node.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// The count at `key` of `node`; a negative count reads as zero.
pub fn usize_of(node: &Spanned, key: &str) -> Option<usize> {
    node.get(key)
        .and_then(|v| v.as_int())
        .map(|i| usize::try_from(i).unwrap_or(0))
}

/// The string or the list of strings at `key` of `node`.
pub fn list_of(node: &Spanned, key: &str) -> Vec<String> {
    node.get(key)
        .and_then(|v| v.as_str_or_list())
        .unwrap_or_default()
}

impl<'a> Lowerer<'a> {
    /// A lowering of a schema at `schema`, whose templates see `types`.
    // sokf:begin lowering
    pub fn new(regexes: &'a RegexCache, schema: &'a RepoPath, types: TypeRegistry) -> Self {
        // sokf:end lowering
        Lowerer {
            regexes,
            schema,
            findings: Vec::new(),
            rule_ids: HashSet::new(),
            lifecycle: Vec::new(),
            types,
        }
    }

    /// A finding on the schema, at `line`, that breaks `criterion`.
    pub fn fail(&mut self, line: usize, criterion: &'static str, message: impl Into<String>) {
        self.findings.push(Finding::error(
            self.schema.clone(),
            Some(line),
            criterion,
            message,
        ));
    }

    /// A finding on the schema, at `line`, for `problem`.
    pub fn fail_with(&mut self, line: usize, problem: Problem) {
        self.findings
            .push(problem.error(self.schema.clone(), Some(line)));
    }

    /// The regex at `key` of `node`, checked where the grammar was read.
    pub fn regex(&self, node: &Spanned, key: &str) -> Option<Regex> {
        let text = node.get(key)?.as_str()?;
        self.regexes.get(text).ok()
    }

    /// The `prohibited-patterns` of `node`.
    pub fn prohibitions(&self, node: &Spanned) -> Vec<super::Prohibition> {
        super::prohibitions(node, self.regexes)
    }

    /// The `only` of a rule, resolved against the lifecycle.
    // sokf-implements: TL_19
    // sokf-implements: TL_20
    pub fn only(&mut self, node: &Spanned) -> Only {
        let mut only = Only::default();
        let Some(map) = node.get("only") else {
            return only;
        };
        if self.lifecycle.is_empty() {
            // sokf-implements: TL_111
            self.fail(map.line, "TL_111", "`only` without a lifecycle");
            return only;
        }
        for (dim, states) in map.as_map().unwrap_or(&[]) {
            let Some(dimension) = self
                .lifecycle
                .iter()
                .find(|d| d.key == dim.value)
                .map(|d| d.states.clone())
            else {
                self.fail(
                    dim.line,
                    "TL_20",
                    format!(
                        "`only` names a dimension the lifecycle does not declare: `{}`",
                        dim.value
                    ),
                );
                continue;
            };
            let admitted = states.as_str_or_list().unwrap_or_default();
            for s in &admitted {
                if !dimension.contains(s) {
                    self.fail(
                        states.line,
                        "TL_20",
                        format!("`only` names `{s}`, which is no state of `{}`", dim.value),
                    );
                }
            }
            only.0.insert(dim.value.clone(), admitted);
        }
        only
    }

    /// The keys every rule map carries: `severity`, and `only`.
    // sokf-implements: TL_18
    pub fn common(&mut self, node: &Spanned) -> RuleCommon {
        RuleCommon {
            severity: match str_of(node, "severity").as_deref() {
                Some("warn") => Severity::Warning,
                _ => Severity::Error,
            },
            only: self.only(node),
            line: node.line,
        }
    }

    /// The keys of the core grammar of a schema: the lifecycle first, which
    /// `only` resolves against, then the frontmatter, the bans and the
    /// examples. `formats` is the registry the schema sees.
    // sokf-implements: CORE-6_AC-2
    // sokf:begin lowering
    pub fn lower_core(&mut self, yaml: &Spanned, formats: FormatRegistry) -> CoreSchema {
        // sokf:end lowering
        self.lifecycle = yaml
            .get("lifecycle")
            .and_then(|l| l.as_seq())
            .map(|dims| {
                dims.iter()
                    .map(|d| LifecycleDimension {
                        key: str_of(d, "key").unwrap_or_default(),
                        states: list_of(d, "states"),
                        guidance: str_of(d, "guidance"),
                        line: d.line,
                        // sokf-implements: CORE-6_AC-8
                        extensions: extensions(d, DIMENSION_KEYS),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut seen_keys = HashSet::new();
        let twice: Vec<(usize, String)> = self
            .lifecycle
            .iter()
            .filter(|d| !seen_keys.insert(d.key.as_str()))
            .map(|d| (d.line, d.key.clone()))
            .collect();
        for (line, key) in twice {
            // sokf-implements: CORE-6_AC-3
            // sokf-implements: TL_95
            self.fail(
                line,
                "TL_95",
                format!("dimension `{key}` is declared twice"),
            );
        }

        let mut frontmatter = Vec::new();
        for (key, node) in yaml
            .get("frontmatter")
            .and_then(|f| f.as_map())
            .unwrap_or(&[])
        {
            if self.lifecycle.iter().any(|d| d.key == key.value) {
                // sokf-implements: CORE-6_AC-4
                // sokf-implements: TL_112
                self.fail(
                    key.line,
                    "TL_112",
                    format!(
                        "`{}` is a lifecycle dimension; it is not repeated under `frontmatter`",
                        key.value
                    ),
                );
            }
            let c = self.frontmatter_constraint(node, &key.value);
            frontmatter.push((key.value.clone(), c));
        }
        // A ban is a bare pattern; its failure quotes what it matched.
        // sokf-implements: TL_17
        let prohibited_patterns = self.prohibitions(yaml);
        let examples = yaml
            .get("examples")
            .and_then(|e| e.as_seq())
            .map(|items| {
                items
                    .iter()
                    .map(|e| Example {
                        document: str_of(e, "document").unwrap_or_default(),
                        line: e.line,
                    })
                    .collect()
            })
            .unwrap_or_default();
        CoreSchema {
            line_limit: yaml.get("line-limit").and_then(|l| l.as_int()).unwrap_or(0),
            lifecycle: self.lifecycle.clone(),
            frontmatter,
            frontmatter_open: bool_of(yaml, "frontmatter-open"),
            prohibited_patterns,
            formats,
            guidance: str_of(yaml, "guidance"),
            examples,
            line: yaml.line,
            key_lines: yaml
                .as_map()
                .unwrap_or(&[])
                .iter()
                .map(|(k, _)| (k.value.clone(), k.line))
                .collect(),
        }
    }
}

mod checks;
mod shapes;
