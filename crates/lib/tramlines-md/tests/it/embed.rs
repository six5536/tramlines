//! The engine as a host program uses it, step by step: the code that the
//! how-to guide "Embed the engine in a Rust program" shows, run.

use tramlines_md::{
    Finding, Locator, RepoPath,
    check::{CheckOptions, NullHost, check_against, check_schema, example::check_formats},
    derive::derive,
    document::Document,
    edit::{AnchorEdit, apply, line_edits},
    grammar::{
        FormatRegistry, FormatSource, Grammar, RegexCache, Schema, compile, load_core_grammar,
        load_grammar,
    },
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
/// frontmatter, renamed and setext headings, labels written as their keys,
/// and the anchors of its keys.
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
        &out.label_repairs,
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

// sokf:begin packs
/// The formats that a schema sees, and the findings of the formats files:
/// `shared` is the text of the program's built-in formats, `pack_files`
/// the name of each pack with the path and the text of its formats file,
/// and `own` the text of the program's own formats file.
fn layer_formats(
    shared: &str,
    pack_files: &[(&str, &str, &str)],
    own: &str,
) -> (FormatRegistry, Vec<Finding>) {
    let grammar = load_core_grammar().expect("the core grammar is built in");
    let regexes = RegexCache::new();
    let mut findings = Vec::new();
    // The shared layer is the program's built-in formats, with the types
    // that the program reserves.
    let shared =
        FormatRegistry::from_text(shared, &[("doc-id", "[a-z][a-z0-9-]*")], &grammar, &regexes)
            .expect("the built-in formats are valid");
    // One layer from the formats file `text` at `path`, built on `base`.
    let mut layer = |text: &str, path: &RepoPath, source: FormatSource, base: &FormatRegistry| {
        let file = yaml::parse(text, 1).expect("the formats file is YAML");
        let registry = file.get("registry").expect("a `registry` map");
        let (layer, said) = FormatRegistry::layer(
            registry,
            file.get("types"),
            source,
            &grammar,
            &regexes,
            path,
            Some(base),
        );
        findings.extend(said);
        layer
    };
    // Each file of the pack layer is built on the shared layer and named
    // by its pack.
    let files: Vec<(RepoPath, FormatRegistry)> = pack_files
        .iter()
        .map(|(name, path, text)| {
            let path = RepoPath::parse(path).expect("a path");
            let file = layer(text, &path, FormatSource::Pack(name.to_string()), &shared);
            (path, file)
        })
        .collect();
    let packs = shared.with_packs(&files);
    // The program's own layer goes on the pack layer, and settles a name
    // that two packs define.
    let own_path = RepoPath::parse("/formats.yaml").expect("a path");
    let host = layer(own, &own_path, FormatSource::Repo, packs.registry());
    let (registry, said) = packs.with_host(&host, &own_path);
    findings.extend(said);
    // The examples of each formats file are checked once, on that file.
    for ((name, _, _), (path, _)) in pack_files.iter().zip(&files) {
        findings.extend(check_formats(
            &registry,
            &FormatSource::Pack(name.to_string()),
            path,
        ));
    }
    findings.extend(check_formats(&registry, &FormatSource::Repo, &own_path));
    (registry, findings)
}
// sokf:end packs

const TASKS: &str = "registry:\n  task:\n    template: 'TODO ${}'\n    example: TODO buy milk\n";

// Two packs define `task`, and one of them has a format whose example
// fails it. The files are read in the order of their paths, and the
// example is reported on its pack's file, not on each schema.
// sokf-tests: MD-6_AC-8
#[test]
fn a_host_layers_the_formats_of_its_packs() {
    let shared = "registry:\n  note:\n    template: 'Note: ${}'\n    example: 'Note: tea'\n";
    let home = format!("{TASKS}  day:\n    template: 'Day ${{:int}}'\n    example: Monday\n");
    let (registry, findings) = layer_formats(
        shared,
        &[
            ("work", "/packs/work/formats.yaml", TASKS),
            ("home", "/packs/home/formats.yaml", &home),
        ],
        "registry: {}\n",
    );
    let shown: Vec<String> = findings
        .iter()
        .map(|f| {
            format!(
                "{}:{}: [{}] {}",
                f.path,
                f.line.unwrap_or(0),
                f.criterion,
                f.message
            )
        })
        .collect();
    assert_eq!(
        shown,
        vec![
            "/packs/work/formats.yaml:3: [TL_128] format `task` is defined in both `/packs/home/formats.yaml` and `/packs/work/formats.yaml` of the pack layer; define it in full in the host's layer to choose one",
            "/packs/home/formats.yaml:6: [TL_65] the example `Monday` of format `day` fails at column 1: expected `Day `, found `Monday`",
        ]
    );
    assert_eq!(registry.entries["task"].1.label(), "pack:home");
    assert_eq!(registry.entries["note"].1.label(), "embedded");
    // A schema under the registry reports nothing of the pack's example.
    let written = yaml::parse(SCHEMA, 1).expect("the schema is YAML");
    let grammar = load_grammar().expect("the grammar is built in");
    let path = RepoPath::parse("/list.yaml").expect("a path");
    let schema = compile(
        &written,
        &grammar,
        &registry,
        &RegexCache::new(),
        &path,
        "list",
        &Dialect::default(),
    )
    .expect("the schema compiles");
    assert_eq!(check_schema(&schema, &NullHost, &path, "list"), vec![]);
}
