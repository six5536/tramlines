//! The parsed grammar: its `meta` vocabulary and the `schema` rule maps,
//! with what a format grammar and a host's extension add.

use std::collections::BTreeMap;

use super::{bool_of, embedded, str_of};
use crate::{Error, Result, Spanned, Yaml, yaml};

/// The `meta.types` of the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeName {
    /// A string.
    String,
    /// An integer.
    Integer,
    /// A boolean.
    Boolean,
    /// A list of `of`.
    List,
    /// A keyed map of `of`.
    Map,
    /// One instance of `of`.
    Object,
    /// A string or a list of strings.
    StringOrList,
    /// A string or a map.
    StringOrMap,
    /// A string, or `{ from: <frontmatter key> }` standing for the key's
    /// value.
    Text,
    /// A `text`, or one instance of the rule map `of`.
    TextOrRule,
}

impl TypeName {
    /// The type the grammar names `s`.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "string" => TypeName::String,
            "integer" => TypeName::Integer,
            "boolean" => TypeName::Boolean,
            "list" => TypeName::List,
            "map" => TypeName::Map,
            "object" => TypeName::Object,
            "string-or-list" => TypeName::StringOrList,
            "string-or-map" => TypeName::StringOrMap,
            "text" => TypeName::Text,
            "text-or-rule" => TypeName::TextOrRule,
            _ => return None,
        })
    }
}

/// One key of a rule map, as `meta.keys` describes it.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDef {
    /// The value's type.
    pub type_: TypeName,
    /// The item, value or instance shape: a type, a scalar format or a rule
    /// map name.
    pub of: Option<String>,
    /// Whether a schema must carry the key.
    pub required: bool,
    /// The values the key admits, as text.
    pub enum_: Vec<String>,
    /// A scalar format or a reference: `regex`, `slug`, `rule-id`,
    /// `format-name`.
    pub format: Option<String>,
    /// Keys that must be present beside this one.
    pub requires_keys: Vec<String>,
    /// Block kinds the enclosing block must be one of.
    pub requires_kind: Vec<String>,
    /// What the key means.
    pub doc: String,
}

/// One rule map under `schema`, or `schema` itself.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuleMap {
    /// What the map is.
    pub doc: String,
    /// The keys, in grammar order.
    pub keys: Vec<(String, KeyDef)>,
    /// Exactly one of these keys must be present.
    pub exactly_one_of: Vec<String>,
    /// At least one of these keys must be present.
    pub at_least_one_of: Vec<String>,
    /// Whether the map accepts the `rule-common` keys.
    pub common: bool,
}

impl RuleMap {
    /// The definition of `key`.
    pub fn key(&self, key: &str) -> Option<&KeyDef> {
        self.keys.iter().find(|(k, _)| k == key).map(|(_, d)| d)
    }
}

// sokf:begin grammar
/// The parsed grammar file.
#[derive(Debug, Clone, PartialEq)]
pub struct Grammar {
    /// The name of the grammar: `tramlines`, then the name of the format
    /// grammar laid on it.
    pub name: String,
    /// The grammar version.
    pub version: String,
    /// The `schema` vocabulary.
    pub schema: RuleMap,
    /// The rule maps under `schema`, by name.
    pub rules: BTreeMap<String, RuleMap>,
    /// The `rule-common` keys.
    pub rule_common: RuleMap,
    /// The sections an extension adds beside `schema`, by name, as written.
    pub sections: BTreeMap<String, Spanned>,
}
// sokf:end grammar

impl Grammar {
    /// The rule map named `name`.
    pub fn rule(&self, name: &str) -> Option<&RuleMap> {
        if name == "schema" {
            return Some(&self.schema);
        }
        self.rules.get(name)
    }

    /// Whether `name` is a rule map that accepts the `rule-common` keys.
    pub fn accepts_common(&self, name: &str) -> bool {
        self.rule(name).is_some_and(|m| m.common)
    }

    /// A section an extension adds, by name.
    // sokf:begin grammar-extend
    pub fn section(&self, name: &str) -> Option<&Spanned> {
        // sokf:end grammar-extend
        self.sections.get(name)
    }

    /// Lay a format grammar on this one, the core grammar: it names
    /// itself with `grammar` and its base with `base`, and adds keys, rule
    /// maps, values of an enum, block kinds a key requires, text of a
    /// `doc`, and the constraint groups of any rule map. A format grammar
    /// that changes what the core grammar says is a defect of the build.
    // sokf-implements: TL_2
    // sokf-implements: TL_3
    // sokf-implements: TL_4
    // sokf-implements: CORE-2_AC-1
    // sokf-implements: CORE-2_AC-2
    // sokf:begin grammar-base
    pub fn base(&mut self, text: &str) -> Result<()> {
        // sokf:end grammar-base
        self.transact(|g| g.lay_base(text))
    }

