//! The three-layer registry of line formats and the hole types they are
//! written with (TL_34 to TL_41).

use std::collections::BTreeMap;

use saphyr::{Mapping, Scalar, Sequence, Yaml as SYaml, YamlEmitter};

use super::{
    Format, Grammar, RegexCache, TypeDef, TypeRegistry, format::lower_format, patch::merge_patch,
    validate,
};
use crate::{Finding, Problem, RepoPath, Result, Spanned, Yaml, yaml};

/// Where a registry entry comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSource {
    /// Defined by the tool itself: the built-in types.
    BuiltIn,
    /// The tool's embedded formats.
    Embedded,
    /// The host's layer, as a formats file the host reads.
    Repo,
    /// A formats file that the caller names, by its path as the caller
    /// spells it.
    File(String),
    /// A schema's `formats`, by schema id.
    Schema(String),
}

impl FormatSource {
    /// The `source` value a printed registry carries.
    pub fn label(&self) -> &str {
        match self {
            FormatSource::BuiltIn => "built-in",
            FormatSource::Embedded => "embedded",
            FormatSource::Repo => "repo",
            FormatSource::File(path) => path,
            FormatSource::Schema(id) => id,
        }
    }
}

// sokf:begin format-registry
/// A named set of formats, each with its source, and the hole types the
/// layers declared.
#[derive(Debug, Clone, Default)]
pub struct FormatRegistry {
    /// The entries, by name.
    pub entries: BTreeMap<String, (Format, FormatSource)>,
    /// The types the layers declared, the built-ins apart.
    pub types: TypeRegistry,
}
// sokf:end format-registry

