---
type: Explanation
id: explanation-grammar-layers
title: How the grammar layers fit together
description: Why a schema is checked against three grammars merged into one, and what each one can add.
order: 10
---

# How the grammar layers fit together

The keys that a schema can use come from one grammar. tramlines builds that grammar from up to three layers: the core grammar, a format grammar, and a host extension. Each layer adds keys to the layer under it. No layer removes or changes a key.

## Why layers

Some parts of a schema depend on the format of the document, and some don't. The frontmatter, the lifecycle, the line formats, the types, and the prohibited patterns mean the same thing in every format. Headings, blocks, and list items exist only in Markdown. A program that embeds tramlines is called its host. A host can also need keys of its own, such as links between documents.

The layers keep each concern in one place. A second format can reuse the core grammar unchanged, and a host can add what it needs without forking the format grammar. Because a layer only adds keys, a schema that is valid against the core and Markdown grammars means the same thing under every host.

## The three layers

The engine reads the layers in order and merges each one onto the result so far.

| Layer          | Names itself with                           | Defines                                                                                                    |
| -------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| core grammar   | `grammar: tramlines`                        | The vocabulary that every grammar file is written in, and the schema keys that every format shares.        |
| format grammar | `grammar: tramlines-md`, `base: tramlines`  | What the body of a document of its format contains.                                                        |
| host extension | `extension: <name>`, `extends: tramlines-md` | The keys and rule maps that the host reads and checks itself.                                              |

A grammar file can contain two sections: `meta`, the vocabulary that grammar files are written in, and `schema`, the vocabulary that a schema uses. Only the core grammar contains a `meta` section, so every layer is written in one vocabulary.

### The core grammar

The core grammar contains the schema keys that apply to every format: `line-limit`, `guidance`, `lifecycle`, `frontmatter`, `frontmatter-open`, `prohibited-patterns`, `formats`, `types`, and `examples`. It also defines the rule maps behind them: a frontmatter constraint, a lifecycle dimension, an example, a format and its variants, and a key derivation. [The schema-keys reference][sokf:reference-schema-keys#schema] lists them.

In the vocabulary that the `meta` section defines, each key takes one of these types: `string`, `integer`, `boolean`, `list`, `map`, `object`, `string-or-list`, `string-or-map`, `text`, or `text-or-rule`. The definition of a key can also specify the type of its elements, whether the key is required, its allowed values, a format, the keys that it requires, and its documentation. A rule map can require that exactly one, or at least one, of a group of keys is present.

### The Markdown format grammar

The Markdown grammar, `tramlines-md`, names the core grammar with `base: tramlines`. It adds the vocabulary of a Markdown body:

- The schema keys `preamble`, `sections`, `sections-ordered`, and `sections-prohibited`, and the rule that a schema declares at least one of `preamble` and `sections`.
- The rule maps [`section`][sokf:reference-schema-keys#section], [`heading`][sokf:reference-schema-keys#heading], [`preamble`][sokf:reference-schema-keys#preamble], [`block`][sokf:reference-schema-keys#block], [`unit`][sokf:reference-schema-keys#unit], and [`prohibited-heading`][sokf:reference-schema-keys#prohibited-heading].

A format grammar can add keys, rule maps, values to an enum, block kinds that a key requires, text to the documentation of a key, and the "exactly one of" and "at least one of" groups. It can't change what the core grammar defines. It has no `meta` section, and it changes no type, element type, or required flag of a core key. The engine refuses a format grammar that tries.

### Host extensions

A host extension names the format grammar that it extends, with `extends`. It can add keys, rule maps, values to an enum, block kinds that a key requires, text to the documentation of a key, and sections of its own. It has no `meta` section and no constraint group, and it changes nothing that the grammar under it defines. The engine refuses an extension that breaks one of these rules. It also refuses an extension whose `extends` names a format grammar other than the one that the engine reads.

The engine keeps a key that a host adds as written, and leaves the host to read and check it. A host can therefore give schemas keys that the engine doesn't know, such as links between documents in a store.

## What this means for a schema

The `tramlines` command runs with no host, so it reads only the core and Markdown grammars. A key that a host extension defines is an unknown key to the command.

This schema uses `links`, a key that a host might add:

**`list.yaml`**

```yaml
line-limit: 0
links:
  - rel: depends-on
sections:
  - heading: Groceries
    level: 1
    blocks:
      - kind: bullet-list
        example: Milk
```

**`list.md`**

```markdown
# Groceries

- Milk
```

```console
$ tramlines validate --schema list.yaml list.md
list.yaml:2: error[TL_12]: unknown key `links` in `schema`
1 error, 0 warnings
[exit 1]
```

The command reports the key as an error in the schema, because none of the layers that the command reads defines the key. Under a host whose extension defines `links`, the same schema is valid. Write a schema for the layers that its documents are checked under, and validate the schema with the program that checks those documents.

## See also

- [TL_1][sokf:tramlines:tramlines-spec#tl_1]: the keys of the core grammar.
- [TL_3][sokf:tramlines:tramlines-spec#tl_3] and [TL_4][sokf:tramlines:tramlines-spec#tl_4]: what a format grammar can add, and what makes the engine refuse it.
- [TL_6][sokf:tramlines:tramlines-spec#tl_6], [TL_7][sokf:tramlines:tramlines-spec#tl_7], and [TL_8][sokf:tramlines:tramlines-spec#tl_8]: what a host extension can add, and what makes the engine refuse it.
- [TL_9][sokf:tramlines:tramlines-spec#tl_9]: the engine keeps a key that a host adds for the host to check.
- [TL_12][sokf:tramlines:tramlines-spec#tl_12] and [TL_101][sokf:tramlines:tramlines-spec#tl_101]: a key that the grammar doesn't define, and a value of the wrong type, are reported on the schema.
- [Built-in types][sokf:reference-types]
- [Embed the engine in a Rust program][sokf:how-to-embed-the-engine]

<!-- sokf:links -->

[sokf:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:reference-schema-keys#block]: /docs/reference/schema-keys.md#block
[sokf:reference-schema-keys#heading]: /docs/reference/schema-keys.md#heading
[sokf:reference-schema-keys#preamble]: /docs/reference/schema-keys.md#preamble
[sokf:reference-schema-keys#prohibited-heading]: /docs/reference/schema-keys.md#prohibited-heading
[sokf:reference-schema-keys#schema]: /docs/reference/schema-keys.md#schema
[sokf:reference-schema-keys#section]: /docs/reference/schema-keys.md#section
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
[sokf:reference-types]: /docs/reference/types.md
[sokf:tramlines:tramlines-spec#tl_1]: /knowledge/spec/tramlines-spec.md#tl_1
[sokf:tramlines:tramlines-spec#tl_101]: /knowledge/spec/tramlines-spec.md#tl_101
[sokf:tramlines:tramlines-spec#tl_12]: /knowledge/spec/tramlines-spec.md#tl_12
[sokf:tramlines:tramlines-spec#tl_3]: /knowledge/spec/tramlines-spec.md#tl_3
[sokf:tramlines:tramlines-spec#tl_4]: /knowledge/spec/tramlines-spec.md#tl_4
[sokf:tramlines:tramlines-spec#tl_6]: /knowledge/spec/tramlines-spec.md#tl_6
[sokf:tramlines:tramlines-spec#tl_7]: /knowledge/spec/tramlines-spec.md#tl_7
[sokf:tramlines:tramlines-spec#tl_8]: /knowledge/spec/tramlines-spec.md#tl_8
[sokf:tramlines:tramlines-spec#tl_9]: /knowledge/spec/tramlines-spec.md#tl_9
