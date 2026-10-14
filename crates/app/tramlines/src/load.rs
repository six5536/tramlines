//! What the commands read: files, standard input, the format of each
//! document, the formats file and the schema.

use std::{io::Read, path::Path};

use tramlines_md::{
    Finding, RepoPath,
    grammar::{FormatRegistry, FormatSource, Grammar, RegexCache, Schema, compile, load_grammar},
    markdown::Dialect,
    text, yaml,
};

use crate::{Stop, cli::FORMATS};

/// Whether `path` names standard input.
pub fn is_stdin(path: &Path) -> bool {
    path.as_os_str() == "-"
}

/// The UTF-8 byte order mark, which an editor on Windows can write at the
/// start of a file.
pub const BOM: char = '\u{feff}';

/// The text of `path`, without a byte order mark.
// sokf-implements: CLI-8_AC-3
pub fn read(path: &Path) -> Result<String, Stop> {
    let text = read_raw(path)?;
    Ok(match text.strip_prefix(BOM) {
        Some(rest) => rest.to_string(),
        None => text,
    })
}

// sokf-implements: CLI-5_AC-2
fn read_raw(path: &Path) -> Result<String, Stop> {
    std::fs::read_to_string(path).map_err(|e| Stop::plain(format!("{}: {e}", path.display())))
}

/// The text of a document as it is written, byte order mark and all: a
/// file, or standard input for `-`.
// sokf-implements: CLI-11_AC-1
pub fn read_doc(path: &Path) -> Result<String, Stop> {
    if !is_stdin(path) {
        return read_raw(path);
    }
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| Stop::plain(format!("-: {e}")))?;
    Ok(text)
}

/// The format of `doc`: the one `--format` names, else the one its
/// extension matches. Standard input has no extension.
// sokf-implements: CLI-3_AC-1
// sokf-implements: TL_88
// sokf-implements: TL_89
// sokf-implements: TL_90
// sokf-implements: TL_91
pub fn format_of(doc: &Path, named: Option<&str>) -> Result<&'static str, Stop> {
    let known = || text::code_list(FORMATS.iter().map(|(f, _)| f), ", ");
    if let Some(name) = named {
        return FORMATS
            .iter()
            .find(|(f, _)| *f == name)
            .map(|(f, _)| *f)
            // sokf-implements: CLI-3_AC-6
            // sokf-implements: CLI-3_AC-7
            .ok_or_else(|| {
                Stop::on(
                    "TL_91",
                    format!("no format `{name}`; the formats are {}", known()),
                )
            });
    }
    // sokf-implements: CLI-11_AC-2
    if is_stdin(doc) {
        return Err(Stop::on(
            "TL_90",
            format!(
                "-: standard input has no extension; name its format with `--format` ({})",
                known()
            ),
        ));
    }
    let ext = doc.extension().and_then(|e| e.to_str()).unwrap_or("");
    FORMATS
        .iter()
        .find(|(_, exts)| exts.contains(&ext))
        .map(|(f, _)| *f)
        // sokf-implements: CLI-3_AC-4
        // sokf-implements: CLI-3_AC-5
        .ok_or_else(|| {
            Stop::on(
                "TL_90",
                format!(
                    "{}: no format matches the extension `{ext}`; name one with `--format` ({})",
                    doc.display(),
                    known()
                ),
            )
        })
}

/// The engine's state for one run: the grammar, the regex cache and the
/// layer of the formats file.
pub struct Engine {
    /// The markdown grammar on the core grammar.
    pub grammar: Grammar,
    /// The compiled regular expressions.
    pub regexes: RegexCache,
    /// The formats file, or an empty registry.
    pub registry: FormatRegistry,
}

/// The engine, with the formats file that `formats` names.
// sokf-implements: CLI-2_AC-1
// sokf-implements: TL_86
// sokf-implements: TL_87
pub fn engine(formats: Option<&Path>) -> Result<Engine, Stop> {
    let grammar = load_grammar().map_err(|e| Stop::plain(e.to_string()))?;
    let regexes = RegexCache::new();
    let registry = match formats {
        Some(path) => formats_file(path, &grammar, &regexes)?,
        // sokf-implements: CLI-2_AC-4
        None => FormatRegistry::default(),
    };
    Ok(Engine {
        grammar,
        regexes,
        registry,
    })
}

/// The formats file at `path`, its entries labelled with the path.
// sokf-implements: CLI-2_AC-2
fn formats_file(
    path: &Path,
    grammar: &Grammar,
    regexes: &RegexCache,
) -> Result<FormatRegistry, Stop> {
    // sokf-implements: CLI-2_AC-3
    let fail = |why: String| Stop::on("TL_87", format!("{}: {why}", path.display()));
    // A YAML anchor, alias or tag breaks TL_14, as it does in a schema.
    // sokf-implements: TL_14
    let root = yaml::parse(&read(path)?, 1).map_err(|e| {
        if e.is_refused() {
            Stop::on("TL_14", format!("{}: {e}", path.display()))
        } else {
            fail(e.to_string())
        }
    })?;
    let registry = root
        .get("registry")
        .ok_or_else(|| fail("no registry".into()))?;
    FormatRegistry::from_yaml_at(
        registry,
        root.get("types"),
        FormatSource::File(path.display().to_string()),
        grammar,
        regexes,
        &RepoPath::root(),
        None,
    )
    // A fault in the registry is the fault it would be in a schema's own
    // `formats`, so it keeps that criterion's key.
    // sokf-implements: TL_124
    // sokf-implements: CLI-2_AC-5
    .map_err(|findings| {
        Stop::each(findings.into_iter().map(|f| {
            let at = f.line.map_or_else(String::new, |l| format!(":{l}"));
            (
                f.criterion,
                format!("{}{at}: {}", path.display(), f.message),
            )
        }))
    })
}

/// The schema at `path`, which a command other than `validate` needs whole:
/// one that does not compile stops the run, and the stop sends the user
/// to `validate`.
pub fn compiled(engine: &Engine, path: &Path) -> Result<Schema, Stop> {
    schema(engine, path)?.map_err(|_| {
        let shown = path.display();
        Stop::plain(format!(
            "{shown}: the schema has errors; run `tramlines validate --schema {shown}`"
        ))
    })
}

/// The id of the schema at `path`: the stem of its filename.
// sokf-implements: CLI-1_AC-3
pub fn schema_id(path: &Path) -> String {
    path.file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
}

/// The schema at `path`, compiled over the formats file, or the findings
/// that keep it from compiling. A schema that does not parse as YAML
/// stops the run.
// sokf-implements: TL_14
pub fn schema(engine: &Engine, path: &Path) -> Result<Result<Schema, Vec<Finding>>, Stop> {
    // A YAML anchor, alias or tag breaks TL_14; any other fault, TL_13.
    // sokf-implements: CLI-5_AC-3
    // sokf-implements: CLI-6_AC-1
    let written = yaml::parse(&read(path)?, 1).map_err(|e| {
        let criterion = if e.is_refused() { "TL_14" } else { "TL_13" };
        Stop::on(criterion, format!("{}: {e}", path.display()))
    })?;
    Ok(compile(
        &written,
        &engine.grammar,
        &engine.registry,
        &engine.regexes,
        &RepoPath::root(),
        &schema_id(path),
        &Dialect::default(),
    ))
}
