//! The layered registry of line formats and the hole types they are
//! written with (TL_34 to TL_41): the schema, the host, the pack layer,
//! and the shared layer.

use std::{borrow::Cow, collections::BTreeMap};

use saphyr::{Mapping, Scalar, Sequence, Yaml as SYaml, YamlEmitter};

use super::{
    Format, Grammar, RegexCache, TypeDef, TypeRegistry, format::lower_format, patch::merge_patch,
    validate,
};
use crate::{Finding, Problem, RepoPath, Result, Spanned, Yaml, yaml};

// sokf:begin format-source
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
    /// A formats file of the pack layer, by the name of its pack.
    Pack(String),
}
// sokf:end format-source

impl FormatSource {
    /// The `source` value a printed registry carries.
    pub fn label(&self) -> Cow<'_, str> {
        match self {
            FormatSource::BuiltIn => Cow::Borrowed("built-in"),
            FormatSource::Embedded => Cow::Borrowed("embedded"),
            FormatSource::Repo => Cow::Borrowed("repo"),
            FormatSource::File(path) => Cow::Borrowed(path),
            FormatSource::Schema(id) => Cow::Borrowed(id),
            // sokf-implements: CORE-5_AC-15
            FormatSource::Pack(name) => Cow::Owned(format!("pack:{name}")),
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
                    Ok(mut format) => {
                        format.patched = !std::ptr::eq(entry, whole);
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
                    && let Ok(mut again) = lower_format(&format.raw, &seen, &regexes)
                {
                    again.patched = format.patched;
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

    /// This registry, the shared layer, with the pack layer on top: the
    /// entries and the types of `files`, each a formats file of the layer
    /// built with `layer` on this registry with the source
    /// `FormatSource::Pack`. The files are read in the order of their
    /// paths, whatever order the caller gives, so no entry patches another
    /// file's; where two files define one name, the entry of the first
    /// stands, and the layer keeps the clash for `PackLayer::with_host`.
    /// Each format that uses a type of the layer is compiled again against
    /// it, so a type means one thing to every format that a schema sees,
    /// and a format that no longer compiles is left out and reported.
    // sokf-implements: TL_34
    // sokf-implements: TL_27
    // sokf-implements: CORE-5_AC-13
    // sokf:begin format-registry
    pub fn with_packs(&self, files: &[(RepoPath, FormatRegistry)]) -> PackLayer {
        // sokf:end format-registry
        let mut files: Vec<&(RepoPath, FormatRegistry)> = files.iter().collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut packs = FormatRegistry::default();
        let mut origin: BTreeMap<&str, &RepoPath> = BTreeMap::new();
        let mut type_origin: BTreeMap<&str, &RepoPath> = BTreeMap::new();
        let mut clashes = Vec::new();
        for (path, file) in files.iter().copied() {
            for (name, entry) in &file.entries {
                match origin.get(name.as_str()) {
                    Some(first) => clashes.push(Clash {
                        what: "format",
                        name: name.clone(),
                        first: (*first).clone(),
                        later: path.clone(),
                        line: Some(entry.0.raw.line),
                    }),
                    None => {
                        origin.insert(name, path);
                        packs.entries.insert(name.clone(), entry.clone());
                    }
                }
            }
            for (name, def) in &file.types.entries {
                match type_origin.get(name.as_str()) {
                    Some(first) => clashes.push(Clash {
                        what: "type",
                        name: name.clone(),
                        first: (*first).clone(),
                        later: path.clone(),
                        line: None,
                    }),
                    None => {
                        type_origin.insert(name, path);
                        packs.types.entries.insert(name.clone(), def.clone());
                    }
                }
            }
        }
        // A file's formats were compiled against its own types, and a
        // type of another file can stand in their place.
        let names: Vec<&str> = packs.types.entries.keys().map(String::as_str).collect();
        let seen = self.types_seen().layered(&packs.types);
        let regexes = RegexCache::new();
        let mut findings = Vec::new();
        let mut again: Vec<(String, Option<Format>)> = Vec::new();
        for (name, (format, _)) in &packs.entries {
            if !format.uses_any(&names) {
                continue;
            }
            // sokf-implements: TL_134
            // sokf-implements: CORE-5_AC-16
            match lower_format(&format.raw, &seen, &regexes) {
                Ok(mut lowered) => {
                    lowered.patched = format.patched;
                    again.push((name.clone(), Some(lowered)));
                }
                Err(said) => {
                    let path = origin[name.as_str()];
                    findings.extend(said.into_iter().map(|(line, mut problem)| {
                        problem.message = format!("format `{name}`: {}", problem.message);
                        problem.error(path.clone(), Some(line))
                    }));
                    again.push((name.clone(), None));
                }
            }
        }
        for (name, format) in again {
            match format {
                Some(format) => {
                    if let Some(entry) = packs.entries.get_mut(&name) {
                        entry.0 = format;
                    }
                }
                None => {
                    packs.entries.remove(&name);
                }
            }
        }
        PackLayer {
            registry: self.layered(&packs),
            clashes,
            findings,
        }
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

// sokf:begin format-registry
/// The shared layer with the pack layer on top, which the host builds its
/// own layer on, and what the pack layer reports once the host's layer is
/// known.
#[derive(Debug, Clone)]
#[must_use = "`PackLayer::with_host` gives the registry and the findings of the pack layer"]
pub struct PackLayer {
    registry: FormatRegistry,
    clashes: Vec<Clash>,
    findings: Vec<Finding>,
}
// sokf:end format-registry

/// A format or a type that two files of the pack layer define: the first
/// and the later file in the order of their paths, and the line of the
/// later entry where the layer keeps one.
#[derive(Debug, Clone)]
struct Clash {
    what: &'static str,
    name: String,
    first: RepoPath,
    later: RepoPath,
    line: Option<usize>,
}

impl PackLayer {
    /// The shared layer with the pack layer on top, the base of the host's
    /// own layer.
    // sokf:begin format-registry
    pub fn registry(&self) -> &FormatRegistry {
        // sokf:end format-registry
        &self.registry
    }

    /// The registry with `host`, the host's own layer built on `registry`,
    /// on top, and the findings of the pack layer: each format that does
    /// not compile against the types of the layer, and each format and
    /// type that two files define and `host` does not define in full. A
    /// patch in `host` settles no clash, because it was merged onto the
    /// entry of whichever file came first.
    // sokf-implements: TL_128
    // sokf-implements: CORE-5_AC-14
    // sokf:begin format-registry
    pub fn with_host(&self, host: &FormatRegistry) -> (FormatRegistry, Vec<Finding>) {
        // sokf:end format-registry
        let settled = |c: &Clash| match c.what {
            "format" => host.entries.get(&c.name).is_some_and(|(f, _)| !f.patched),
            _ => host.types.entries.contains_key(&c.name),
        };
        let mut findings = self.findings.clone();
        findings.extend(self.clashes.iter().filter(|c| !settled(c)).map(|c| {
            Problem::new(
                "TL_128",
                format!(
                    "{} `{}` is defined in both `{}` and `{}` of the pack layer; define it in full in the host's layer to choose one",
                    c.what, c.name, c.first, c.later
                ),
            )
            .error(c.later.clone(), c.line)
        }));
        (self.registry.layered(host), findings)
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
