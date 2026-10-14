//! `tramlines validate`: the schema against its grammar and its own
//! examples, then each document against the schema, repaired first with
//! `--fix`.

use std::path::{Path, PathBuf};

use tramlines_md::{
    Finding, Locator, RepoPath,
    check::{self, CheckOutput, NullHost, example},
    document::Document,
    grammar::{FormatSource, Schema},
    markdown::Dialect,
};

use crate::{
    Stop,
    cli::ValidateArgs,
    load::{self, is_stdin},
    report::{self, Report},
};

/// Check the schema and every document, print the report, and give the
/// exit code.
// sokf-implements: CLI-1_AC-1
// sokf-implements: TL_85
// sokf-implements: TL_93
pub fn validate(args: &ValidateArgs) -> Result<u8, Stop> {
    let piped = args.docs.iter().filter(|d| is_stdin(d)).count();
    // sokf-implements: CLI-11_AC-3
    if piped > 1 {
        return Err(Stop::plain(
            "`-` names standard input, which the command reads once".into(),
        ));
    }
    // sokf-implements: CLI-11_AC-5
    if args.fix && piped == 1 && (args.json || args.github) {
        return Err(Stop::plain(
            "`--fix` with `-` prints the repaired document on stdout, so it takes no `--json` and no `--github`".into(),
        ));
    }
    // A query that does not compile stops the run before `--fix` writes a file.
    let query = args.query.as_deref().map(report::compile).transpose()?;
    // With `--fix`, a document on standard input goes back to stdout, and
    // the report to stderr, whatever the run finds.
    let pipe = args.fix && piped == 1;
    // A document named twice is one document, checked and repaired once.
    // sokf-implements: CLI-1_AC-13
    let mut docs: Vec<&PathBuf> = Vec::new();
    for doc in &args.docs {
        if !docs.contains(&doc) {
            docs.push(doc);
        }
    }
    // Every document has a format before any is read.
    // sokf-implements: CLI-3_AC-3
    for doc in &docs {
        load::format_of(doc, args.format.as_deref())?;
    }
    let engine = load::engine(args.formats.as_deref())?;
    let id = load::schema_id(&args.schema);
    // sokf-implements: CLI-1_AC-7
    // sokf-implements: CLI-4_AC-2
    let shown = |p: &Path| p.display().to_string();
    let mut report = Report::default();
    // The formats file holds its own examples, so their faults sit on it.
    // sokf-implements: CLI-2_AC-6
    if let Some(path) = args.formats.as_deref() {
        let source = FormatSource::File(path.display().to_string());
        report.add(
            &shown(path),
            example::check_formats(&engine.registry, &source, &RepoPath::root()),
        );
    }
    // The repaired document on standard input, which goes to stdout.
    let mut repaired_stdin = None;
    // sokf-implements: CLI-1_AC-4
    match load::schema(&engine, &args.schema)? {
        // sokf-implements: CLI-1_AC-5
        Err(findings) => {
            report.add(&shown(&args.schema), findings);
            if pipe {
                repaired_stdin = Some(load::read_doc(Path::new("-"))?);
            }
        }
        Ok(schema) => {
            report.add(
                &shown(&args.schema),
                // sokf-implements: CLI-1_AC-6
                check::check_schema(&schema, &NullHost, &RepoPath::root(), &id),
            );
            // Every document is read before any is rewritten, so a file
            // that does not read stops the run with no file changed.
            // sokf-implements: CLI-9_AC-5
            let texts = docs
                .iter()
                .map(|d| load::read_doc(d))
                .collect::<Result<Vec<_>, _>>()?;
            // sokf-implements: CLI-1_AC-11
            for (doc, text) in docs.into_iter().zip(texts) {
                // The engine reads a document without its byte order mark,
                // and a repaired document keeps it.
                // sokf-implements: CLI-8_AC-3
                // sokf-implements: CLI-8_AC-4
                let body = text.strip_prefix(load::BOM).unwrap_or(&text);
                let bom = &text[..text.len() - body.len()];
                let (body, findings) = if args.fix {
                    fix(&schema, &id, doc, bom, body, &mut report)?
                } else {
                    (body.to_string(), check(&schema, &id, body).1.findings)
                };
                if pipe && is_stdin(doc) {
                    repaired_stdin = Some(format!("{bom}{body}"));
                }
                report.add(&shown(doc), findings);
            }
        }
    }
    report.finish();
    // sokf-implements: CLI-5_AC-1
    let code = u8::from(report.has_errors());
    // sokf-implements: CLI-11_AC-4
    if pipe {
        print!("{}", repaired_stdin.unwrap_or_default());
        eprint!("{}", report.text(args.warnings));
    // sokf-implements: CLI-10_AC-3
    } else if let Some(expr) = &query {
        print!("{}", report::query(expr, &report.json())?);
    } else if args.json {
        println!("{}", report.json());
    } else if args.github {
        print!("{}", report.github(args.warnings));
    } else {
        print!("{}", report.text(args.warnings));
    }
    Ok(code)
}

/// The document as parsed, and its check against the schema.
fn check(schema: &Schema, id: &str, text: &str) -> (Document, CheckOutput) {
    // sokf-implements: CLI-1_AC-2
    // sokf-implements: CLI-6_AC-2
    let parsed = Document::parse(RepoPath::root(), text, &Dialect::default());
    let out = parsed.check(schema, &NullHost, Locator::schema(id));
    (parsed, out)
}

/// The document repaired, and the findings that remain. A file whose
/// text changes is written back, with its own line endings and its byte
/// order mark, `bom`; standard input is left to the caller.
// sokf-implements: CLI-9_AC-1
// sokf-implements: CLI-9_AC-2
fn fix(
    schema: &Schema,
    id: &str,
    doc: &Path,
    bom: &str,
    text: &str,
    report: &mut Report,
) -> Result<(String, Vec<Finding>), Stop> {
    let (parsed, out) = check(schema, id, text);
    // sokf-implements: CLI-9_AC-6
    if parsed.problem.is_some() {
        return Ok((text.to_string(), out.findings));
    }
    // sokf-implements: CLI-9_AC-3
    let Some(mut repaired) = parsed
        .repair(&out)
        .map_err(|e| Stop::plain(e.to_string()))?
    else {
        return Ok((text.to_string(), out.findings));
    };
    // sokf-implements: CLI-9_AC-4
    if text.contains("\r\n") {
        repaired = repaired.replace('\n', "\r\n");
    }
    if !is_stdin(doc) {
        std::fs::write(doc, format!("{bom}{repaired}"))
            .map_err(|e| Stop::plain(format!("{}: {e}", doc.display())))?;
        report.rewrote(&doc.display().to_string());
    }
    let remaining = check(schema, id, &repaired).1.findings;
    Ok((repaired, remaining))
}
