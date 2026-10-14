---
type: Schema
id: schema-grammar
title: Grammar schema
description: The sidecar of a grammar file, the usage guide to the grammar it describes.
---

<!-- prettier-ignore -->
````yaml
line-limit: 0

asset-files: /knowledge/spec/*.grammar.yaml

frontmatter:
  type:
    const: Grammar
  version:
    template: '${:int}.${:int}'
    example: "0.1"
    guidance: The version is the version of the grammar that the guide describes, as the `version` key of the grammar file gives it.

# A section body is free: any of these blocks, in any order, any number
# of times. Every fence has a tag.
sections:
  - heading: { from: title }
    level: 1
    guidance: >
      The heading is the guide's name, from the frontmatter. The opening says who the guide
      is for and what it shows.
    blocks:
      - kind: prose
        example: 'This guide shows schema authors how to write a schema, following the grammar section by section.'
  - heading: { template: '${n:int}. ${}', key: n, example: "5. Lines and guards" }
    level: 2
    repeat: true
    subheadings-open: true
    guidance: Each section has a number, and the number is the section's key.
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        example: 'A unit rule over a fence can have `lines`, the grammar of the lines that the fence contains.'
      - kind: bullet-list
        optional: true
        repeat: true
        example: 'A template shows a line exactly as written, with a typed hole where the line varies.'
      - kind: numbered-list
        optional: true
        repeat: true
        example: 'Lower the schema, then derive its scaffold.'
      - kind: blockquote
        optional: true
        repeat: true
        example: 'A pattern checks the shape of a line. An example also shows the writer what to write.'
      - { kind: table, optional: true, repeat: true, columns: [], columns-open: true }
      - kind: code
        optional: true
        repeat: true
        lang: any
        example: "lines:\n  - template: 'classDef ${:class} fill:#${:hex}'\n    min: 7\n    max: 7"

````
