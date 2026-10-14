//! The schema keys every format shares, lowered: the frontmatter, the
//! lifecycle, the formats, the examples and the bans of a schema, and the
//! keys every rule map carries.

use std::collections::BTreeMap;

use super::{FormatRegistry, IdTemplate, Shape};
use crate::{Severity, Spanned};

/// The scope of a captured key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// Unique within the document.
    Document,
    /// Unique within a scope the host names and checks, as `knowledge`.
    Host(String),
}

impl Scope {
    /// The scope a `unique` value names; absent means the document.
    pub fn parse(s: Option<&str>) -> Scope {
        match s {
            None | Some("document") => Scope::Document,
            Some(host) => Scope::Host(host.to_string()),
        }
    }
}

/// Lifecycle dimension to admitted states (grammar `only`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Only(pub BTreeMap<String, Vec<String>>);

impl Only {
    /// Whether the rule applies to a document in `states`: for every
    /// dimension named, the document's states intersect the admitted ones.
    pub fn admits(&self, states: &States) -> bool {
        self.0.iter().all(|(dim, admitted)| {
            states
                .0
                .get(dim)
                .is_some_and(|have| have.iter().any(|s| admitted.contains(s)))
        })
    }

    /// Whether some document's states admit both this rule and `other`:
    /// each dimension both name admits a state in common.
    pub fn overlaps(&self, other: &Only) -> bool {
        self.0.iter().all(|(dim, admitted)| {
            other
                .0
                .get(dim)
                .is_none_or(|theirs| theirs.iter().any(|s| admitted.contains(s)))
        })
    }

    /// Whether the rule is gated at all.
    pub fn is_gated(&self) -> bool {
        !self.0.is_empty()
    }

    /// `k=v,v2 k2=v`, as an `only` marker writes it.
    pub fn render(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{k}={}", v.join(",")))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A document's lifecycle states: one per dimension, or a set for an
/// example.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct States(pub BTreeMap<String, Vec<String>>);

/// Grammar `rule-common` plus `only`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleCommon {
    /// Error or warning.
    pub severity: Severity,
    /// When the rule applies.
    pub only: Only,
    /// The rule's line in the schema.
    pub line: usize,
}

/// A grammar `text` value: a literal, or a frontmatter key whose value
/// stands in for it. Frontmatter is the authority; a repair rewrites the
/// body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Text {
    /// The string as written.
    Literal(String),
    /// `{ from: <key> }`.
    From(String),
}

/// Grammar `key-derivation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDerivation {
    /// Where the allowed prefixes come from.
    pub source: DerivationSource,
    /// The literal ending the prefix.
    pub separator: String,
}

/// Where a `key-derivation` takes its prefixes (grammar exactly-one-of).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivationSource {
    /// `rule`: the keys that rule captured.
    Rule(String),
    /// `from`: the value of a frontmatter key, the one prefix.
    From(String),
}

impl KeyDerivation {
    /// The source as a finding names it: `rule \`x\`` or `the frontmatter \`x\``.
    pub fn source_label(&self) -> String {
        match &self.source {
            DerivationSource::Rule(r) => format!("rule `{r}`"),
            DerivationSource::From(k) => format!("the frontmatter `{k}`"),
        }
    }
}

/// `format` of a frontmatter constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmFormat {
    /// An absolute URL.
    Url,
    /// A slug: lower-case letters and digits joined by single hyphens.
    Slug,
    /// A format the host names and checks, as `document-id`.
    Host(String),
}

impl FmFormat {
    /// The format's name in the grammar.
    pub fn name(&self) -> &str {
        match self {
            FmFormat::Url => "url",
            FmFormat::Slug => "slug",
            FmFormat::Host(name) => name,
        }
    }
}

