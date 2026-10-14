//! Checks a schema YAML against the grammar's `meta` and `schema`
//! sections generically. Each failure is a finding on the schema.

use super::{Grammar, KeyDef, RegexCache, TypeName};
use crate::{
    Finding, RepoPath, Spanned, Yaml,
    text::{self, is_slug},
};

/// Validate a schema against grammar `schema`.
// sokf-implements: TL_11
// sokf-implements: TL_12
// sokf-implements: CORE-4_AC-1
// sokf-implements: CORE-4_AC-5
// sokf:begin validate
pub fn validate_schema(
    yaml: &Spanned,
    grammar: &Grammar,
    regexes: &RegexCache,
    schema: &RepoPath,
) -> Vec<Finding> {
    // sokf:end validate
    validate_map(yaml, "schema", grammar, regexes, schema, None)
}

/// Validate one instance of rule map `rule` at `yaml`. `kind` is the
/// enclosing block's kind, for `requires: {kind}`.
///
/// # Panics
///
/// When `rule` names no rule map of the grammar: the caller's fault, since
/// the grammar refuses any `of` that names none.
// sokf:begin validate
pub fn validate_map(
    yaml: &Spanned,
    rule: &str,
    grammar: &Grammar,
    regexes: &RegexCache,
    schema: &RepoPath,
    kind: Option<&str>,
) -> Vec<Finding> {
    // sokf:end validate
    let mut v = Validator {
        grammar,
        regexes,
        schema,
        findings: Vec::new(),
    };
    v.map(yaml, rule, kind);
    v.findings
}

struct Validator<'a> {
    grammar: &'a Grammar,
    regexes: &'a RegexCache,
    schema: &'a RepoPath,
    findings: Vec<Finding>,
}

