//! The schema keys every format shares, lowered: the frontmatter, the
//! lifecycle, the formats, the examples and the bans of a schema, and the
//! keys every rule map carries.

use std::collections::BTreeMap;

use serde::Serialize;

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

/// One map of a condition: lifecycle dimension to admitted states.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
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

/// A condition on the lifecycle states (grammar `condition`): the maps of
/// a list, of which any one that admits the states is enough. A map alone
/// is a list of one. No map at all is no condition, which admits every
/// state, as a rule with no `only` applies in every state. Build one with
/// [`Condition::new`], which sorts the maps and keeps each once, so that
/// two lists of the same maps make one value.
/// Two conditions are equal where they hold the same maps, whatever the
/// order of the maps and of the states of each.
#[derive(Debug, Clone, Default)]
pub struct Condition(pub Vec<Only>);

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.normal() == other.normal()
    }
}

impl Eq for Condition {}

impl Condition {
    /// The condition of `maps`, in order and each once, whatever the order
    /// of the states of each map. A map that names no dimension admits
    /// every state, so a list that holds one is no condition.
    // sokf-implements: CORE-6_AC-11
    pub fn new(maps: impl IntoIterator<Item = Only>) -> Self {
        let maps: Vec<Only> = maps.into_iter().collect();
        if maps.iter().any(|only| only.0.is_empty()) {
            return Condition::default();
        }
        // Each map once by its normal form, in the order of those forms,
        // each kept as written.
        let mut keyed: Vec<(Vec<Only>, Only)> = maps
            .into_iter()
            .map(|only| (Condition(vec![only.clone()]).normal(), only))
            .collect();
        keyed.sort_by(|a, b| a.0.cmp(&b.0));
        keyed.dedup_by(|a, b| a.0 == b.0);
        Condition(keyed.into_iter().map(|(_, only)| only).collect())
    }

    /// The maps, each with its states in order and each once, in order
    /// and each once: the form that equality compares.
    fn normal(&self) -> Vec<Only> {
        let mut maps: Vec<Only> = self
            .0
            .iter()
            .map(|only| {
                Only(
                    only.0
                        .iter()
                        .map(|(dim, states)| {
                            let mut states = states.clone();
                            states.sort();
                            states.dedup();
                            (dim.clone(), states)
                        })
                        .collect(),
                )
            })
            .collect();
        maps.sort();
        maps.dedup();
        maps
    }

    /// Whether some map of the condition admits `states`, or the condition
    /// names no map. A map admits the states where, for each dimension that
    /// it names, every state of the document that `lifecycle` declares is
    /// one that the map admits. A dimension for which the document holds
    /// no declared state counts as admitted, so a fault in the state of a
    /// document is reported once, on its frontmatter, and not again by each
    /// part that the condition makes optional. A map that names a dimension
    /// that `lifecycle` does not declare admits no document.
    // sokf-implements: CORE-6_AC-12
    pub fn admits_known(&self, states: &States, lifecycle: &[LifecycleDimension]) -> bool {
        self.0.is_empty()
            || self.0.iter().any(|only| {
                only.0.iter().all(|(dim, admitted)| {
                    let Some(declared) = lifecycle.iter().find(|d| &d.key == dim) else {
                        return false;
                    };
                    let mut known = states
                        .0
                        .get(dim)
                        .into_iter()
                        .flatten()
                        .filter(|s| declared.states.contains(s))
                        .peekable();
                    known.peek().is_none() || known.all(|s| admitted.contains(s))
                })
            })
    }

    /// Whether some map of the condition admits `states`, or the
    /// condition names no map.
    // sokf-implements: TL_135
    pub fn admits(&self, states: &States) -> bool {
        self.0.is_empty() || self.0.iter().any(|only| only.admits(states))
    }

    /// Whether this condition and `other` both admit some document's
    /// states: some map of the one overlaps some map of the other.
    pub fn overlaps(&self, other: &Condition) -> bool {
        self.0.is_empty()
            || other.0.is_empty()
            || self.0.iter().any(|a| other.0.iter().any(|b| a.overlaps(b)))
    }

    /// Whether the condition restricts the states at all.
    pub fn is_gated(&self) -> bool {
        !self.0.is_empty()
    }

