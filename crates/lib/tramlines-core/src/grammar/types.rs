//! Hole types of line templates (TL_27 to TL_29, TL_109): a named regex or a list of
//! literal choices, in layers as formats are, the built-ins reserved.

use std::collections::BTreeMap;

use super::FormatSource;
use crate::{Problem, Spanned, text};

/// A slug (TLMD_101): lowercase words of letters and digits, joined by
/// single hyphens.
pub const SLUG: &str = "[a-z0-9]+(?:-[a-z0-9]+)*";

/// The types the engine defines and no layer may redefine: the plain ones
/// and the id holes a new document is filled with. A host may add built-ins
/// of its own (`FormatRegistry::from_text`), reserved likewise.
// sokf-implements: TL_28
pub const BUILT_INS: [(&str, &str); 5] = [
    ("text", ".+"),
    ("word", r"\w+"),
    ("int", r"\d+"),
    ("slug", SLUG),
    ("seq", r"\d+"),
];

/// What a type matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    /// A regex with no anchors and no capture groups.
    Regex(String),
    /// Literal choices, each escaped.
    Choices(Vec<String>),
}

impl TypeDef {
    /// The regex the type compiles to, as one group that captures nothing.
    pub fn regex(&self) -> String {
        match self {
            TypeDef::Regex(r) => format!("(?:{r})"),
            TypeDef::Choices(c) => format!(
                "(?:{})",
                c.iter()
                    .map(|s| regex::escape(s))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        }
    }

    /// The type as a failure names it: its regex, or its choices.
    pub fn describe(&self) -> String {
        match self {
            TypeDef::Regex(r) => format!("`{r}`"),
            TypeDef::Choices(c) => format!("one of {}", text::code_list(c, ", ")),
        }
    }
}

/// Types by name, each with the layer it comes from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeRegistry {
    /// The types, by name.
    pub entries: BTreeMap<String, (TypeDef, FormatSource)>,
}

impl TypeRegistry {
    /// The built-ins alone.
    pub fn built_ins() -> Self {
        Self::reserved(&BUILT_INS)
    }

    /// The regex types `types`, each reserved as a built-in is.
    pub(crate) fn reserved(types: &[(&str, &str)]) -> Self {
        let entries = types
            .iter()
            .map(|(name, re)| {
                (
                    (*name).to_string(),
                    (TypeDef::Regex((*re).to_string()), FormatSource::BuiltIn),
                )
            })
            .collect();
        TypeRegistry { entries }
    }

    /// The type named `name`.
    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.entries.get(name).map(|(t, _)| t)
    }

    /// Whether `name` is a built-in, which no layer redefines.
    pub fn is_built_in(name: &str) -> bool {
        BUILT_INS.iter().any(|(n, _)| *n == name)
    }

    /// Whether `name` is built in here: the engine's, or a host's.
    pub fn reserves(&self, name: &str) -> bool {
        Self::is_built_in(name)
            || self
                .entries
                .get(name)
                .is_some_and(|(_, s)| *s == FormatSource::BuiltIn)
    }

    /// This registry with the types of `over` on top: the nearest layer
    /// wins by name, and a built-in stays as it is.
    // sokf-implements: TL_27
    pub fn layered(&self, over: &TypeRegistry) -> TypeRegistry {
        let mut entries = self.entries.clone();
        for (name, entry) in &over.entries {
            if !self.reserves(name) {
                entries.insert(name.clone(), entry.clone());
            }
        }
        TypeRegistry { entries }
    }

    /// The types a `types` map declares, from `source`, each checked: a
    /// name no built-in holds, and a regex that compiles, with no anchor
    /// and no capture group, or a list of strings. The problems are
    /// `(line, message)`.
    // sokf-implements: TL_29
    pub fn from_yaml(
        node: &Spanned,
        source: &FormatSource,
        below: &TypeRegistry,
    ) -> (TypeRegistry, Vec<(usize, Problem)>) {
        let mut entries = BTreeMap::new();
        let mut problems = Vec::new();
        let Some(map) = node.as_map() else {
            problems.push((
                node.line,
                Problem::new("TL_101", "`types` must be a map of types"),
            ));
            return (TypeRegistry { entries }, problems);
        };
        for (name, value) in map {
            if below.reserves(&name.value) {
                problems.push((
                    name.line,
                    Problem::new(
                        "TL_29",
                        format!(
                            "type `{}` is built in and reserved; name the type otherwise",
                            name.value
                        ),
                    ),
                ));
                continue;
            }
            if !crate::text::is_slug(&name.value) {
                problems.push((
                    name.line,
                    Problem::new(
                        "TL_102",
                        format!("type `{}` must be named by a slug", name.value),
                    ),
                ));
                continue;
            }
            let def = if let Some(re) = value.as_str() {
                match check_type_regex(re) {
                    Ok(()) => TypeDef::Regex(re.to_string()),
                    Err((criterion, e)) => {
                        problems.push((
                            value.line,
                            Problem::new(criterion, format!("type `{}`: {e}", name.value)),
                        ));
                        continue;
                    }
                }
            } else if let Some(items) = value.as_seq() {
                let choices: Vec<String> = items
                    .iter()
                    .filter_map(|i| i.scalar_text())
                    .filter(|s| !s.is_empty())
                    .collect();
                if choices.len() != items.len() || choices.is_empty() {
                    problems.push((
                        value.line,
                        Problem::new(
                            "TL_101",
                            format!(
                                "type `{}`: a list of choices holds one or more non-empty strings",
                                name.value
                            ),
                        ),
                    ));
                    continue;
                }
                TypeDef::Choices(choices)
            } else {
                problems.push((
                    value.line,
                    Problem::new(
                        "TL_101",
                        format!("type `{}` must be a regex or a list of choices", name.value),
                    ),
                ));
                continue;
            };
            entries.insert(name.value.clone(), (def, source.clone()));
        }
        (TypeRegistry { entries }, problems)
    }
}

