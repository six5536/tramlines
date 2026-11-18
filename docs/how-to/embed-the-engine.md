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

3. Repair the document, if your program writes files. The repairs rewrite a heading or a line that is bound to a frontmatter value, a renamed heading, and a setext heading. They also write, move, and remove the anchors of keys, and write the host's words in place of a label written as its key. The function `line_edits` collects them, and the function `apply` applies them in one pass.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#repair -->

```rust
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

   To word the count of each limit in a scaffold, implement `Host::limit_lead`. It takes a `LimitFacts`, which holds the kind, the count, and the subject of a limit, and the `language` that the schema declares, as written. It returns the words that the guidance line holds before the limit's `when-at-limit`, in that language, or `None` for no words. The subject is a `LimitSubject`: the kind of the block for a block's own limit, the key for a frontmatter key's limit, and for a rule's limit `LimitSubject::Rule`, which holds the kind of the rule's block, whether the rule counts units or lines (`Counts`), and the format that it binds or else its `rule-id` (`RuleName`), where it has either. In a compiled schema, every rule with a limit has one of the two, as [TLMD_235][sokf:errors-units#tlmd_235] requires. Write the count, then a sentence that forbids leaving out or merging content to stay within the count, then a label that words the key `when-at-limit` in the schema's language, such as `When at the limit:`. From a rule that binds the format `node`, counts lines, and has a hard count of 9, an English host writes ``At most 9 `node` lines. Never leave out or merge content to stay within the count. When at the limit:``, and from a block's own soft count of 1 in a prose block, `Usually up to 1 paragraph. Never leave out or merge content to stay within the count. When at the limit:`. A hard count of 0 bans what the rule counts, so word it as a ban rather than as a count. Compare the language ignoring case, because `EN` and `en` name one language, and fall back from a tag such as `en-GB` to `en` where you have no words for the region. The engine adds no words of its own. Where your host returns `None` or blank words, such as for a schema in a language that it has no words for, the line holds the limit in the syntax of the schema, such as `` `hard: 9`, `when-at-limit`: ``, before its text.

   A finding on a hard limit holds none of the host's words: it quotes the limit's `when-at-limit` after the engine's English words and `When at the limit:`, before the place of the rule.

   Where a schema declares `labels`, implement `Host::label_words` to give the words of each label in the schema's language. It takes a `LabelFacts`, which holds the key of the label and the `language` that the schema declares, and returns the words that a document writes between the label's brackets, such as `Mensch` for the key `human` in German, or `None` for no words. The engine asks for them when it checks a document, when it checks a schema, when it derives a scaffold, and when it returns the guide, so pass the same host to each. A document accepts only the words that your host gives, and a label's key where your host gives none, blank words, or words that hold a square bracket or begin with `^`. The repair of a check, in `CheckOutput::label_repairs`, writes the words in place of a label written as its key: pass it to `line_edits`, as the preceding code does. To name the label that a new document's units carry, such as `agent` for a document that an agent writes, implement `Host::scaffold_label`, which takes the keys of the schema's labels and returns one of them. Where your host names none, each unit takes the label of its example, else the first label of the schema. The engine gives a label no meaning, and writes no words of its own for one.

   To show the writer of an existing document the guidance of its parts, call `guide` with the states of the document, or with the states that it is moving to, and with the host that derives its scaffold, so that the guide holds the same words. The following example passes the null host. The guide lists the schema's guidance, each frontmatter key, the preamble, and each section with its blocks, in the order of the schema. Each part has its own guidance, the text of the guidance line that the scaffold writes for it, whether it is optional at those states, and its limits, each with its kind, its count, and its `when-at-limit`. The guide owns its data and serialises with serde:

<!-- sokf:include /crates/lib/tramlines-md/tests/it/guide.rs#guide -->

```rust
    // The guidance of each part of a document at its states, or at the
    // states that it is moving to, as a host shows it to the writer.
    let open = guide(&schema, &states(&[("state", "open")]), &NullHost);
    let json = serde_json::to_value(&open).expect("the guide serialises");
```

<!-- /sokf:include -->

5. Add keys of your own with a host extension. An extension names the format grammar that it extends, and it can only add to that grammar: keys, rule maps, enum values, and `doc` text. The engine keeps each key that an extension adds unchanged. Your program reads and checks the key itself. A key that the extension adds to a unit rule stays in the `extensions` of the lowered `Unit`. To check such a key against the units that the rule accepts, return `true` from `Host::records_accepted`. Each check then lists in `CheckOutput::accepted` each paragraph, item, and row that a rule accepts, with its lines and the `Unit::index` of its rule, in the order of the units in the document, and `Schema::unit` returns the rule with that index.

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

6. Layer the formats of packs, if your program copies sets of schemas from other repositories, each with a formats file of its own. A name resolves through four layers, the nearest first: the `formats` of the schema, your program's own formats file, the pack layer, and the shared formats, which are your program's built-in formats. Build each file of the pack layer with `FormatRegistry::layer` on the shared layer, with `FormatSource::Pack` and the name of its pack, so that a printed registry gives `pack:` and the pack as the `source` of each entry. The method `with_packs` puts the pack layer on the shared layer and returns a `PackLayer`. It reads the files in the order of their paths, whatever order you give them in. Build your own layer on `PackLayer::registry`, then pass it and the path of your formats file to `PackLayer::with_host`, which returns the registry that your schemas compile against and the findings of the pack layer and of your layer. Two files that define one format or one type clash, and `with_host` reports each such name as [TL_128][sokf:errors-formats-and-examples#tl_128], unless your own formats file defines it in full, without `patch`. A type of a nearer layer can make a format of a farther layer too large to compile. The format is then left out, and `with_host` reports it on the file that declares the type. Your own formats file can settle a shared format that a type of a pack breaks: define the format in full, patch it, or define a type against which it compiles. `PackLayer::registry` keeps such a format as the shared layer compiled it, for your patch to merge onto. Give your schemas the registry of `with_host`, never `PackLayer::registry`. If your program lays one registry on another with `FormatRegistry::layered`, report each `BrokenFormat` that it returns on the file of the nearer layer. Check the examples of each formats file with `check_formats`, once, on that file. Report each finding as your program's own.

<!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#packs -->

```rust
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
```

<!-- /sokf:include -->

A host also supplies the facts that lie outside a document. Implement the `Host` trait of the `check` module, in place of `NullHost`. A host can then own frontmatter keys, add a frontmatter format, resolve what an image or a region names, and spell markers in its own namespace. [How the grammar layers fit together][sokf:explanation-grammar-layers] explains what a host can change.

## Troubleshooting

- `TL_12` on a key that your extension adds: you compiled the schema with a grammar that lacks the extension. Compile with the grammar that `extend` changed.
- `TL_128` from `PackLayer::with_host`: two packs define one format or one type. Define the name in full in your program's own formats file to choose one entry, or remove it from one pack.
- `extend` returns an error: the extension changes something that the grammar under it defines, or names another grammar in `extends`. An extension can only add to a grammar.

## See also

- [How the grammar layers fit together][sokf:explanation-grammar-layers]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [The error reference][sokf:reference-errors]

<!-- sokf:links -->

[sokf:errors-formats-and-examples#tl_128]: /docs/reference/errors-formats-and-examples.md#tl_128
[sokf:errors-units#tlmd_235]: /docs/reference/errors-units.md#tlmd_235
[sokf:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:reference-errors]: /docs/reference/errors.md
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