    /// Lay a layer with `lay` on a copy of the grammar. A layer that fails
    /// leaves the grammar as it was, so no half-laid layer reaches the
    /// validator.
    fn transact(&mut self, lay: impl FnOnce(&mut Grammar) -> Result<()>) -> Result<()> {
        let mut next = self.clone();
        lay(&mut next)?;
        *self = next;
        Ok(())
    }

    fn lay_base(&mut self, text: &str) -> Result<()> {
        let fail = |m: String| Error::Internal(format!("format grammar: {m}"));
        let root = yaml::parse(text, 1).map_err(|e| fail(e.to_string()))?;
        let Some(name) = root.get("grammar").and_then(|e| e.as_str()) else {
            return Err(fail("no `grammar` name".into()));
        };
        let name = name.to_string();
        // sokf-implements: CORE-2_AC-3
        match root.get("base").and_then(|e| e.as_str()) {
            Some(base) if base == self.name => {}
            other => {
                return Err(fail(format!("`base` is {other:?}, not `{}`", self.name)));
            }
        }
        for (key, value) in root.as_map().unwrap_or(&[]) {
            match key.value.as_str() {
                "grammar" | "base" | "doc" => {}
                // sokf-implements: CORE-2_AC-6
                "version" => {
                    if let Some(v) = value.scalar_text() {
                        self.version = v;
                    }
                }
                "meta" => return Err(fail("`meta` is the core grammar's".into())),
                "schema" => self.extend_schema(value, true).map_err(fail)?,
                // sokf-implements: CORE-2_AC-4
                other => return Err(fail(format!("unknown section `{other}`"))),
            }
        }
        self.check_references().map_err(fail)?;
        // sokf-implements: CORE-2_AC-5
        self.name = name;
        Ok(())
    }

    /// Add a host's extension, which names itself with `extension` and
    /// names the format grammar it extends with `extends`: keys, rule maps
    /// and sections of its own; on a key of the grammar, more `enum`
    /// values, more block kinds it `requires`, and more `doc`. An extension
    /// that changes what the grammar says is a defect of the build.
    // sokf-implements: TL_5
    // sokf-implements: TL_6
    // sokf-implements: TL_7
    // sokf-implements: TL_8
    // sokf-implements: CORE-3_AC-1
    // sokf-implements: CORE-3_AC-2
    // sokf-implements: CORE-3_AC-5
    // sokf:begin grammar-extend
    pub fn extend(&mut self, text: &str) -> Result<()> {
        // sokf:end grammar-extend
        self.transact(|g| g.lay_extension(text))
    }

    fn lay_extension(&mut self, text: &str) -> Result<()> {
        let fail = |m: String| Error::Internal(format!("schema extension: {m}"));
        let root = yaml::parse(text, 1).map_err(|e| fail(e.to_string()))?;
        if root.get("extension").and_then(|e| e.as_str()).is_none() {
            return Err(fail("no `extension` name".into()));
        }
        match root.get("extends").and_then(|e| e.as_str()) {
            Some(name) if name == self.name => {}
            other => {
                return Err(fail(format!("extends {other:?}, not `{}`", self.name)));
            }
        }
        for (name, value) in root.as_map().unwrap_or(&[]) {
            match name.value.as_str() {
                "extension" | "extends" | "version" | "doc" => {}
                "meta" => return Err(fail("`meta` is the core grammar's".into())),
                "schema" => self.extend_schema(value, false).map_err(fail)?,
                // sokf-implements: CORE-3_AC-4
                other if self.sections.contains_key(other) => {
                    return Err(fail(format!("section `{other}` twice")));
                }
                other => {
                    self.sections.insert(other.to_string(), value.clone());
                }
            }
        }
        self.check_references().map_err(fail)
    }

