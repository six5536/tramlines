//! `tramlines new`: the scaffold of a schema at the states that the
//! command line names, with its id and title filled.

use serde_json::json;
use tramlines_md::{
    Locator, RepoPath, Severity, Yaml,
    check::{CheckOptions, NullHost, check_against},
    derive::{derive_one, derived_title},
    document::Document,
    edit::{apply, line_edits},
    grammar::{Schema, States},
    markdown::Dialect,
    text, yaml,
};

use crate::{Stop, cli::NewArgs, load};

/// Print the scaffold, or one JSON object with its id and the scaffold.
// sokf-implements: CLI-12_AC-1
// sokf-implements: CLI-12_AC-10
// sokf-implements: CLI-12_AC-11
pub fn new(args: &NewArgs) -> Result<u8, Stop> {
    let engine = load::engine(args.formats.as_deref())?;
    let shown = args.schema.display().to_string();
    let schema = load::compiled(&engine, &args.schema)?;
    let states = states(&schema, &args.state)?;
    let scaffold = derive_one(&schema, &states, &NullHost).map_err(|refusals| {
        let mut why = format!("{shown}: the schema derives no scaffold:");
        for r in refusals {
            why.push_str(&format!("\n  {shown}:{}: {}", r.line, r.what));
        }
        Stop::plain(why)
    })?;
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
    let title = match (&args.title, &args.slug, &args.id) {
        (Some(title), _, _) => Some(title.clone()),
        (None, Some(slug), _) if has_title => Some(derived_title(slug)),
        (None, None, Some(id)) if has_title => Some(derived_title(id)),
        _ => None,
    };
    let mut document = strip_only(&scaffold);
    if args.id.is_some() || args.slug.is_some() {
        document = fill(&document, "id", id.as_deref().unwrap_or_default());
    }
    if let Some(title) = &title {
        document = fill(&document, "title", title);
    }
    let document = bind(&schema, &states, &load::schema_id(&args.schema), &document)?;
    if args.json {
        let mut one = json!({});
        if let Some(id) = id.or_else(|| value_of(&document, "id")) {
            one["id"] = json!(id);
        }
        one["document"] = json!(document);
        println!("{one}");
    } else {
        print!("{document}");
    }
    Ok(0)
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

/// The frontmatter line of `key` in `document` set to `value`; the line is
/// added before the closing fence when the scaffold has none.
fn fill(document: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = document.lines().collect();
    let close = (lines.first() == Some(&"---"))
        .then(|| {
            lines
                .iter()
                .skip(1)
                .position(|l| *l == "---")
                .map(|i| i + 1)
        })
        .flatten();
    let written = format!("{key}: {}", scalar(value));
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    match close {
        Some(close) => {
            let prefix = format!("{key}:");
            match (1..close).find(|&i| {
                lines[i]
                    .strip_prefix(&prefix)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
            }) {
                Some(i) => out[i] = written,
                None => out.insert(close, written),
            }
        }
        None => {
            out.insert(0, "---".to_string());
            out.insert(1, written);
            out.insert(2, "---".to_string());
            out.insert(3, String::new());
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// A scalar as YAML: plain where it reads back as the same string, so
/// that `2026` or `true` stays text, else double-quoted.
fn scalar(value: &str) -> String {
    let plain = !value.is_empty()
        && !value.contains('#')
        && yaml::parse(&format!("k: {value}\n"), 1)
            .ok()
            .and_then(|y| y.get("k").map(|v| v.value.clone()))
            == Some(Yaml::Str(value.to_string()));
    if plain {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// The value of the frontmatter key `key` in `document`.
fn value_of(document: &str, key: &str) -> Option<String> {
    let parsed = Document::parse(RepoPath::root(), document, &Dialect::default());
    parsed.frontmatter?.get(key).and_then(|v| v.scalar_text())
}

/// The filled scaffold with the headings and units that its frontmatter
/// binds rewritten, checked against the schema as a scaffold. A value
/// that the schema refuses stops the run.
// sokf-implements: CLI-12_AC-8
// sokf-implements: CLI-12_AC-12
fn bind(schema: &Schema, states: &States, id: &str, document: &str) -> Result<String, Stop> {
    let check = |text: &str| {
        let parsed = Document::parse(RepoPath::root(), text, &Dialect::default());
        let out = check_against(
            parsed.view(),
            schema,
            states,
            &NullHost,
            CheckOptions { scaffold: true },
            Locator::schema(id),
        );
        (parsed, out)
    };
    let (parsed, out) = check(document);
    let edits = line_edits(&parsed.text, &parsed.segments, &out.repairs, None);
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
    Ok(bound)
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

    #[test]
    fn fill_replaces_a_key() {
        assert_eq!(
            fill(
                "---\nid: x-1\ntitle: Old\n---\n\n# Old\n",
                "title",
                "New words"
            ),
            "---\nid: x-1\ntitle: New words\n---\n\n# Old\n"
        );
    }

    #[test]
    fn fill_adds_a_key_the_scaffold_lacks() {
        assert_eq!(
            fill("---\nid: x-1\n---\n\n# Old\n", "title", "A: b"),
            "---\nid: x-1\ntitle: \"A: b\"\n---\n\n# Old\n"
        );
        assert_eq!(fill("# Old\n", "id", "x-1"), "---\nid: x-1\n---\n\n# Old\n");
    }

    #[test]
    fn a_value_that_yaml_reads_otherwise_is_quoted() {
        assert_eq!(scalar("Tabs in lists"), "Tabs in lists");
        assert_eq!(scalar("2026"), "\"2026\"");
        assert_eq!(scalar("true"), "\"true\"");
        assert_eq!(scalar("null"), "\"null\"");
        assert_eq!(scalar("A: b"), "\"A: b\"");
        assert_eq!(scalar("C# tips"), "\"C# tips\"");
    }
}
