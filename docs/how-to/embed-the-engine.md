---
type: HowTo
id: how-to-embed-the-engine
title: Embed the engine in a Rust program
description: Use the tramlines-md library to compile a schema, check documents, repair them, derive a starter document, and add schema keys of your own.
order: 120
---

# Embed the engine in a Rust program

When your program owns the documents, as an editor plugin or a documentation tool does, call the engine as a library instead of running `tramlines validate`. The library compiles a schema, checks each document, applies the repairs that the frontmatter and the schema require, and derives the scaffold that a new document starts from. Your program is the host. It chooses the schema for each document, and it can add its own keys to the grammar.

## Before you begin

- A Rust project that depends on the library. Add it with `cargo add tramlines-md`. The crate re-exports everything from `tramlines-core` that a host needs.
- A schema that the command accepts, as in [Your first schema][sokf:tutorial-first-schema].
- The [library documentation on docs.rs](https://docs.rs/tramlines-md), which describes every type that this guide names.

## Steps

The following code comes from the test `crates/lib/tramlines-md/tests/it/embed.rs` of the repository, so it compiles and runs as shown.

1. Load the grammar and compile the schema. The function `load_grammar` returns the markdown grammar merged onto the core grammar. The library has both grammars built in. The function `compile` checks the schema against the grammar, and returns findings when the schema breaks the grammar. The function `check_schema` then checks the schema against its own examples and scaffolds, as the command does.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#compile -->

```rust
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
```

<!-- /sokf:include -->

2. Check a document. The function `Document::parse` reads the frontmatter and the body. The method `Schema::states` reads the lifecycle states from the frontmatter. The function `check_against` returns the findings, with the repairs and the anchors that the document needs. Each `Finding` holds a path, a line, a severity, the key of its criterion, and a message.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#check -->

```rust
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
```

<!-- /sokf:include -->

3. Repair the document, if your program writes files. The repairs rewrite a heading or a line that is bound to a frontmatter value, a renamed heading, and a setext heading. They also write, move, and remove the anchors of keys. The function `line_edits` collects them, and the function `apply` applies them in one pass.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#repair -->

```rust
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
```

<!-- /sokf:include -->

4. Derive the scaffold of a new document. [How a schema derives its scaffold][sokf:explanation-scaffolds] explains what a scaffold contains.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#derive -->

```rust
    // The scaffold a new document starts from: one for each combination
    // of lifecycle states, each with its frontmatter and its examples.
    let derived = derive(&schema, &NullHost).expect("the schema derives");
    let scaffold = &derived.scaffolds[0].document;
```

<!-- /sokf:include -->

5. Add keys of your own with a host extension. An extension names the format grammar that it extends, and it can only add to that grammar: keys, rule maps, enum values, and `doc` text. The engine keeps each key that an extension adds unchanged. Your program reads and checks the key itself.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#extend -->

```rust
    // A host extension names the format grammar it extends, and only adds
    // to it: here a schema key `owner` that the host reads itself.
    let mut grammar = load_grammar().expect("the grammar is built in");
    grammar
        .extend(
            "extension: shop\nextends: tramlines-md\nschema:\n  keys:\n    owner:\n      type: string\n      doc: The team that owns the documents of this schema.\n",
        )
        .expect("the extension only adds");
```

<!-- /sokf:include -->

A host also supplies the facts that lie outside a document. Implement the `Host` trait of the `check` module, in place of `NullHost`. A host can then own frontmatter keys, add a frontmatter format, resolve what an image or a region names, and spell markers in its own namespace. [How the grammar layers fit together][sokf:explanation-grammar-layers] explains what a host can change.

## Troubleshooting

- `TL_12` on a key that your extension adds: you compiled the schema with a grammar that lacks the extension. Compile with the grammar that `extend` changed.
- `extend` returns an error: the extension changes something that the grammar under it defines, or names another grammar in `extends`. An extension can only add to a grammar.

## See also

- [How the grammar layers fit together][sokf:explanation-grammar-layers]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [The error reference][sokf:reference-errors]

<!-- sokf:links -->

[sokf:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:reference-errors]: /docs/reference/errors.md
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
