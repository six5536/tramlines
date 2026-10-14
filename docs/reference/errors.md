---
type: Reference
id: reference-errors
title: Errors
description: How to read an error from tramlines, and where each error is explained.
order: 30
---

# Errors

Every error and warning that tramlines prints contains a key, such as `TL_54` in `error[TL_54]`. The key names the rule of the specification that the document or the schema breaks. Keys that start with `TL_` come from the core specification, which every format shares. Keys that start with `TLMD_` come from the Markdown specification. Each of the following pages explains the errors of one part of a schema or a document. To find a key, search for it on the site.

## Pages

- [The schema][sokf:errors-schema]: faults in the YAML, keys, values, and regular expressions of the schema itself.
- [Shapes and templates][sokf:errors-schema-shapes]: the one shape of each rule, and the holes of a template.
- [Schema types and keys][sokf:errors-schema-types-keys-lifecycle]: the types, keys, key derivations, and bounds of a schema.
- [Lifecycle and ids][sokf:errors-lifecycle-and-ids]: the lifecycle of a schema, the rules that it gates, and the `id` template.
- [Formats][sokf:errors-formats-and-examples]: line formats, and a line that lacks the key of its format.
- [Examples and guidance][sokf:errors-examples]: the examples of the rules and formats of a schema, and its `examples`.
- [Frontmatter][sokf:errors-frontmatter]: the frontmatter of a document.
- [Lines and length][sokf:errors-lines]: the shape of the lines of a document, its bans, and its length.
- [Headings and sections][sokf:errors-sections]: the headings of a document and the sections they open.
- [Sections inside sections][sokf:errors-nested-sections]: sections that declare a `parent`, and their numbers.
- [Blocks][sokf:errors-blocks]: the blocks that a section contains, and the number of units in each block.
- [Kinds of block][sokf:errors-block-kinds]: the keys, tags, columns, and HTML that each kind of block allows.
- [Units][sokf:errors-units]: the paragraphs and items that the rules of a block take, select, and count.
- [Lines and pairings][sokf:errors-lines-and-pairings]: the lines of a code block, and the items that pair with the keys of another rule.
- [Markers][sokf:errors-keys-markers-scaffolds]: the markers of a scaffold that a document keeps.
- [Scaffolds][sokf:errors-scaffolds]: the scaffold that a schema derives for a new document.
- [Keys and repairs][sokf:errors-keys-and-repairs]: keys and their anchors, and the text that a repair rewrites.
- [The command][sokf:errors-command]: errors that stop `tramlines validate` before it checks a document.

## How an error reads

```text
<file>:<line>: error[<KEY>]: <message> (schema <id>, <place of the rule>)
```

The end of the message names the rule in your schema that the document breaks, by its `rule-id`, or by its section and its position in the section. A message that quotes an example line shows what the rule expects. An error in the schema file itself names no rule, because the fault is in the schema.

<!-- sokf:links -->

[sokf:errors-block-kinds]: /docs/reference/errors-block-kinds.md
[sokf:errors-blocks]: /docs/reference/errors-blocks.md
[sokf:errors-command]: /docs/reference/errors-command.md
[sokf:errors-examples]: /docs/reference/errors-examples.md
[sokf:errors-formats-and-examples]: /docs/reference/errors-formats-and-examples.md
[sokf:errors-frontmatter]: /docs/reference/errors-frontmatter.md
[sokf:errors-keys-and-repairs]: /docs/reference/errors-keys-and-repairs.md
[sokf:errors-keys-markers-scaffolds]: /docs/reference/errors-keys-markers-scaffolds.md
[sokf:errors-lifecycle-and-ids]: /docs/reference/errors-lifecycle-and-ids.md
[sokf:errors-lines]: /docs/reference/errors-lines.md
[sokf:errors-lines-and-pairings]: /docs/reference/errors-lines-and-pairings.md
[sokf:errors-nested-sections]: /docs/reference/errors-nested-sections.md
[sokf:errors-scaffolds]: /docs/reference/errors-scaffolds.md
[sokf:errors-schema]: /docs/reference/errors-schema.md
[sokf:errors-schema-shapes]: /docs/reference/errors-schema-shapes.md
[sokf:errors-schema-types-keys-lifecycle]: /docs/reference/errors-schema-types-keys-lifecycle.md
[sokf:errors-sections]: /docs/reference/errors-sections.md
[sokf:errors-units]: /docs/reference/errors-units.md