    /// `k=v,v2 k2=v | k3=v`, as an `only` marker writes it: each map as
    /// [`Only::render`] writes it, joined by ` | `.
    pub fn render(&self) -> String {
        self.0
            .iter()
            .map(Only::render)
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

impl From<Only> for Condition {
    /// The condition of one map, as [`Condition::new`] builds it.
    fn from(only: Only) -> Self {
        Condition::new([only])
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
    pub only: Condition,
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
    /// The source as a finding names it: ``set `x` `` or ``the frontmatter `x` ``.
    pub fn source_label(&self) -> String {
        match &self.source {
            DerivationSource::Rule(r) => format!("set `{r}`"),
            DerivationSource::From(k) => format!("the frontmatter `{k}`"),
        }
    }

    /// The prefix of `prefixes` that `key` derives from: the longest that
    /// `key` starts with, followed by the separator. A prefix can contain
    /// the separator, so the key is never split at its first one.
    // sokf-implements: TL_126
    pub fn prefix_of<'a>(
        &self,
        key: &str,
        prefixes: impl IntoIterator<Item = &'a str>,
    ) -> Option<&'a str> {
        prefixes
            .into_iter()
            .filter(|p| {
                key.strip_prefix(p)
                    .is_some_and(|rest| rest.starts_with(&self.separator))
            })
            .max_by_key(|p| p.len())
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

// sokf:begin limit
/// Whether a limit is checked: grammar `limit` declares `soft` or `hard`.
// sokf-implements: MD-13_AC-3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LimitKind {
    /// The usual length: never checked.
    Soft,
    /// The most a part holds: a part that holds more is an error.
    Hard,
}

/// Grammar `limit`: how many units, lines, or elements a part holds at
/// most, and the schema's text that tells the writer what to do with
/// more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Limit {
    /// `soft` or `hard`.
    pub kind: LimitKind,
    /// The number that `soft` or `hard` states.
    pub count: usize,
    /// `when-at-limit`: what the writer does at the limit, as the schema
    /// writes it.
    pub when_at_limit: String,
}

impl Limit {
    /// The count that the engine checks and bounds by: a hard limit's, and
    /// none for a soft limit.
    // sokf-implements: TL_139
    // sokf-implements: CORE-6_AC-14
    pub fn hard(&self) -> Option<usize> {
        (self.kind == LimitKind::Hard).then_some(self.count)
    }
}
// sokf:end limit

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
    /// `max`: the limit on the number of elements.
    pub max: Option<Limit>,
    /// `optional`.
    pub optional: bool,
    /// `example`: a value of the kind this key demands.
    pub example: Option<String>,
    /// `guidance`: what the example cannot say, written on a comment line before the key.
    pub guidance: Option<String>,
}

impl FrontmatterConstraint {
    /// The value that the schema fixes for the key in a scaffold at
    /// `states`: its `const`, else the `id` template rendered with its
    /// sequence at one and the slug `example`, else its `example`.
    pub fn fixed_value(&self, states: &States) -> Option<String> {
        if let Some(v) = &self.const_ {
            return Some(v.clone());
        }
        if let Some(t) = &self.id_template
            && let Ok(v) = t.render_with(1, t.has_slug().then_some("example"), states)
        {
            return Some(v);
        }
        self.example.clone()
    }

    /// The value that a scaffold at `states` writes for a declared key:
    /// [`FrontmatterConstraint::fixed_value`], else the first value of its
    /// `enum`.
    pub fn scaffold_value(&self, states: &States) -> Option<String> {
        self.fixed_value(states)
            .or_else(|| self.enum_.as_ref()?.first().cloned())
    }
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

impl LifecycleDimension {
    /// The state that a scaffold at `states` writes for the dimension: the
    /// first that `states` gives it, else its first state.
    pub fn state_in<'a>(&'a self, states: &'a States) -> Option<&'a String> {
        states
            .0
            .get(&self.key)
            .and_then(|s| s.first())
            .or_else(|| self.states.first())
    }
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
    /// `language`: the BCP 47 tag of the language of the schema's text.
    pub language: Option<String>,
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
            language: None,
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
                .and_then(Spanned::scalar_text)
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

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: TL_126
    #[test]
    fn a_key_derives_from_the_longest_prefix_that_starts_it() {
        let kd = KeyDerivation {
            source: DerivationSource::From("code".into()),
            separator: "-".into(),
        };
        assert_eq!(kd.prefix_of("CE-BUG-1", ["CE", "CE-BUG"]), Some("CE-BUG"));
        assert_eq!(kd.prefix_of("CE-BUG-1", ["CE-BUG", "CE"]), Some("CE-BUG"));
        assert_eq!(kd.prefix_of("CE-BUG-1", ["CE-BU"]), None);
        assert_eq!(kd.prefix_of("CE-BUG", ["CE-BUG"]), None);
        assert_eq!(kd.prefix_of("CE-BUG-1", ["CE-FIX"]), None);
    }