/// Grammar `frontmatter-constraint`.
#[derive(Debug, Clone)]
pub struct FrontmatterConstraint {
    /// The common keys.
    pub common: RuleCommon,
    /// `const`.
    pub const_: Option<String>,
    /// `template` or `pattern`: the value's shape.
    pub shape: Option<Shape>,
    /// `enum`.
    pub enum_: Option<Vec<String>>,
    /// `format`.
    pub format: Option<FmFormat>,
    /// The id a new document is filled with, from the `id` key's template.
    pub id_template: Option<IdTemplate>,
    /// `list`.
    pub list: bool,
    /// `min`.
    pub min: Option<usize>,
    /// `max`.
    pub max: Option<usize>,
    /// `optional`.
    pub optional: bool,
    /// `example`: a value of the kind this key demands.
    pub example: Option<String>,
    /// `guidance`: what the example cannot say, written on a comment line before the key.
    pub guidance: Option<String>,
}

/// Grammar `lifecycle-dimension`.
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleDimension {
    /// `key`.
    pub key: String,
    /// `states`.
    pub states: Vec<String>,
    /// `guidance`: what the states mean, written on a comment line before the key.
    pub guidance: Option<String>,
    /// The line in the schema.
    pub line: usize,
    /// The keys a host's extension adds, as written: the host lowers what
    /// it adds.
    pub extensions: BTreeMap<String, Spanned>,
}

/// Grammar `example`.
#[derive(Debug, Clone)]
pub struct Example {
    /// `document`.
    pub document: String,
    /// The line in the schema.
    pub line: usize,
}

// sokf:begin core-schema
/// The part of a lowered schema that every format shares: grammar
/// `schema`, the keys of the core grammar.
#[derive(Debug, Clone)]
pub struct CoreSchema {
    /// `line-limit`; zero or less declares no limit.
    pub line_limit: i64,
    /// `lifecycle`.
    pub lifecycle: Vec<LifecycleDimension>,
    /// `frontmatter`, in schema order.
    pub frontmatter: Vec<(String, FrontmatterConstraint)>,
    /// `frontmatter-open`.
    pub frontmatter_open: bool,
    /// `prohibited-patterns`: what no line of the body may match.
    pub prohibited_patterns: Vec<super::Prohibition>,
    /// The registry as this schema sees it.
    pub formats: FormatRegistry,
    /// `guidance`: what the schema says that no example can.
    pub guidance: Option<String>,
    /// `examples`.
    pub examples: Vec<Example>,
    /// The line of the schema where it is written.
    pub line: usize,
    /// The line of each key of the schema, by the key.
    pub key_lines: BTreeMap<String, usize>,
}
// sokf:end core-schema

impl CoreSchema {
    /// A schema that demands nothing.
    pub fn demand_nothing() -> CoreSchema {
        CoreSchema {
            line_limit: 0,
            lifecycle: vec![],
            frontmatter: vec![],
            frontmatter_open: true,
            prohibited_patterns: vec![],
            formats: FormatRegistry::default(),
            guidance: None,
            examples: vec![],
            line: 0,
            key_lines: BTreeMap::new(),
        }
    }

    /// A document's lifecycle states, one per dimension its frontmatter
    /// carries.
    pub fn states(&self, frontmatter: Option<&Spanned>) -> States {
        let mut states = States::default();
        for dim in &self.lifecycle {
            if let Some(value) = frontmatter
                .and_then(|f| f.get(&dim.key))
                .and_then(|v| v.scalar_text())
            {
                states.0.insert(dim.key.clone(), vec![value]);
            }
        }
        states
    }

    /// The `type` const, where the `type` key declares one.
    pub fn type_const(&self) -> Option<&str> {
        self.constraint("type")?.const_.as_deref()
    }

    /// The constraint the schema declares on the frontmatter `key`.
    // sokf:begin core-schema
    pub fn constraint(&self, key: &str) -> Option<&FrontmatterConstraint> {
        // sokf:end core-schema
        // sokf-implements: CORE-6_AC-9
        self.frontmatter
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, c)| c)
    }

    /// The id template, if `frontmatter.id.template` is declared.
    pub fn id_template(&self) -> Option<&IdTemplate> {
        self.constraint("id")?.id_template.as_ref()
    }

    /// The lifecycle dimension with `key`.
    pub fn dimension(&self, key: &str) -> Option<&LifecycleDimension> {
        self.lifecycle.iter().find(|d| d.key == key)
    }
}
