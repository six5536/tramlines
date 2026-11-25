//! `tramlines new`: the skeleton of a new document at the states that the
//! command line names, with its id and title, after the check of the
//! scaffold for them, and the guide of its parts on stderr.

use std::{collections::BTreeMap, fmt::Write as _};

use serde_json::json;
use tramlines_md::{
    Locator, RepoPath, Severity,
    check::{CheckOptions, check_against},
    derive::{derive_for, derive_one, derived_title, guide_for, skeleton},
    document::Document,
    edit::{apply, line_edits},
    grammar::{Schema, States},
    markdown::Dialect,
    text,
};

use crate::{
    Stop,
    cli::NewArgs,
    guide, load,
    texts::{Texts, Words},
};

/// Print the skeleton of a new document, with its guide on stderr, or one
/// JSON object with its id, the skeleton and the guide.
// sokf-implements: CLI-12_AC-1
// sokf-implements: CLI-12_AC-10
// sokf-implements: CLI-12_AC-11
// sokf-implements: CLI-12_AC-14
// sokf-implements: CLI-12_AC-15
pub fn new(args: &NewArgs) -> Result<u8, Stop> {
    let engine = load::engine(args.formats.as_deref())?;
    let shown = args.schema.display().to_string();
    let schema = load::compiled(&engine, &args.schema)?;
    let texts = Texts::load(args.texts.as_deref())?;
    let host = Words(&texts);
    let states = states(&schema, &args.state)?;
    let id = id(&schema, args, &states)?;
    // sokf-implements: CLI-12_AC-7
    // sokf-implements: CLI-12_AC-13
    let has_title = declares(&schema, "title");
    if args.title.is_some() && !has_title {
        return Err(Stop::plain(format!(
            "{shown}: the schema declares no frontmatter key `title`"
        )));
    }
    // The title is the one `--title` names, else the words of the slug, or
    // of the id, where the schema declares a title.
    // An empty `--title` names no title, so the title comes from the slug or
    // the id, as without the option.
    let named = args.title.as_ref().filter(|t| !t.trim().is_empty());
    let title = match (named, &args.slug, &args.id) {
        (Some(title), _, _) => Some(title.clone()),
        (None, Some(slug), _) if has_title => Some(derived_title(slug)),
        (None, None, Some(id)) if has_title => Some(derived_title(id)),
        _ => None,
    };
    let mut values = BTreeMap::new();
    if let Some(title) = named {
        fixed(&schema, "title", title)?;
    }
    for (key, value) in [("id", &id), ("title", &title)] {
        if let Some(value) = value {
            values.insert(key.to_string(), value.clone());
        }
    }
    let refused = |refusals: Vec<tramlines_md::derive::Refusal>| {
        let mut why = format!("{shown}: the schema derives no scaffold:");
        for r in refusals {
            let _ = write!(why, "\n  {shown}:{}: {}", r.line, r.what);
        }
        Stop::plain(why)
    };
    // The scaffold of the schema's own values tells a fault of the schema
    // from a fault of the id or the title.
    derive_one(&schema, &states, &host).map_err(refused)?;
    // The scaffold with the id and the title is the proof that the schema,
    // the id and the title fit together, so that the skeleton is a document
    // that its writer can bring to pass.
    // sokf-implements: CLI-12_AC-12
    let scaffold = derive_for(&schema, &states, &host, &values).map_err(|refusals| {
        let first = refusals.first();
        Stop::plain(first.map_or_else(String::new, |r| {
            format!(
                "the new document fails the schema at line {} of the schema: [{}] {}",
                r.line, r.criterion, r.what
            )
        }))
    })?;
    bind(
        &schema,
        &states,
        &host,
        &load::schema_id(&args.schema),
        &strip_only(&scaffold),
    )?;
    // sokf-implements: CLI-12_AC-8
    // sokf-implements: CLI-12_AC-9
    let document = skeleton(&schema, &states, &host, &values).map_err(refused)?;
    let guide = guide_for(&schema, &states, &host, &values);
    if args.json {
        let mut one = json!({});
        // A skeleton holds an empty `id` until the command line names one.
        if let Some(id) = id
            .or_else(|| value_of(&document, "id"))
            .filter(|id| !id.is_empty())
        {
            one["id"] = json!(id);
        }
        one["document"] = json!(document);
        one["guide"] = json!(guide);
        println!("{one}");
    } else {
        print!("{document}");
        eprint!("{}", guide::text(&guide, &document));
    }
    Ok(0)
}

/// Refuse a value that `--title` names for `key` where the schema
/// fixes the key by `const` to another value, which the new document can't
/// hold.
// sokf-implements: CLI-12_AC-16
fn fixed(schema: &Schema, key: &str, value: &str) -> Result<(), Stop> {
    let fixed = schema.core.constraint(key).and_then(|c| c.const_.as_ref());
    match fixed {
        Some(fixed) if fixed != value => Err(Stop::plain(format!(
            "`{value}` is not `{fixed}`, the `{key}` that the schema fixes"
        ))),
        _ => Ok(()),
    }
}