impl Validator<'_> {
    /// A value of the wrong type.
    // sokf-implements: TL_101
    fn mistyped(&mut self, line: usize, message: String) {
        self.fail_on(line, "TL_101", message);
    }

    fn fail_on(&mut self, line: usize, criterion: &str, message: String) {
        self.findings.push(Finding::error(
            self.schema.clone(),
            Some(line),
            criterion,
            message,
        ));
    }

    fn map(&mut self, yaml: &Spanned, rule: &str, kind: Option<&str>) {
        // The grammar refuses an `of` that names no rule map when it is
        // built, so only a caller that names one wrongly reaches this.
        let Some(def) = self.grammar.rule(rule) else {
            panic!("internal error: the grammar has no rule map `{rule}`");
        };
        let Some(entries) = yaml.as_map() else {
            self.mistyped(
                yaml.line,
                format!(
                    "`{rule}` must be a map, not {} {}",
                    text::article(yaml.type_name()),
                    yaml.type_name()
                ),
            );
            return;
        };
        // A format patch states only its difference, so it is checked
        // after it is merged onto the entry it shadows, not here.
        // sokf-implements: TL_40
        if rule == "format"
            && entries
                .iter()
                .any(|(k, v)| k.value == "patch" && v.as_bool() == Some(true))
        {
            return;
        }
        let own_kind = if rule == "block" {
            entries
                .iter()
                .find(|(k, _)| k.value == "kind")
                .and_then(|(_, v)| v.as_str())
                .map(str::to_string)
        } else {
            kind.map(str::to_string)
        };
        let present: Vec<&str> = entries.iter().map(|(k, _)| k.value.as_str()).collect();
        for (key, value) in entries {
            let Some(key_def) = self.grammar.key_def(rule, &key.value) else {
                let hint = if rule == "section" && key.value == "heading-from" {
                    " (`heading-from: <key>` became `heading: { from: <key> }`)"
                } else {
                    ""
                };
                // sokf-implements: TL_12
                self.fail_on(
                    key.line,
                    "TL_12",
                    format!("unknown key `{}` in `{rule}`{hint}", key.value),
                );
                continue;
            };
            self.value(value, &key.value, key_def, own_kind.as_deref());
            for required in &key_def.requires_keys {
                if !present.contains(&required.as_str()) {
                    // sokf-implements: TL_103
                    self.fail_on(
                        key.line,
                        "TL_103",
                        format!("`{}` requires `{required}` beside it", key.value),
                    );
                }
            }
            // sokf-implements: TLMD_42
            if !key_def.requires_kind.is_empty() {
                match own_kind.as_deref() {
                    Some(k) if key_def.requires_kind.iter().any(|r| r == k) => {}
                    Some(k) => self.fail_on(
                        key.line,
                        "TLMD_42",
                        format!(
                            "`{}` does not apply to {} `{k}` block; it needs kind {}",
                            key.value,
                            text::article(k),
                            key_def.requires_kind.join(" or ")
                        ),
                    ),
                    None => {}
                }
            }
        }
        for (name, key_def) in &def.keys {
            if key_def.required && !present.contains(&name.as_str()) {
                self.fail_on(yaml.line, "TL_103", format!("`{rule}` requires `{name}`"));
            }
        }
        if !def.exactly_one_of.is_empty() {
            let n = def
                .exactly_one_of
                .iter()
                .filter(|k| present.contains(&k.as_str()))
                .count();
            // A group of shapes is the one-shape rule; any other group is
            // the grammar's. None of a group is a key missing, two of it a
            // key too many.
            // sokf-implements: CORE-4_AC-3
            // sokf-implements: TL_22
            // sokf-implements: TL_104
            // sokf-implements: TL_105
            let shapes = def
                .exactly_one_of
                .iter()
                .any(|k| matches!(k.as_str(), "text" | "template" | "pattern"));
            // sokf-implements: CORE-4_AC-4
            let criterion = match (n, shapes) {
                (0, true) => "TL_105",
                (0, false) => "TL_103",
                (_, true) => "TL_22",
                (_, false) => "TL_104",
            };
            if n != 1 {
                self.fail_on(
                    yaml.line,
                    criterion,
                    format!(
                        "`{rule}` needs exactly one of {}",
                        list(&def.exactly_one_of)
                    ),
                );
            }
        }
        if !def.at_least_one_of.is_empty()
            && !def
                .at_least_one_of
                .iter()
                .any(|k| present.contains(&k.as_str()))
        {
            self.fail_on(
                yaml.line,
                "TL_103",
                format!(
                    "`{rule}` needs at least one of {}",
                    list(&def.at_least_one_of)
                ),
            );
        }
    }

    fn value(&mut self, value: &Spanned, key: &str, def: &KeyDef, kind: Option<&str>) {
        let of = def.of.as_deref();
        match def.type_ {
            TypeName::String => self.scalar(value, key, def),
            TypeName::Integer => {
                if value.as_int().is_none() {
                    self.mistyped(value.line, format!("`{key}` must be an integer"));
                } else {
                    self.enum_check(value, key, def);
                }
            }
            TypeName::Boolean => {
                if value.as_bool().is_none() {
                    self.mistyped(value.line, format!("`{key}` must be true or false"));
                }
            }
            TypeName::List => match value.as_seq() {
                Some(items) => {
                    for item in items {
                        self.item(item, key, of, def, kind);
                    }
                }
                None => self.mistyped(value.line, format!("`{key}` must be a list")),
            },
            TypeName::Map => match value.as_map() {
                Some(entries) => {
                    for (_, item) in entries {
                        self.item(item, key, of, def, kind);
                    }
                }
                None => self.mistyped(value.line, format!("`{key}` must be a map")),
            },
            TypeName::Object => self.item(value, key, of, def, kind),
            TypeName::StringOrList => match &value.value {
                Yaml::Str(_) => self.scalar(value, key, def),
                // Each item is held to the key's format and enum as the
                // single string is.
                Yaml::Seq(items) => {
                    for item in items {
                        match item.as_str() {
                            Some(_) => self.scalar(item, key, def),
                            None => {
                                self.mistyped(item.line, format!("`{key}` items must be strings"));
                            }
                        }
                    }
                }
                _ => self.mistyped(value.line, format!("`{key}` must be a string or a list")),
            },
            TypeName::Text => match &value.value {
                Yaml::Str(_) => self.scalar(value, key, def),
                Yaml::Map(entries) if is_from(entries) => {}
                _ => self.mistyped(
                    value.line,
                    format!("`{key}` must be a string or `{{ from: <frontmatter key> }}`"),
                ),
            },
            TypeName::TextOrRule => match &value.value {
                Yaml::Str(_) => self.scalar(value, key, def),
                Yaml::Map(entries) if is_from(entries) => {}
                Yaml::Map(_) => self.item(value, key, of, def, kind),
                _ => self.mistyped(
                    value.line,
                    format!(
                        "`{key}` must be a string, `{{ from: <frontmatter key> }}` or a `{}` map",
                        of.unwrap_or("rule")
                    ),
                ),
            },
            TypeName::StringOrMap => match &value.value {
                Yaml::Str(_) => self.scalar(value, key, def),
                Yaml::Map(_) => {}
                _ => self.mistyped(value.line, format!("`{key}` must be a string or a map")),
            },
        }
    }

    /// One item of a list, value of a map, or instance of an object.
    fn item(
        &mut self,
        item: &Spanned,
        key: &str,
        of: Option<&str>,
        def: &KeyDef,
        kind: Option<&str>,
    ) {
        match of {
            None => {}
            // An item is held to the key's format and enum as a scalar
            // is, so a list of regexes reports the one that does not parse.
            // sokf-implements: TL_17
            Some("string") => match item.as_str() {
                Some(_) => self.scalar(item, key, def),
                None => self.mistyped(item.line, format!("`{key}` items must be strings")),
            },
            Some("integer") => {
                if item.as_int().is_none() {
                    self.mistyped(item.line, format!("`{key}` items must be integers"));
                }
            }
            Some("boolean") => {
                if item.as_bool().is_none() {
                    self.mistyped(item.line, format!("`{key}` items must be booleans"));
                }
            }
            Some("string-or-list") => {
                if item.as_str_or_list().is_none() {
                    self.mistyped(
                        item.line,
                        format!("`{key}` values must be strings or lists of strings"),
                    );
                }
            }
            Some(name) if self.grammar.rule(name).is_some() => {
                let inner_kind = if name == "block" { None } else { kind };
                self.map(item, name, inner_kind);
            }
            // The grammar refuses an `of` that names nothing when it is
            // built.
            Some(other) => {
                unreachable!("the grammar names an unknown shape `{other}` for `{key}`");
            }
        }
    }

    fn scalar(&mut self, value: &Spanned, key: &str, def: &KeyDef) {
        let Some(text) = value.as_str() else {
            self.mistyped(
                value.line,
                format!(
                    "`{key}` must be a string, not {} {}",
                    text::article(value.type_name()),
                    value.type_name()
                ),
            );
            return;
        };
        self.enum_check(value, key, def);
        match def.format.as_deref() {
            Some("regex") => {
                if let Err(e) = self.regexes.get(text) {
                    // sokf-implements: CORE-4_AC-2
                    // sokf-implements: TL_118
                    self.fail_on(
                        value.line,
                        "TL_118",
                        format!("`{key}` is not a valid regex: {e}"),
                    );
                }
            }
            Some("slug") if !is_slug(text) => {
                // sokf-implements: TL_102
                self.fail_on(
                    value.line,
                    "TL_102",
                    format!("`{key}` must be a slug: `{text}` is not"),
                );
            }
            // References resolve when the schema is lowered.
            _ => {}
        }
    }

    fn enum_check(&mut self, value: &Spanned, key: &str, def: &KeyDef) {
        if def.enum_.is_empty() {
            return;
        }
        let text = value.scalar_text().unwrap_or_default();
        if !def.enum_.contains(&text) {
            self.fail_on(
                value.line,
                "TL_102",
                format!(
                    "`{key}` must be one of {}; `{text}` is not",
                    list(&def.enum_)
                ),
            );
        }
    }
}