impl FormatRegistry {
    /// A host's embedded registry: the built-in types it adds, reserved as
    /// the engine's are, then the formats file `text`, whose entries are
    /// `embedded`. A file that does not read is a defect of the build.
    // sokf:begin format-registry
    pub fn from_text(
        text: &str,
        built_ins: &[(&str, &str)],
        grammar: &Grammar,
        regexes: &RegexCache,
    ) -> Result<Self> {
        // sokf:end format-registry
        // sokf-implements: CORE-5_AC-9
        let fail = |m: String| crate::Error::Internal(format!("embedded formats: {m}"));
        // sokf-implements: CORE-5_AC-7
        let base = FormatRegistry {
            entries: BTreeMap::new(),
            types: TypeRegistry::reserved(built_ins),
        };
        let root = yaml::parse(text, 1).map_err(|e| fail(e.to_string()))?;
        let registry = root
            .get("registry")
            .ok_or_else(|| fail("no registry".into()))?;
        let own = Self::from_yaml_at(
            registry,
            root.get("types"),
            FormatSource::Embedded,
            grammar,
            regexes,
            &RepoPath::root(),
            Some(&base),
        )
        .map_err(|findings| {
            fail(
                findings
                    .iter()
                    .map(|f| f.message.clone())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;
        Ok(base.layered(&own))
    }

    /// Build a registry from a `registry` map with no types of its own,
    /// validating each entry against grammar `format`. Findings are on `/`.
    // sokf-implements: TL_34
    // sokf-implements: TL_36
    pub fn from_yaml(
        registry: &Spanned,
        source: FormatSource,
        grammar: &Grammar,
        regexes: &RegexCache,
    ) -> std::result::Result<Self, Vec<Finding>> {
        Self::from_yaml_at(
            registry,
            None,
            source,
            grammar,
            regexes,
            &RepoPath::root(),
            None,
        )
    }

    /// As `from_yaml`, with the layer's `types` map, findings on `path`,
    /// and the farther layers in `base`: an entry declaring `patch` is
    /// merged onto the same-named entry of `base` before it is checked
    /// (grammar `format`), and every template compiles against the types of
    /// `base` with this layer's on top.
    // sokf-implements: TL_27
    // sokf-implements: CORE-5_AC-1
    // sokf-implements: CORE-5_AC-2
    // sokf-implements: TL_35
    // sokf:begin format-registry
    pub fn from_yaml_at(
        registry: &Spanned,
        types: Option<&Spanned>,
        source: FormatSource,
        grammar: &Grammar,
        regexes: &RegexCache,
        path: &RepoPath,
        base: Option<&FormatRegistry>,
    ) -> std::result::Result<Self, Vec<Finding>> {
        // sokf:end format-registry
        let (out, findings) = Self::layer(registry, types, source, grammar, regexes, path, base);
        if findings.is_empty() {
            Ok(out)
        } else {
            Err(findings)
        }
    }

    /// As `from_yaml_at`, keeping what loads beside the findings of what
    /// does not: each type and format that fails is reported once, and
    /// what uses it reports nothing more. A type that fails stands as one
    /// that matches anything, so a template that names it still compiles;
    /// a format that fails is left out.
    // sokf-implements: TL_125
    // sokf-implements: CORE-5_AC-12
    // sokf:begin format-registry
    pub fn layer(
        registry: &Spanned,
        types: Option<&Spanned>,
        source: FormatSource,
        grammar: &Grammar,
        regexes: &RegexCache,
        path: &RepoPath,
        base: Option<&FormatRegistry>,
    ) -> (Self, Vec<Finding>) {
        // sokf:end format-registry
        let fail = |line: usize, problem: Problem| problem.error(path.clone(), Some(line));
        let mut findings = Vec::new();
        let below = base.map(|b| b.types.clone()).unwrap_or_default();
        let own_types = match types {
            Some(node) => {
                // sokf-implements: CORE-5_AC-8
                let (mut own, problems) = TypeRegistry::from_yaml(node, &source, &below);
                findings.extend(problems.into_iter().map(|(l, m)| fail(l, m)));
                let failed = node
                    .as_map()
                    .unwrap_or(&[])
                    .iter()
                    .map(|(name, _)| &name.value)
                    .filter(|name| !own.entries.contains_key(*name) && !below.reserves(name));
                for name in failed.collect::<Vec<_>>() {
                    own.entries
                        .insert(name.clone(), (TypeDef::Regex(".*?".into()), source.clone()));
                }
                own
            }
            None => TypeRegistry::default(),
        };
        let seen = TypeRegistry::built_ins()
            .layered(&below)
            .layered(&own_types);
        // A layer's registry carries its own types alone; the farther
        // layers' come back when it is layered on them.
        let mut out = FormatRegistry {
            entries: BTreeMap::new(),
            types: own_types,
        };
        // sokf-implements: CORE-5_AC-5
        let Some(entries) = registry.as_map() else {
            findings.push(fail(
                registry.line,
                Problem::new("TL_101", "the format registry must be a map of formats"),
            ));
            return (out, findings);
        };
        for (name, whole) in entries {
            // A patch says how it differs from the entry it shadows, so it
            // is merged first and checked as the entry it becomes.
            // sokf-implements: CORE-5_AC-3
            // sokf-implements: TL_37
            // sokf-implements: TL_41
            let merged;
            let entry = if whole.get("patch").and_then(|p| p.as_bool()) == Some(true) {
                match base.and_then(|b| b.entries.get(&name.value)) {
                    Some((f, _)) => {
                        merged = merge_patch(&f.raw, whole);
                        &merged
                    }
                    None => {
                        findings.push(fail(
                            whole.line,
                            Problem::new(
                                "TL_41",
                                format!("format `{}` patches no farther entry", name.value),
                            ),
                        ));
                        continue;
                    }
                }
            } else {
                whole
            };
            // sokf-implements: CORE-5_AC-4
            let mut problems =
                validate::validate_map(entry, "format", grammar, regexes, path, None);
            if problems.is_empty() {
                match lower_format(entry, &seen, regexes) {
                    Ok(format) => {
                        out.entries
                            .insert(name.value.clone(), (format, source.clone()));
                        continue;
                    }
                    Err(said) => problems.extend(said.into_iter().map(|(l, m)| fail(l, m))),
                }
            }
            // sokf-implements: CORE-5_AC-6
            for p in &mut problems {
                p.message = format!("format `{}`: {}", name.value, p.message);
            }
            findings.extend(problems);
        }
        (out, findings)
    }

    /// The types a schema under this registry sees: the built-ins, then
    /// the layers' own.
    pub fn types_seen(&self) -> TypeRegistry {
        TypeRegistry::built_ins().layered(&self.types)
    }

    /// This registry with `over` on top: a same-named entry of `over`
    /// replaces this one whole, and a same-named type likewise. An entry of
    /// `over` that declared a patch was merged onto the farther one when it
    /// was built. Where `over` declares types, this registry's entries are
    /// compiled again against them, so a type means one thing to every
    /// format a schema sees.
    // sokf-implements: TL_34
    // sokf-implements: TL_36
    // sokf:begin format-registry
    // sokf-implements: TL_51
    pub fn layered(&self, over: &FormatRegistry) -> FormatRegistry {
        // sokf:end format-registry
        let types = self.types.layered(&over.types);
        let seen = TypeRegistry::built_ins().layered(&types);
        let regexes = RegexCache::new();
        let mut entries = self.entries.clone();
        // sokf-implements: CORE-5_AC-11
        // Only a type this registry already defines can change one of its
        // formats: a new name is one no farther template uses.
        let redefined: Vec<&str> = over
            .types
            .entries
            .keys()
            .filter(|n| self.types.entries.contains_key(*n))
            .map(String::as_str)
            .collect();
        if !redefined.is_empty() {
            for (format, _) in entries.values_mut() {
                if format.uses_any(&redefined)
                    && let Ok(again) = lower_format(&format.raw, &seen, &regexes)
                {
                    *format = again;
                }
            }
        }
        // sokf-implements: CORE-5_AC-10
        for (name, entry) in &over.entries {
            entries.insert(name.clone(), entry.clone());
        }
        FormatRegistry { entries, types }
    }

    /// The format named `name`.
    // sokf:begin format-registry
    pub fn get(&self, name: &str) -> Option<&Format> {
        // sokf:end format-registry
        self.entries.get(name).map(|(f, _)| f)
    }

    /// The registry as one YAML document in the shape of
    /// a formats file, each type and entry carrying `source`. The
    /// built-in types are printed where `built_ins` asks for them.
    pub fn to_yaml(&self, built_ins: bool) -> String {
        let types = if built_ins {
            self.types_seen()
        } else {
            self.types.clone()
        };
        let mut type_map = Mapping::new();
        for (name, (def, source)) in &types.entries {
            let value = match def {
                TypeDef::Regex(r) => ystr(r),
                TypeDef::Choices(c) => SYaml::Sequence(c.iter().map(ystr).collect::<Sequence>()),
            };
            let mut entry = Mapping::new();
            entry.insert(ystr("match"), value);
            entry.insert(ystr("source"), ystr(source.label()));
            type_map.insert(ystr(name), SYaml::Mapping(entry));
        }
        let mut registry = Mapping::new();
        for (name, (format, source)) in &self.entries {
            let mut entry = match to_saphyr(&format.raw) {
                SYaml::Mapping(m) => m,
                _ => Mapping::new(),
            };
            entry.insert(ystr("source"), ystr(source.label()));
            registry.insert(ystr(name), SYaml::Mapping(entry));
        }
        let mut root = Mapping::new();
        if !type_map.is_empty() {
            root.insert(ystr("types"), SYaml::Mapping(type_map));
        }
        root.insert(ystr("registry"), SYaml::Mapping(registry));
        let mut out = String::new();
        let mut emitter = YamlEmitter::new(&mut out);
        emitter.multiline_strings(true);
        emitter
            .dump(&SYaml::Mapping(root))
            .expect("emitting to a string cannot fail");
        out.push('\n');
        out.strip_prefix("---\n").map(str::to_string).unwrap_or(out)
    }
}

fn to_saphyr(node: &Spanned) -> SYaml<'static> {
    match &node.value {
        Yaml::Null => SYaml::Value(Scalar::Null),
        Yaml::Bool(b) => SYaml::Value(Scalar::Boolean(*b)),
        Yaml::Int(i) => SYaml::Value(Scalar::Integer(*i)),
        Yaml::Float(f) => SYaml::Value(Scalar::FloatingPoint((*f).into())),
        Yaml::Str(s) => ystr(s),
        Yaml::Seq(items) => SYaml::Sequence(items.iter().map(to_saphyr).collect::<Sequence>()),
        Yaml::Map(entries) => {
            let mut m = Mapping::new();
            for (k, v) in entries {
                m.insert(ystr(&k.value), to_saphyr(v));
            }
            SYaml::Mapping(m)
        }
    }
}

/// A YAML string scalar of `s`.
fn ystr(s: impl AsRef<str>) -> SYaml<'static> {
    SYaml::Value(Scalar::String(s.as_ref().to_string().into()))
}
