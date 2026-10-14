---
type: Explanation
id: explanation-non-goals
title: What tramlines doesn't do
description: The things tramlines leaves out on purpose, why each is left out, and where that work belongs instead.
order: 60
---

# What tramlines doesn't do

tramlines validates one document at a time against one schema, and reads only the files that you name. It leaves out the work around that check on purpose: links between documents, an editor integration, and other formats and schema languages. A host program can provide that work, or the project can add it when someone needs it.

## Why it stays small

tramlines is an engine that other programs embed, and a command that runs the engine on its own. Each feature that it leaves out would tie it to one way of storing, editing, or serving documents. Without those features, the same engine works with every host, and a check gives the same result for the same files on any machine.

## Checks across documents

tramlines doesn't follow links, resolve ids that another document defines, build a graph of documents, or know about a store of documents. When a rule needs information from outside the document, the engine asks the host program. The `tramlines` command has no host, so it checks only what's inside each document. For example, the command checks that a key is unique within one document. A host can check that the key is unique across many documents.

Why: the program that manages a set of documents owns the model of how they relate. The engine leaves that model out, so each host can define it in its own way.

## Knowledge of any particular host

tramlines knows nothing of any program that embeds it. A host adds its own schema keys through a grammar extension, and answers the engine's questions about what's outside a document. The engine keeps the host's keys as written and leaves them for the host to check.

Why: an engine that knows nothing of its hosts can serve all of them. [How the grammar layers fit together][sokf:explanation-grammar-layers] explains how a host adds to the grammar.

## An editor integration

tramlines has no language server and no editor extension. It prints its findings, one per line, and exits with a code that shows whether any finding is an error.

Why: the host program owns the files and the editor, and decides where to show the findings. [The command reference][sokf:reference-cli] describes the output format to build on.

## Formats other than Markdown

tramlines validates only Markdown documents. The command picks the format of a file by its extension, `.md` for Markdown. On a file with any other extension, the command stops, unless you name a format with `--format`:

**`list.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - kind: bullet-list
        example: Milk
```

**`list.txt`**

```text
Groceries: milk, bread
```

```console
$ tramlines validate --schema list.yaml list.txt
error[TL_90]: list.txt: no format matches the extension `txt`; name one with `--format` (`md`)
[exit 2]
```

Why: the core grammar leaves room for a second format, but the project builds a second format only when a user needs one.

## Schema languages other than YAML

A schema must be YAML 1.2. tramlines doesn't read JSON Schema, a schema written in code, or any other schema language.

Why: one schema language means one grammar, one set of error messages, and one guide to learn.

## Files you didn't name

tramlines reads only the files that you, or the program that calls it, name: the schema, a file of shared formats, and the documents. It doesn't search directories for documents, fetch anything, or open a network connection.

Why: a check must give the same findings for the same files, wherever it runs. Finding the files is the job of whoever runs the check, such as a script in continuous integration (CI) or a host program.

## See also

- [TL_9][sokf:tramlines:tramlines-spec#tl_9]: the engine keeps a key that a host adds, for the host to check.
- [TL_79][sokf:tramlines:tramlines-spec#tl_79]: a rule is skipped when the host can't supply what the rule needs.
- [TL_85][sokf:tramlines:tramlines-spec#tl_85]: the command checks each document that it names, with no host.
- [TL_88][sokf:tramlines:tramlines-spec#tl_88] and [TL_90][sokf:tramlines:tramlines-spec#tl_90]: the format of a document comes from its file extension.
- [TL_13][sokf:tramlines:tramlines-spec#tl_13]: a schema that does not parse as YAML 1.2 is refused.
- [Embed the engine in a Rust program][sokf:how-to-embed-the-engine]
- [Check documents in CI][sokf:tutorial-check-documents-in-ci]

<!-- sokf:links -->

[sokf:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:tramlines:tramlines-spec#tl_13]: /knowledge/spec/tramlines-spec.md#tl_13
[sokf:tramlines:tramlines-spec#tl_79]: /knowledge/spec/tramlines-spec.md#tl_79
[sokf:tramlines:tramlines-spec#tl_85]: /knowledge/spec/tramlines-spec.md#tl_85
[sokf:tramlines:tramlines-spec#tl_88]: /knowledge/spec/tramlines-spec.md#tl_88
[sokf:tramlines:tramlines-spec#tl_9]: /knowledge/spec/tramlines-spec.md#tl_9
[sokf:tramlines:tramlines-spec#tl_90]: /knowledge/spec/tramlines-spec.md#tl_90
[sokf:tutorial-check-documents-in-ci]: /docs/tutorials/check-documents-in-ci.md
