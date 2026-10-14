---
type: Grammar
id: tramlines-grammar
title: tramlines core grammar usage guide
description: Where to learn to write the parts of a schema that every format shares, which the core grammar defines.
resource: /knowledge/spec/tramlines.grammar.yaml
version: "0.1"
---

# tramlines core grammar usage guide

The core grammar, `tramlines.grammar.yaml`, is the vocabulary of the parts of a schema that every format shares. The [core specification][sokf:tramlines-spec] defines it, and the documentation for users teaches it, with examples that run.

## <a id="1"></a>1. Where to read

- [Schema keys][sokf:tramlines-docs:reference-schema-keys]: every key. A test writes that page from this file and the markdown grammar.
- [How the grammar layers fit together][sokf:tramlines-docs:explanation-grammar-layers] and [Strict by default][sokf:tramlines-docs:explanation-strict-by-default]: what the grammar governs, its layers, and what a schema requires.
- [Write the shape of a line][sokf:tramlines-docs:how-to-write-line-shapes] and [Built-in types][sokf:tramlines-docs:reference-types]: text, templates, patterns, and types.
- [Share line formats across schemas][sokf:tramlines-docs:how-to-share-line-formats]: formats, variants, layers, and patches.
- [Apply a rule in some states only][sokf:tramlines-docs:how-to-gate-rules-by-state]: lifecycle and `only`.
- [Write examples and guidance][sokf:tramlines-docs:how-to-write-examples-and-guidance]: examples, guidance, and failures.
- [Constrain frontmatter and ids][sokf:tramlines-docs:how-to-constrain-frontmatter]: frontmatter constraints and `id` templates.
- [Ban text in a document][sokf:tramlines-docs:how-to-ban-text]: `prohibited-patterns`.
- [Embed the engine in a Rust program][sokf:tramlines-docs:how-to-embed-the-engine]: host extensions.

<!-- sokf:links -->

[sokf:tramlines-docs:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:tramlines-docs:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:tramlines-docs:how-to-ban-text]: /docs/how-to/ban-text.md
[sokf:tramlines-docs:how-to-constrain-frontmatter]: /docs/how-to/constrain-frontmatter.md
[sokf:tramlines-docs:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:tramlines-docs:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:tramlines-docs:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:tramlines-docs:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:tramlines-docs:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:tramlines-docs:reference-schema-keys]: /docs/reference/schema-keys.md
[sokf:tramlines-docs:reference-types]: /docs/reference/types.md
[sokf:tramlines-spec]: /knowledge/spec/tramlines-spec.md