    /// Refuse an `of` that names neither a scalar nor a rule map, so that
    /// the validator, which follows each one, never meets one that names
    /// nothing: the fault is the grammar's, and no schema can mend it.
    fn check_references(&self) -> std::result::Result<(), String> {
        let maps = [("schema", &self.schema), ("rule-common", &self.rule_common)]
            .into_iter()
            .chain(self.rules.iter().map(|(n, m)| (n.as_str(), m)));
        for (name, map) in maps {
            for (key, def) in &map.keys {
                match def.of.as_deref() {
                    None | Some("string" | "integer" | "boolean" | "string-or-list") => {}
                    Some(of) if self.rule(of).is_some() => {}
                    Some(of) => {
                        return Err(format!(
                            "`{name}.{key}` holds `{of}`, which the grammar does not define"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Add `node`, a `schema` section, to this grammar's. A format grammar
    /// may also add text to a `doc` and constraint groups to a rule map; a
    /// host extension may not.
    fn extend_schema(&mut self, node: &Spanned, format: bool) -> std::result::Result<(), String> {
        for (name, value) in node.as_map().unwrap_or(&[]) {
            match name.value.as_str() {
                "keys" => extend_keys(&mut self.schema, value, "schema")?,
                "doc" | "exactly-one-of" | "at-least-one-of" if format => {
                    add_to_map(&mut self.schema, name.value.as_str(), value);
                }
                "doc" | "exactly-one-of" | "at-least-one-of" | "rule-common" => {
                    return Err(format!("`schema.{}` is the core grammar's", name.value));
                }
                map => match self.rules.get_mut(map) {
                    Some(core) => {
                        for (k, v) in value.as_map().unwrap_or(&[]) {
                            match k.value.as_str() {
                                "keys" => extend_keys(core, v, map)?,
                                "doc" | "exactly-one-of" | "at-least-one-of" if format => {
                                    add_to_map(core, k.value.as_str(), v);
                                }
                                other => {
                                    return Err(format!("`{map}.{other}` is the core grammar's"));
                                }
                            }
                        }
                    }
                    None => {
                        let added = rule_map(value).map_err(|e| e.to_string())?;
                        self.rules.insert(map.to_string(), added);
                    }
                },
            }
        }
        Ok(())
    }

    /// The definition of `key` in rule map `name`, common keys included.
    pub fn key_def(&self, name: &str, key: &str) -> Option<&KeyDef> {
        self.rule(name).and_then(|m| m.key(key)).or_else(|| {
            if self.accepts_common(name) {
                self.rule_common.key(key)
            } else {
                None
            }
        })
    }
}

/// Append the text of a layer's `doc` to `doc`.
fn append_doc(doc: &mut String, more: &Spanned) {
    let more = more.as_str().unwrap_or("").trim();
    *doc = format!("{doc} {more}").trim().to_string();
}

/// Add a format grammar's `doc` text or constraint group to `map`.
fn add_to_map(map: &mut RuleMap, key: &str, value: &Spanned) {
    match key {
        "doc" => append_doc(&mut map.doc, value),
        "exactly-one-of" => map.exactly_one_of.extend(string_list(Some(value))),
        _ => map.at_least_one_of.extend(string_list(Some(value))),
    }
}

/// Add an extension's `keys` to a core rule map: a new key whole; on a
/// core key, only more `enum` values, more required kinds and more `doc`.
fn extend_keys(map: &mut RuleMap, keys: &Spanned, name: &str) -> std::result::Result<(), String> {
    for (key, def) in keys.as_map().unwrap_or(&[]) {
        let Some(core) = map.keys.iter_mut().find(|(k, _)| *k == key.value) else {
            map.keys
                .push((key.value.clone(), key_def(def).map_err(|e| e.to_string())?));
            continue;
        };
        let core = &mut core.1;
        for (k, v) in def.as_map().unwrap_or(&[]) {
            match k.value.as_str() {
                "enum" => core.enum_.extend(
                    v.as_seq()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|i| i.scalar_text()),
                ),
                "requires" if matches!(v.value, Yaml::Map(_)) => core.requires_kind.extend(
                    v.get("kind")
                        .and_then(|k| k.as_str_or_list())
                        .unwrap_or_default(),
                ),
                "doc" => append_doc(&mut core.doc, v),
                other => {
                    return Err(format!(
                        "`{name}.{}` changes the core grammar's `{other}`",
                        key.value
                    ));
                }
            }
        }
    }
    Ok(())
}

// sokf-implements: CORE-1_AC-1
// sokf:begin grammar-load
/// Parse the embedded core grammar, onto which a format grammar is merged.
pub fn load_core_grammar() -> Result<Grammar> {
    parse_grammar(embedded::grammar_text())
}
// sokf:end grammar-load

/// Parse a core grammar file.
// sokf-implements: CORE-1_AC-2
// sokf-implements: CORE-1_AC-3
// sokf:begin grammar-load
pub fn parse_grammar(text: &str) -> Result<Grammar> {
    // sokf:end grammar-load
    let root = yaml::parse(text, 1).map_err(|e| Error::Internal(format!("grammar: {e}")))?;
    let name = root
        .get("grammar")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Internal("grammar: no `grammar` name".into()))?
        .to_string();
    let version = root
        .get("version")
        .and_then(|v| v.scalar_text())
        .ok_or_else(|| Error::Internal("grammar: no version".into()))?;
    let schema_node = root
        .get("schema")
        .ok_or_else(|| Error::Internal("grammar: no schema section".into()))?;
    let schema = rule_map(schema_node)?;
    let mut rules = BTreeMap::new();
    let mut rule_common = RuleMap::default();
    for (key, value) in schema_node.as_map().unwrap_or(&[]) {
        match key.value.as_str() {
            "doc" | "keys" | "exactly-one-of" | "at-least-one-of" | "common" => {}
            "rule-common" => rule_common = rule_map(value)?,
            name => {
                rules.insert(name.to_string(), rule_map(value)?);
            }
        }
    }
    let grammar = Grammar {
        name,
        version,
        schema,
        rules,
        rule_common,
        sections: BTreeMap::new(),
    };
    grammar
        .check_references()
        .map_err(|e| Error::Internal(format!("grammar: {e}")))?;
    Ok(grammar)
}

fn rule_map(node: &Spanned) -> Result<RuleMap> {
    let mut map = RuleMap {
        doc: doc_of(node),
        ..RuleMap::default()
    };
    if let Some(keys) = node.get("keys") {
        for (name, def) in keys.as_map().unwrap_or(&[]) {
            map.keys.push((name.value.clone(), key_def(def)?));
        }
    }
    map.exactly_one_of = string_list(node.get("exactly-one-of"));
    map.at_least_one_of = string_list(node.get("at-least-one-of"));
    map.common = bool_of(node, "common");
    Ok(map)
}

fn key_def(node: &Spanned) -> Result<KeyDef> {
    let type_ = match node.get("type").and_then(|t| t.as_str()) {
        Some(t) => TypeName::parse(t)
            .ok_or_else(|| Error::Internal(format!("grammar: unknown type `{t}`")))?,
        None => TypeName::String,
    };
    let mut requires_keys = Vec::new();
    let mut requires_kind = Vec::new();
    if let Some(r) = node.get("requires") {
        match &r.value {
            Yaml::Seq(_) => requires_keys = string_list(Some(r)),
            Yaml::Map(_) => {
                requires_kind = r
                    .get("kind")
                    .and_then(|k| k.as_str_or_list())
                    .unwrap_or_default()
            }
            _ => {}
        }
    }
    Ok(KeyDef {
        type_,
        of: str_of(node, "of"),
        required: bool_of(node, "required"),
        enum_: node
            .get("enum")
            .and_then(|e| e.as_seq())
            .map(|items| items.iter().filter_map(|i| i.scalar_text()).collect())
            .unwrap_or_default(),
        format: str_of(node, "format"),
        requires_keys,
        requires_kind,
        doc: doc_of(node),
    })
}

/// The `doc` of a grammar node, trimmed, or empty.
fn doc_of(node: &Spanned) -> String {
    str_of(node, "doc").unwrap_or_default().trim().to_string()
}

fn string_list(node: Option<&Spanned>) -> Vec<String> {
    node.and_then(|n| n.as_str_or_list()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMAT: &str = "grammar: fmt\nbase: tramlines\nversion: '0.2'\ndoc: A format.\nschema:\n  doc: More.\n  at-least-one-of: [body]\n  keys:\n    body:\n      type: list\n      of: part\n  part:\n    common: true\n    keys:\n      text:\n        type: string\n  frontmatter-constraint:\n    exactly-one-of: [const, enum]\n    keys:\n      format:\n        enum: [email]\n";

    // sokf-tests: TL_1
    #[test]
    fn loads_the_core_grammar() {
        let g = load_core_grammar().unwrap();
        assert_eq!(g.name, "tramlines");
        assert_eq!(g.version, "0.1");
        assert!(g.schema.key("line-limit").unwrap().required);
        assert!(g.schema.key("frontmatter").is_some());
        assert!(g.schema.key("sections").is_none());
        assert!(g.schema.at_least_one_of.is_empty());
        assert!(g.rule("section").is_none());
        assert!(g.rule("frontmatter-constraint").is_some());
        assert!(g.rule_common.key("severity").is_some());
    }

    // sokf-tests: TL_2
    // sokf-tests: TL_3
    #[test]
    fn a_format_grammar_adds_keys_rule_maps_and_constraint_groups() {
        let mut g = load_core_grammar().unwrap();
        g.base(FORMAT).unwrap();
        assert_eq!(g.name, "fmt");
        assert_eq!(g.version, "0.2");
        assert!(g.schema.doc.ends_with("More."));
        assert_eq!(g.schema.at_least_one_of, vec!["body"]);
        assert!(g.schema.key("body").is_some());
        assert!(g.accepts_common("part"));
        let fm = g.rule("frontmatter-constraint").unwrap();
        assert_eq!(fm.exactly_one_of, vec!["const", "enum"]);
        assert!(
            fm.key("format")
                .unwrap()
                .enum_
                .contains(&"email".to_string())
        );
    }

    // sokf-tests: TL_4
    #[test]
    fn a_format_grammar_that_changes_the_core_is_refused() {
        let base = |text: &str| load_core_grammar().unwrap().base(text);
        assert!(base("base: tramlines\n").is_err());
        assert!(base("grammar: fmt\nbase: other\n").is_err());
        assert!(base("grammar: fmt\nbase: tramlines\nmeta: {}\n").is_err());
        assert!(base("grammar: fmt\nbase: tramlines\nextra: {}\n").is_err());
        assert!(
            base("grammar: fmt\nbase: tramlines\nschema:\n  keys:\n    line-limit:\n      type: string\n")
                .is_err()
        );
        assert!(base("grammar: fmt\nbase: tramlines\nschema:\n  rule-common: {}\n").is_err());
    }

    // sokf-tests: TL_5
    // sokf-tests: TL_6
    // sokf-tests: TL_7
    // sokf-tests: TL_8
    #[test]
    fn a_host_extension_only_adds_to_its_format_grammar() {
        let mut g = load_core_grammar().unwrap();
        g.base(FORMAT).unwrap();
        let mut h = g.clone();
        h.extend("extension: host\nextends: fmt\nschema:\n  keys:\n    links:\n      type: list\n  part:\n    keys:\n      text:\n        doc: Also this.\nnotes:\n  a: 1\n")
            .unwrap();
        assert!(h.schema.key("links").is_some());
        assert!(h.section("notes").is_some());
        let refused = |text: &str| g.clone().extend(text);
        assert!(refused("extends: fmt\n").is_err());
        assert!(refused("extension: host\nextends: tramlines\n").is_err());
        assert!(refused("extension: host\nextends: fmt\nmeta: {}\n").is_err());
        assert!(
            refused("extension: host\nextends: fmt\nschema:\n  at-least-one-of: [links]\n")
                .is_err()
        );
        assert!(
            refused(
                "extension: host\nextends: fmt\nschema:\n  part:\n    exactly-one-of: [text]\n"
            )
            .is_err()
        );
        assert!(refused("extension: host\nextends: fmt\nschema:\n  part:\n    keys:\n      text:\n        type: integer\n").is_err());
        assert!(refused("extension: host\nextends: fmt\nnotes: {}\nnotes: {}\n").is_err());
    }

    #[test]
    fn a_grammar_whose_of_names_nothing_is_refused() {
        let mut g = load_core_grammar().unwrap();
        let format = "grammar: fmt\nbase: tramlines\nversion: '0.2'\nschema:\n  keys:\n    body:\n      type: list\n      of: nothing\n";
        let before = g.clone();
        let e = g.base(format).unwrap_err().to_string();
        assert!(e.contains("`schema.body` holds `nothing`"), "{e}");
        // A layer that fails leaves the grammar as it was.
        assert_eq!(g, before);
        let core = "grammar: x\nversion: '1'\nschema:\n  keys:\n    a:\n      type: list\n      of: nothing\n";
        assert!(parse_grammar(core).is_err());
    }

    #[test]
    fn refuses_a_broken_grammar() {
        assert!(parse_grammar("grammar: x\n").is_err());
        assert!(parse_grammar("version: '1'\nschema: {}\n").is_err());
        assert!(
            parse_grammar("grammar: x\nversion: '1'\nschema:\n  keys:\n    a:\n      type: nope\n")
                .is_err()
        );
        assert!(parse_grammar("a: &x 1\n").is_err());
        let g = parse_grammar("grammar: x\nversion: '1'\nschema:\n  keys:\n    a: {}\n").unwrap();
        assert_eq!(g.schema.key("a").unwrap().type_, TypeName::String);
        assert!(TypeName::parse("nope").is_none());
    }
}
