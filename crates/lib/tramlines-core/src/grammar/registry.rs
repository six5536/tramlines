//! The layered registry of line formats and the hole types they are
//! written with (`TL_34` to `TL_41`): the schema, the host, the pack layer,
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

// sokf:begin format-registry
/// A format of the receiver of `layered` that no longer compiles against
/// the types of the nearer layer, which `layered` leaves out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenFormat {
    /// The name of the format.
    pub name: String,
    /// Each type of the nearer layer that the format uses and that differs
    /// from the farther one, in the order in which its holes name them.
    pub types: Vec<String>,
    /// Each reason, its message naming the format, its layer, and the
    /// types.
    pub problems: Vec<Problem>,
}
// sokf:end format-registry

impl BrokenFormat {
    /// The format `name` of the layer `source`, which `types` break, with
    /// each reason of `said`. Each line of `said` lies in the file of the
    /// format, which no finding lies on, so the line is dropped.
    fn of(name: &str, source: &FormatSource, types: &[&str], said: Vec<(usize, Problem)>) -> Self {
        Self::of_layers(name, source, types, &[], &RepoPath::root(), said)
    }

    /// As `of`, where the types `host` of the host's file `path` take part
    /// beside `types`, each group in the order of the holes, and a group
    /// that is empty goes unnamed.
    fn of_layers(
        name: &str,
        source: &FormatSource,
        types: &[&str],
        host: &[&str],
        path: &RepoPath,
        said: Vec<(usize, Problem)>,
    ) -> Self {
        let group = |types: &[&str], whose: &str| {
            let quoted: Vec<String> = types.iter().map(|t| format!("`{t}`")).collect();
            let what = if quoted.len() == 1 { "type" } else { "types" };
            format!("the {what} {} of {whose}", quoted.join(", "))
        };
        let mut groups = Vec::new();
        if !types.is_empty() {
            groups.push(group(types, "this layer"));
        }
        if !host.is_empty() {
            groups.push(group(host, &format!("`{path}`")));
        }
        let prefix = format!(
            "format `{name}`: the entry of the layer `{}` no longer compiles against {}: ",
            source.label(),
            groups.join(" and ")
        );
        BrokenFormat {
            name: name.to_string(),
            types: types.iter().chain(host).map(|t| (*t).to_string()).collect(),
            problems: said.into_iter().map(|(_, p)| p.prefixed(&prefix)).collect(),
        }
    }

