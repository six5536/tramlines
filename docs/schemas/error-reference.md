---
type: Schema
id: schema-error-reference
title: Error reference schema
description: The errors that tramlines reports for one part of a schema or a document, each under the key that tramlines prints.
---

<!-- prettier-ignore -->
````yaml
line-limit: 800

guidance: >
  An error reference page contains one entry for each key that `tramlines` prints for one
  part of a schema or a document, in the order of the specification. Each entry describes
  the message, its cause, and its fix. It then shows the error with four things: the
  files, a run that reports the error, the fixed file, and a run that reports no errors. A
  test fails when a key that the command can print has no entry.

frontmatter:
  type:
    const: ErrorReference
  id:
    template: 'errors-${:slug}'
  title:
    template: 'Errors: ${}'
    example: 'Errors: frontmatter'
  description:
    guidance: The description says which errors the page lists, in one sentence.
    example: The errors that tramlines reports for the frontmatter of a document, and how to fix each one.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the page among the error pages, smallest first.
    example: '30'

types:
  criterion: 'TL(?:MD)?_[1-9][0-9]*'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says which errors the page lists, and which section of the specification defines them.
        example: These errors come from the frontmatter of a document, which section 6 of the core specification defines.
  - heading:
      template: '${key:criterion}'
      key: key
      example: TL_54
    level: 2
    repeat: true
    rule-id: entry
    blocks:
      - kind: prose
        max: 1
        guidance: The entry opens with what the error means, in one sentence, with a link to its criterion.
        example: A key that the schema requires is missing from the frontmatter ([TL_54][sokf:tramlines:tramlines-spec#tl_54]).
  - heading: Message
    level: 3
    parent: entry
    blocks:
      - kind: code
        lang: [text]
        guidance: The block shows the message as the command prints it, with each part that changes in angle brackets.
        example: |
          <file>:<line>: error[TL_54]: frontmatter key `<key>` is missing (schema <id>, key `<key>`)
  - heading: Cause
    level: 3
    parent: entry
    blocks:
      - kind: prose
        guidance: This section says what in the schema or the document causes the error.
        example: The schema declares the key under `frontmatter`, without `optional`, and the document leaves it out.
  - heading: Fix
    level: 3
    parent: entry
    blocks:
      - kind: prose
        guidance: This section says what to change, in the document or in the schema.
        example: Add the key to the frontmatter of the document, or set `optional` on the key in the schema.
  - heading: Example
    level: 3
    parent: entry
    blocks-ordered: false
    blocks:
      - kind: prose
        repeat: true
        guidance: A caption comes before each file, and a sentence before each run.
        example: '**`note.md`**'
      - kind: code
        repeat: true
        lang: [console]
        guidance: The first run reports the error, and a run after the fix reports no errors.
        example: |
          $ tramlines validate --schema note.yaml note.md
          0 errors, 0 warnings
      - kind: code
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          ---
          title: Groceries
          ---
````