/// Whether `name` names a hole or a named group: a letter or `_`, then
/// letters, digits and `_`, which is what a regex group name admits.
pub fn is_hole_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A type's regex compiles, and holds no anchor and no capture group: the
/// template anchors the whole line, and names its own groups.
// sokf-implements: TL_109
// sokf-implements: TL_118
fn check_type_regex(re: &str) -> Result<(), (&'static str, String)> {
    let compiled = regex::Regex::new(re).map_err(|e| {
        (
            "TL_118",
            format!("not a valid regex: {}", super::regex::reason(&e)),
        )
    })?;
    if compiled.captures_len() > 1 {
        return Err((
            "TL_109",
            format!(
                "`{re}` holds a capture group; write `(?:…)`, since a template names its own groups"
            ),
        ));
    }
    if has_anchor(re) {
        return Err((
            "TL_109",
            format!("`{re}` holds an anchor; a template matches the whole line itself"),
        ));
    }
    Ok(())
}

/// Whether `re` holds `^` or `$` outside a character class and unescaped.
fn has_anchor(re: &str) -> bool {
    let mut class = 0usize;
    let mut chars = re.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '[' => class += 1,
            ']' if class > 0 => class -= 1,
            '^' | '$' if class == 0 => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml;

    fn types(text: &str) -> (TypeRegistry, Vec<(usize, Problem)>) {
        let node = yaml::parse(text, 1).unwrap();
        TypeRegistry::from_yaml(&node, &FormatSource::Repo, &TypeRegistry::default())
    }

    // sokf-tests: TL_29
    #[test]
    fn built_ins_are_there_and_reserved() {
        let r = TypeRegistry::built_ins();
        for (name, _) in BUILT_INS {
            assert!(r.get(name).is_some(), "{name}");
        }
        let (_, problems) = types("slug: '[a-z]+'\nseq: x\n");
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].1.message.contains("reserved"), "{problems:?}");
    }

    #[test]
    fn a_type_is_a_regex_or_a_list() {
        let (r, problems) = types("code: '[A-Z]{1,10}'\nkind: [a, b.c]\n");
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(r.get("code").unwrap().regex(), "(?:[A-Z]{1,10})");
        assert_eq!(r.get("kind").unwrap().regex(), r"(?:a|b\.c)");
        assert_eq!(r.get("kind").unwrap().describe(), "one of `a`, `b.c`");
    }

    #[test]
    fn refuses_anchors_groups_bad_regexes_and_empty_lists() {
        for bad in [
            "x: '^a'\n",
            "x: 'a$'\n",
            "x: '(a)'\n",
            "x: '(?P<n>a)'\n",
            "x: '('\n",
            "x: []\n",
            "x: 3\n",
        ] {
            let (_, problems) = types(bad);
            assert_eq!(problems.len(), 1, "{bad}: {problems:?}");
        }
        // Escaped and bracketed, `^` and `$` are text.
        let (_, problems) = types("x: '[$^]\\$\\^'\ny: '(?:a|b)'\n");
        assert!(problems.is_empty(), "{problems:?}");
    }

    // sokf-tests: TL_27
    #[test]
    fn the_nearest_layer_wins_but_never_over_a_built_in() {
        let (repo, _) = types("code: '[A-Z]+'\n");
        let (schema, _) = types("code: '[A-Z]{2}'\n");
        let r = TypeRegistry::built_ins().layered(&repo).layered(&schema);
        assert_eq!(r.get("code").unwrap().regex(), "(?:[A-Z]{2})");
        assert_eq!(r.get("text").unwrap().regex(), "(?:.+)");
    }
}