    /// Each reason as an error finding on `path` at `line`.
    // sokf:begin format-registry
    pub fn findings(&self, path: &RepoPath, line: Option<usize>) -> Vec<Finding> {
        // sokf:end format-registry
        self.problems
            .iter()
            .map(|p| p.clone().error(path.clone(), line))
            .collect()
    }
}

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
        // The base holds no format, so no type of the file breaks one.
        Ok(base.layered(&own).0)
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
    #[expect(
        clippy::needless_pass_by_value,
        reason = "a public signature of a published crate"
    )]
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
            let entry = if whole.get("patch").and_then(Spanned::as_bool) == Some(true) {
                if let Some((f, _)) = base.and_then(|b| b.entries.get(&name.value)) {
                    merged = merge_patch(&f.raw, whole);
                    &merged
                } else {
                    findings.push(fail(
                        whole.line,
                        Problem::new(
                            "TL_41",
                            format!("format `{}` patches no farther entry", name.value),
                        ),
                    ));
                    continue;
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

    /// This registry with `over` on top, and each format of this registry
    /// that no longer compiles against the types of `over`: a same-named
    /// entry of `over` replaces this one whole, and a same-named type
    /// likewise. An entry of `over` that declared a patch was merged onto
    /// the farther one when it was built. Where `over` declares a type that
    /// differs from this registry's, each entry that uses it is compiled
    /// again against it, so a type means one thing to every format a schema
    /// sees, and an entry that no longer compiles is left out, so the engine
    /// checks no line against its old compile.
    // sokf-implements: TL_34
    // sokf-implements: TL_36
    // sokf:begin format-registry
    // sokf-implements: TL_51
    pub fn layered(&self, over: &FormatRegistry) -> (FormatRegistry, Vec<BrokenFormat>) {
        // sokf:end format-registry
        let types = self.types.layered(&over.types);
        let seen = TypeRegistry::built_ins().layered(&types);
        let regexes = RegexCache::new();
        let mut entries = self.entries.clone();
        let mut broken = Vec::new();
        // sokf-implements: CORE-5_AC-11
        // Only a type this registry defines otherwise can change one of its
        // formats: a new name is one no farther template uses, and the same
        // value compiles to the same format.
        let changed: Vec<&str> = over
            .types
            .entries
            .iter()
            .filter(|(n, (def, _))| self.types.get(n).is_some_and(|d| d != def))
            .map(|(n, _)| n.as_str())
            .collect();
        if !changed.is_empty() {
            entries.retain(|name, (format, source)| {
                // An entry that `over` replaces is one that no schema sees.
                if over.entries.contains_key(name) {
                    return true;
                }
                let used = format.uses(&changed);
                if used.is_empty() {
                    return true;
                }
                match lower_format(&format.raw, &seen, &regexes) {
                    Ok(mut again) => {
                        again.patched = format.patched;
                        *format = again;
                        true
                    }
                    // sokf-implements: TL_140
                    // sokf-implements: CORE-5_AC-17
                    Err(said) => {
                        broken.push(BrokenFormat::of(name, source, &used, said));
                        false
                    }
                }
            });
        }
        // sokf-implements: CORE-5_AC-10
        for (name, entry) in &over.entries {
            entries.insert(name.clone(), entry.clone());
        }
        (FormatRegistry { entries, types }, broken)
    }

    /// This registry, the shared layer, with the pack layer on top: the
    /// entries and the types of `files`, each a formats file of the layer
    /// built with `layer` on this registry with the source
    /// `FormatSource::Pack`. The files are read in the order of their
    /// paths, whatever order the caller gives, so no entry patches another
    /// file's; where two files define one name, the entry of the first
    /// stands, and the layer keeps the clash for `PackLayer::with_host`.
    /// Each format that uses a type of the layer is compiled again against
    /// it, so a type means one thing to every format that a schema sees. A
    /// format of a file that no longer compiles is left out and reported on
    /// that file. A format of this registry that no longer compiles stays
    /// as this registry compiled it, for the host's layer to settle, and
    /// `PackLayer::with_host` reports it where the host's layer leaves it
    /// broken.
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
                if let Some(first) = origin.get(name.as_str()) {
                    clashes.push(Clash {
                        what: "format",
                        name: name.clone(),
                        first: (*first).clone(),
                        later: path.clone(),
                        line: Some(entry.0.raw.line),
                    });
                } else {
                    origin.insert(name, path);
                    packs.entries.insert(name.clone(), entry.clone());
                }
            }
            for (name, def) in &file.types.entries {
                if let Some(first) = type_origin.get(name.as_str()) {
                    clashes.push(Clash {
                        what: "type",
                        name: name.clone(),
                        first: (*first).clone(),
                        later: path.clone(),
                        line: None,
                    });
                } else {
                    type_origin.insert(name, path);
                    packs.types.entries.insert(name.clone(), def.clone());
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
        let mut refused = Vec::new();
        for (name, format) in again {
            if let Some(format) = format {
                if let Some(entry) = packs.entries.get_mut(&name) {
                    entry.0 = format;
                }
            } else {
                packs.entries.remove(&name);
                refused.push(name);
            }
        }
        // A shared format that a type of the layer breaks stays as this
        // registry compiled it, so that the host's layer can patch it, and
        // `with_host` decides it once the host's types are known. Its
        // findings, kept for a host that changes no type of the format, lie
        // on the file of the first type that it uses and that the layer
        // changes: every type of the layer has a file. The shared format of
        // a name that a refused format of a file takes stays out, because
        // that format has its finding already.
        let (mut registry, broken) = self.layered(&packs);
        let mut pending = BTreeMap::new();
        for b in broken {
            if refused.contains(&b.name) {
                continue;
            }
            let entry = self.entries[&b.name].clone();
            registry.entries.insert(b.name.clone(), entry);
            let findings = b.findings(type_origin[b.types[0].as_str()], None);
            pending.insert(b.name, findings);
        }
        PackLayer {
            registry,
            clashes,
            findings,
            shared_types: self.types.clone(),
            pending,
            type_origin: type_origin
                .into_iter()
                .map(|(name, path)| (name.to_string(), path.clone()))
                .collect(),
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
    /// The types of the shared layer, the built-ins apart.
    shared_types: TypeRegistry,
    /// Each shared format that a type of the layer breaks, which `registry`
    /// holds as the shared layer compiled it, with its findings on the file
    /// of the first such type.
    pending: BTreeMap<String, Vec<Finding>>,
    /// The first file of the layer that declares each type.
    type_origin: BTreeMap<String, RepoPath>,
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
    /// own layer. It holds each shared format that a type of the layer
    /// breaks as the shared layer compiled it, so that the host's layer can
    /// patch it; a schema compiles against the registry of `with_host`.
    // sokf-implements: CORE-5_AC-13
    // sokf:begin format-registry
    pub fn registry(&self) -> &FormatRegistry {
        // sokf:end format-registry
        &self.registry
    }

    /// The registry with `host`, the host's own layer built on `registry`
    /// from the file `path`, on top, and the findings of the pack layer and
    /// of `host`: each format of a file that does not compile against the
    /// types of the layer, each format and type that two files define and
    /// `host` does not define in full, each format of a file that no longer
    /// compiles against a type of `host`, on `path`, and each shared format
    /// that no longer compiles against the types of the registry. A patch
    /// in `host` settles no clash, because it was merged onto the entry of
    /// whichever file came first. A shared format that a type of the layer
    /// breaks is settled where `host` defines the format, patches it, or
    /// defines a type against which it compiles.
    // sokf-implements: TL_128
    // sokf-implements: CORE-5_AC-14
    // sokf:begin format-registry
    pub fn with_host(
        &self,
        host: &FormatRegistry,
        path: &RepoPath,
    ) -> (FormatRegistry, Vec<Finding>) {
        // sokf:end format-registry
        let settled = |c: &Clash| match c.what {
            "format" => host.entries.get(&c.name).is_some_and(|(f, _)| !f.patched),
            _ => host.types.entries.contains_key(&c.name),
        };
        // A shared format that a type of the layer broke is decided apart,
        // so that it compiles once, against the types of the layer under
        // `host`; `layered` decides every other format.
        let mut base = self.registry.clone();
        base.entries
            .retain(|name, _| !self.pending.contains_key(name));
        let (mut registry, broken) = base.layered(host);
        let seen = registry.types_seen();
        let regexes = RegexCache::new();
        let mut of_packs = Vec::new();
        for (name, said) in &self.pending {
            // An entry of `host` replaces the format, or patches it.
            if host.entries.contains_key(name) {
                continue;
            }
            let (format, source) = &self.registry.entries[name];
            let hosts = self.host_types(format, host);
            // sokf-implements: TL_140
            // sokf-implements: CORE-5_AC-18
            if hosts.is_empty() {
                of_packs.extend(said.iter().cloned());
                continue;
            }
            // sokf-implements: CORE-5_AC-20
            match lower_format(&format.raw, &seen, &regexes) {
                Ok(mut again) => {
                    again.patched = format.patched;
                    registry
                        .entries
                        .insert(name.clone(), (again, source.clone()));
                }
                Err(said) => of_packs.extend(self.still_broken(name, host, &hosts, said, path)),
            }
        }
        let mut findings = self.findings.clone();
        findings.extend(of_packs);
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
        // sokf-implements: TL_140
        // sokf-implements: CORE-5_AC-19
        findings.extend(broken.iter().flat_map(|b| b.findings(path, None)));
        (registry, findings)
    }

    /// Each type of `host` that `format` uses and that differs from the
    /// type of the layer, in the order of the holes.
    fn host_types<'h>(&self, format: &Format, host: &'h FormatRegistry) -> Vec<&'h str> {
        let differ: Vec<&str> = host
            .types
            .entries
            .iter()
            .filter(|(n, (def, _))| self.registry.types.get(n) != Some(def))
            .map(|(n, _)| n.as_str())
            .collect();
        format.uses(&differ)
    }

    /// The findings, for each reason of `said`, on the shared format `name`,
    /// which a type of the layer broke and which the types `hosts` of the
    /// host's layer `host`, of the file `path`, take part in and leave
    /// broken. Each type that the format uses and that differs from the
    /// shared one is named: one of `hosts` by `path`, and any other as a
    /// type of the layer. The findings lie on the file of the first type of
    /// the layer, because the layer broke the format first, or on `path`
    /// where only `hosts` take part.
    // sokf-implements: TL_140
    // sokf-implements: CORE-5_AC-18
    fn still_broken(
        &self,
        name: &str,
        host: &FormatRegistry,
        hosts: &[&str],
        said: Vec<(usize, Problem)>,
        path: &RepoPath,
    ) -> Vec<Finding> {
        let (format, source) = &self.registry.entries[name];
        let differs = |n: &str| {
            let now = if hosts.contains(&n) {
                host.types.get(n)
            } else {
                self.registry.types.get(n)
            };
            now != self.shared_types.get(n)
        };
        let all: Vec<&str> = self
            .registry
            .types
            .entries
            .keys()
            .map(String::as_str)
            .collect();
        let used: Vec<&str> = format
            .uses(&all)
            .into_iter()
            .filter(|t| differs(t))
            .collect();
        let (of_host, of_packs): (Vec<&str>, Vec<&str>) =
            used.iter().partition(|t| hosts.contains(t));
        match of_packs.first().and_then(|t| self.type_origin.get(*t)) {
            Some(file) => {
                let mut broken =
                    BrokenFormat::of_layers(name, source, &of_packs, &of_host, path, said);
                // The types in the order of the holes.
                broken.types = used.iter().map(|t| (*t).to_string()).collect();
                broken.findings(file, None)
            }
            None => BrokenFormat::of(name, source, &used, said).findings(path, None),
        }
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
