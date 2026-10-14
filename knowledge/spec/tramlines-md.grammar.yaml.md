---
type: Grammar
id: tramlines-md-grammar
title: tramlines-md markdown grammar usage guide
description: Where to learn to write the body of a markdown schema, which the markdown format grammar defines.
resource: /knowledge/spec/tramlines-md.grammar.yaml
version: "0.1"
---

# tramlines-md markdown grammar usage guide

The markdown format grammar, `tramlines-md.grammar.yaml`, adds to the core grammar what only a markdown document has: sections, blocks, units, keys and anchors, and scaffolds. The [markdown specification][sokf:tramlines-md-spec] defines it, and the documentation for users teaches it, with examples that run.

## <a id="1"></a>1. Where to read

- [Your first schema][sokf:tramlines-docs:tutorial-first-schema]: a schema written one rule at a time.
- [Schema keys][sokf:tramlines-docs:reference-schema-keys]: every key. A test writes that page from this file and the core grammar.
- [Describe sections and headings][sokf:tramlines-docs:how-to-describe-sections]: headings, levels, open regions, parents, numbers, and the preamble.
- [Describe blocks and their units][sokf:tramlines-docs:how-to-describe-blocks] and [How blocks are matched to a schema][sokf:tramlines-docs:explanation-block-matching]: the kinds of block, and how the engine matches a body.
- [Select and count units and lines][sokf:tramlines-docs:how-to-select-and-count-lines]: the `rules` of a block, guards, counters, and the `lines` of a code block.
- [Capture keys and anchor them][sokf:tramlines-docs:how-to-capture-keys]: keys, pairings, derivations, and anchors.
- [How a schema derives its scaffold][sokf:tramlines-docs:explanation-scaffolds]: scaffolds and their markers.
- [Pitfalls in writing a schema][sokf:tramlines-docs:explanation-pitfalls]: the mistakes that the checks do not catch.

<!-- sokf:links -->

[sokf:tramlines-docs:explanation-block-matching]: /docs/explanation/block-matching.md
[sokf:tramlines-docs:explanation-pitfalls]: /docs/explanation/pitfalls.md
[sokf:tramlines-docs:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:tramlines-docs:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:tramlines-docs:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:tramlines-docs:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:tramlines-docs:how-to-select-and-count-lines]: /docs/how-to/select-and-count-lines.md
[sokf:tramlines-docs:reference-schema-keys]: /docs/reference/schema-keys.md
[sokf:tramlines-docs:tutorial-first-schema]: /docs/tutorials/first-schema.md
[sokf:tramlines-md-spec]: /knowledge/spec/tramlines-md-spec.md
