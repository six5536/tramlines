//! The engine as a host program uses it, step by step: the code that the
//! how-to guide "Embed the engine in a Rust program" shows, run.

use tramlines_md::{
    Finding, Locator, RepoPath,
    check::{CheckOptions, NullHost, check_against, check_schema},
    derive::derive,
    document::Document,
    edit::{AnchorEdit, apply, line_edits},
    grammar::{FormatRegistry, Grammar, RegexCache, Schema, compile, load_grammar},
    markdown::Dialect,
    yaml,
};

// sokf:begin compile
/// Compile a schema, written in YAML, against the markdown grammar. The
/// findings, when there are any, are on the schema.
fn compile_schema(grammar: &Grammar, text: &str) -> Result<Schema, Vec<Finding>> {
    let written = yaml::parse(text, 1).expect("the schema is YAML");
    let schema = compile(
        &written,
        grammar,
        &FormatRegistry::default(),
        &RegexCache::new(),
        &RepoPath::parse("/list.yaml").expect("a path"),
        "list",
        &Dialect::default(),
    )?;
    // A schema is checked against its own examples and scaffolds too.
    let findings = check_schema(&schema, &NullHost, &RepoPath::root(), "list");
    if findings.is_empty() {
        Ok(schema)
    } else {
        Err(findings)
    }
}
// sokf:end compile

// sokf:begin check
/// Check one document against a compiled schema, at the states of its
/// frontmatter, and return each finding as the command prints it.
fn check_document(schema: &Schema, path: &str, text: &str) -> Vec<String> {
    let dialect = Dialect::default();
    let document = Document::parse(RepoPath::parse(path).expect("a path"), text, &dialect);
    let states = schema.states(document.frontmatter.as_ref());
    let out = check_against(
        document.view(),
        schema,
        &states,
        &NullHost,
        CheckOptions::default(),
        Locator::schema("list"),
    );
    document
        .problem
        .iter()
        .chain(&out.findings)
        .map(|f| {
            format!(
                "{}:{}: [{}] {}",
                f.path,
                f.line.unwrap_or(0),
                f.criterion,
                f.message
            )
        })
        .collect()
}
// sokf:end check

// sokf:begin repair
/// The document with every repair of its check applied: headings bound to
/// frontmatter, renamed and setext headings, and the anchors of its keys.
fn repair(schema: &Schema, text: &str) -> String {
    let dialect = Dialect::default();
    let document = Document::parse(RepoPath::root(), text, &dialect);
    let states = schema.states(document.frontmatter.as_ref());
    let out = check_against(
        document.view(),
        schema,
        &states,
        &NullHost,
        CheckOptions::default(),
        Locator::schema("list"),
    );
    let anchors = AnchorEdit {
        body: &document.body,
        placements: &out.anchors,
        text: &document.text,
        renames: &[],
    };
    let edits = line_edits(
        &document.text,
        &document.segments,
        &out.repairs,
        Some(&anchors),
    );
    apply(&document.text, &edits).expect("the edits of one check never clash")
}
// sokf:end repair

const SCHEMA: &str = "line-limit: 50\nfrontmatter:\n  title:\n    example: Groceries\nsections:\n  - heading: { from: title }\n    level: 1\n    blocks:\n      - kind: bullet-list\n        each: { template: \"${item:word} x${n:int}\", example: Milk x2 }\n";

#[test]
fn a_host_compiles_checks_repairs_and_derives() {
    let grammar = load_grammar().expect("the grammar is built in");
    let schema = compile_schema(&grammar, SCHEMA).expect("the schema compiles");

    let findings = check_document(
        &schema,
        "/shopping.md",
        "---\ntitle: Groceries\n---\n\n# Shopping\n\n- Milk x2\n- Bread\n",
    );
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings[0].starts_with("/shopping.md:5: [TLMD_165] "));
    assert!(findings[1].starts_with("/shopping.md:8: [TL_74] "));

    // The heading follows the frontmatter; the list item is the writer's.
    let fixed = repair(
        &schema,
        "---\ntitle: Groceries\n---\n\n# Shopping\n\n- Milk x2\n",
    );
    assert_eq!(
        fixed,
        "---\ntitle: Groceries\n---\n\n# Groceries\n\n- Milk x2\n"
    );

    // sokf:begin derive
    // The scaffold a new document starts from: one for each combination
    // of lifecycle states, each with its frontmatter and its examples.
    let derived = derive(&schema, &NullHost).expect("the schema derives");
    let scaffold = &derived.scaffolds[0].document;
    // sokf:end derive
    assert!(scaffold.contains("title: Groceries"), "{scaffold}");
    assert!(scaffold.contains("- Milk x2"), "{scaffold}");
}

#[test]
fn a_host_extension_adds_a_key() {
    // sokf:begin extend
    // A host extension names the format grammar it extends, and only adds
    // to it: here a schema key `owner` that the host reads itself.
    let mut grammar = load_grammar().expect("the grammar is built in");
    grammar
        .extend(
            "extension: shop\nextends: tramlines-md\nschema:\n  keys:\n    owner:\n      type: string\n      doc: The team that owns the documents of this schema.\n",
        )
        .expect("the extension only adds");
    // sokf:end extend
    let with_owner = format!("owner: groceries-team\n{SCHEMA}");
    assert!(compile_schema(&grammar, &with_owner).is_ok());
    let plain = load_grammar().expect("the grammar is built in");
    let refused =
        compile_schema(&plain, &with_owner).expect_err("no `owner` without the extension");
    assert!(
        refused.iter().any(|f| f.criterion == "TL_12"),
        "{refused:#?}"
    );
}