    /// One map of a condition, from `dimension=state` pairs.
    fn map(pairs: &[(&str, &str)]) -> Only {
        Only(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), vec![v.to_string()]))
                .collect(),
        )
    }

    // A list matches where any of its maps does, two lists overlap where
    // a map of each does, and no map at all is no condition.
    // sokf-tests: TL_135
    #[test]
    fn a_condition_matches_where_any_of_its_maps_does() {
        let either = Condition(vec![
            map(&[("plan", "planned")]),
            map(&[("state", "closed")]),
        ]);
        let at = |pairs: &[(&str, &str)]| States(map(pairs).0);
        assert!(either.admits(&at(&[("plan", "planned"), ("state", "open")])));
        assert!(either.admits(&at(&[("plan", "unplanned"), ("state", "closed")])));
        assert!(!either.admits(&at(&[("plan", "unplanned"), ("state", "open")])));
        assert!(Condition::default().admits(&at(&[("state", "open")])));
        assert!(either.overlaps(&Condition(vec![map(&[("state", "closed")])])));
        assert!(!either.overlaps(&Condition(vec![map(&[
            ("plan", "unplanned"),
            ("state", "open")
        ])])));
        assert!(either.overlaps(&Condition::default()));
        assert_eq!(either.render(), "plan=planned | state=closed");
        assert_eq!(Condition::from(Only::default()), Condition::default());
        let closed = map(&[("state", "closed")]);
        let planned = map(&[("plan", "planned")]);
        assert_eq!(
            Condition::new([closed.clone(), planned.clone(), closed.clone()]),
            Condition::new([planned.clone(), closed.clone()])
        );
        assert_eq!(
            Condition::new([planned.clone(), Only::default()]),
            Condition::default()
        );
        let two = |a: &str, b: &str| {
            Only(BTreeMap::from([(
                "state".to_string(),
                vec![a.to_string(), b.to_string()],
            )]))
        };
        assert_eq!(
            Condition::new([two("open", "closed")]),
            Condition::new([two("closed", "open")])
        );
        assert_eq!(
            Condition::new([two("open", "closed"), two("closed", "open")]).0,
            vec![two("open", "closed")]
        );
    }

    // A dimension for which the document holds no declared state counts as
    // admitted, a dimension that the lifecycle does not declare admits
    // nothing, and a set of states is admitted only where each is.
    // sokf-tests: CORE-6_AC-12
    // sokf-tests: CORE-6_AC-11
    #[test]
    fn a_condition_counts_an_unknown_state_as_admitted() {
        let lifecycle = [LifecycleDimension {
            key: "plan".into(),
            states: vec!["unplanned".into(), "planned".into()],
            guidance: None,
            line: 0,
            extensions: BTreeMap::new(),
        }];
        let unplanned = Condition::new([map(&[("plan", "unplanned")])]);
        let at = |pairs: &[(&str, &str)]| States(map(pairs).0);
        assert!(unplanned.admits_known(&at(&[("plan", "unplanned")]), &lifecycle));
        assert!(!unplanned.admits_known(&at(&[("plan", "planned")]), &lifecycle));
        assert!(unplanned.admits_known(&at(&[]), &lifecycle));
        assert!(unplanned.admits_known(&at(&[("plan", "later")]), &lifecycle));
        let both = States(BTreeMap::from([(
            "plan".to_string(),
            vec!["unplanned".to_string(), "planned".to_string()],
        )]));
        assert!(!unplanned.admits_known(&both, &lifecycle));
        let unknown = Condition::new([map(&[("phase", "closed")])]);
        assert!(!unknown.admits_known(&at(&[("plan", "planned")]), &lifecycle));
        assert!(!Condition::from(Only::default()).is_gated());
    }
}