/// Whether a map is `{ from: <key> }`, which binds a text to a frontmatter
/// key.
fn is_from(entries: &[(Spanned<String>, Spanned)]) -> bool {
    matches!(entries, [(k, v)] if k.value == "from" && v.as_str().is_some())
}

fn list(items: &[String]) -> String {
    text::code_list(items, ", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grammar::load_core_grammar, yaml};

    fn check(text: &str) -> Vec<String> {
        let grammar = load_core_grammar().unwrap();
        let yaml = yaml::parse(text, 1).unwrap();
        validate_schema(&yaml, &grammar, &RegexCache::new(), &RepoPath::root())
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    // sokf-tests: TL_11
    // sokf-tests: TL_12
    // sokf-tests: TL_102
    // sokf-tests: TL_103
    #[test]
    fn the_core_keys_are_checked_against_the_core_grammar() {
        assert!(check("line-limit: 0\nfrontmatter:\n  title:\n    example: A\n").is_empty());
        let out = check(
            "frontmatter:\n  title:\n    format: nope\n    list: 1\nlifecycle:\n  - key: Not Slug\n    states: [a]\nsections: []\n",
        );
        assert!(
            out.iter().any(|m| m.contains("requires `line-limit`")),
            "{out:?}"
        );
        assert!(
            out.iter().any(|m| m.contains("unknown key `sections`")),
            "{out:?}"
        );
        assert!(
            out.iter().any(|m| m.contains("`format` must be one of")),
            "{out:?}"
        );
        assert!(out.iter().any(|m| m.contains("must be a slug")), "{out:?}");
    }
}