/// One state of each dimension: the state that `--state` names, else the
/// dimension's first.
// sokf-implements: CLI-12_AC-2
// sokf-implements: CLI-12_AC-3
fn states(schema: &Schema, named: &[(String, String)]) -> Result<States, Stop> {
    let mut states = States::default();
    for (key, state) in named {
        let Some(dim) = schema.core.dimension(key) else {
            let lifecycle = &schema.core.lifecycle;
            return Err(Stop::plain(if lifecycle.is_empty() {
                format!("`{key}` is no lifecycle key: the schema declares no lifecycle")
            } else {
                format!(
                    "`{key}` is no lifecycle key; the keys are {}",
                    text::code_list(lifecycle.iter().map(|d| &d.key), ", ")
                )
            }));
        };
        if !dim.states.contains(state) {
            return Err(Stop::plain(format!(
                "`{state}` is no state of `{key}`; the states are {}",
                text::code_list(&dim.states, ", ")
            )));
        }
        states.0.insert(key.clone(), vec![state.clone()]);
    }
    for dim in &schema.core.lifecycle {
        if !states.0.contains_key(&dim.key)
            && let Some(first) = dim.states.first()
        {
            states.0.insert(dim.key.clone(), vec![first.clone()]);
        }
    }
    Ok(states)
}

/// Whether the schema declares the frontmatter key `key`.
fn declares(schema: &Schema, key: &str) -> bool {
    schema.core.frontmatter.iter().any(|(k, _)| k == key)
}

/// The id that `--id` names, or that `--slug` fills into the id template.
/// `None` where the command line names neither.
// sokf-implements: CLI-12_AC-4
// sokf-implements: CLI-12_AC-5
// sokf-implements: CLI-12_AC-6
fn id(schema: &Schema, args: &NewArgs, states: &States) -> Result<Option<String>, Stop> {
    if (args.id.is_some() || args.slug.is_some()) && !declares(schema, "id") {
        return Err(Stop::plain(
            "the schema declares no frontmatter key `id`".into(),
        ));
    }
    let template = schema.core.id_template();
    if let Some(id) = &args.id {
        if let Some(t) = template
            && !t.matches(id)
        {
            return Err(Stop::plain(format!(
                "`{id}` does not match the id template `{}`",
                t.regex()
            )));
        }
        return Ok(Some(id.clone()));
    }
    let Some(slug) = &args.slug else {
        return Ok(None);
    };
    let Some(t) = template.filter(|t| t.has_slug()) else {
        return Err(Stop::plain(
            "the schema's `id` has no template with a `slug` hole; name the id with `--id`".into(),
        ));
    };
    if t.seq_width().is_some() {
        return Err(Stop::plain(
            "the id template has a `seq` hole, which needs the documents around it; name the whole id with `--id`".into(),
        ));
    }
    t.render_with(1, Some(slug), states)
        .map(Some)
        .map_err(Stop::plain)
}

/// The scaffold with its `only` markers stripped. It is derived at one
/// state of each dimension, so every part that a marker gates is one that
/// the states admit, and a document carries no marker.
// sokf-implements: CLI-12_AC-9
fn strip_only(scaffold: &str) -> String {
    let dialect = Dialect::default();
    let parsed = Document::parse(RepoPath::root(), scaffold, &dialect);
    let mut out = Vec::new();
    for (i, line) in scaffold.lines().enumerate() {
        if !dialect.is_only_marker(line) {
            out.push(line.to_string());
        } else if i + 1 < parsed.body_line {
            let cut = line.rfind('#').unwrap_or(line.len());
            out.push(line[..cut].trim_end().to_string());
        } else if parsed.body.in_code(i + 1) || !line.trim_start().starts_with("<!--") {
            out.push(line.to_string());
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// The value of the frontmatter key `key` in `document`.
fn value_of(document: &str, key: &str) -> Option<String> {
    let parsed = Document::parse(RepoPath::root(), document, &Dialect::default());
    parsed
        .frontmatter?
        .get(key)
        .and_then(tramlines_md::Spanned::scalar_text)
}

/// Check the scaffold for the id and the title against the schema as a
/// scaffold, after the repairs of the headings and units that its
/// frontmatter binds. A value that the schema refuses stops the run,
/// because no document with it passes.
// sokf-implements: CLI-12_AC-8
// sokf-implements: CLI-12_AC-12
fn bind(
    schema: &Schema,
    states: &States,
    host: &Words<'_>,
    id: &str,
    document: &str,
) -> Result<(), Stop> {
    let check = |text: &str| {
        let parsed = Document::parse(RepoPath::root(), text, &Dialect::default());
        let out = check_against(
            parsed.view(),
            schema,
            states,
            host,
            CheckOptions { scaffold: true },
            Locator::schema(id),
        );
        (parsed, out)
    };
    let (parsed, out) = check(document);
    let edits = line_edits(
        &parsed.text,
        &parsed.segments,
        &out.repairs,
        &out.label_repairs,
        None,
    );
    let bound = if edits.is_empty() {
        document.to_string()
    } else {
        apply(&parsed.text, &edits).map_err(|e| Stop::plain(e.to_string()))?
    };
    let (_, out) = check(&bound);
    if let Some(f) = out.findings.iter().find(|f| f.severity == Severity::Error) {
        return Err(Stop::plain(format!(
            "the new document fails the schema at line {}: [{}] {}",
            f.line.unwrap_or(0),
            f.criterion,
            f.message
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_only_markers_go_and_their_parts_stay() {
        let doc = "---\nid: x-1\ndone: '2026-09-30' # tramlines:only state=closed\n---\n\n<!-- tramlines:only state=closed -->\n## Outcome\n\n```markdown\n<!-- tramlines:only state=closed -->\n```\n";
        assert_eq!(
            strip_only(doc),
            "---\nid: x-1\ndone: '2026-09-30'\n---\n\n## Outcome\n\n```markdown\n<!-- tramlines:only state=closed -->\n```\n"
        );
    }
}
