---
type: Schema
id: schema-how-to
title: How-to guide schema
description: One task with tramlines, shown from start to finish, for a reader who knows what they want to do.
---

<!-- prettier-ignore -->
````yaml
line-limit: 400

guidance: >
  A how-to guide shows one task from start to finish, for a reader who already knows what
  they want to do. The guide states only the rules that the task needs. It links to the
  reference and the explanations for the rest. Every example runs, because the test suite
  runs each one and compares its output. Each file sits under a caption line that contains
  only its name in bold code. Each run of `tramlines` is in a `console` block with its
  output.

frontmatter:
  type:
    const: HowTo
  id:
    template: 'how-to-${:slug}'
  title:
    pattern: '^[A-Z][a-z]+ [^.]+$'
    guidance: The title names the task, starting with a verb, in the words that a reader searches for.
    example: Share line formats across schemas
  description:
    guidance: The description says what the reader achieves, in one sentence.
    example: Put a line format in a formats file, so that two schemas check the same kind of line in the same way.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the guide among the how-to guides, smallest first.
    example: '20'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says when the reader needs to do this task, and what they have when they finish.
        example: When two schemas check the same kind of line, write its format once in a formats file and name it from each schema.
  - heading: Before you begin
    level: 2
    optional: true
    blocks:
      - kind: bullet-list
        guidance: Each item names one thing that the reader needs before the first step, with a link to the page that explains it.
        example: A schema that checks a list, as in [Your first schema][sokf:tutorial-first-schema].
  - heading: Steps
    level: 2
    blocks-ordered: false
    blocks:
      - kind: numbered-list
        repeat: true
        guidance: >
          Each item holds one action. A file or a run that belongs to a step follows the
          item, or sits inside the item, indented to the item's text.
        each:
          example: Write the format in a formats file.
          blocks:
            - kind: prose
              optional: true
              repeat: true
              example: 'This changelog schema bans the word `TODO`:'
            - kind: bullet-list
              optional: true
              repeat: true
              example: '`min: 2` demands two items.'
            - kind: prose
              optional: true
              repeat: true
              example: 'Each item names one bound.'
            - kind: code
              optional: true
              repeat: true
              lang: any
              example: |
                line-limit: 0
      - kind: prose
        optional: true
        repeat: true
        guidance: 'A caption is the name of a file in bold code, or a sentence that describes what a run shows.'
        example: '**`formats.yaml`**'
      - kind: code
        optional: true
        repeat: true
        lang: [console]
        guidance: >
          This block holds a run of `tramlines` and its output, as the example harness writes
          them. Every guide to a task with the command includes at least one run. A guide for a
          host program shows the Rust code of the program instead.
        example: |
          $ tramlines validate --schema list.yaml groceries.md
          0 errors, 0 warnings
      - kind: include
        optional: true
        repeat: true
        source-pattern: '^/crates/'
        guidance: >
          The Rust code for a host program comes from a test of the repository, so that the
          code compiles and runs as shown.
        example: |
          <!-- sokf:include /crates/lib/tramlines-md/tests/it/embed.rs#check -->
          <!-- /sokf:include -->
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          registry:
            item:
              template: "${name:word} x${n:int}"
              example: Milk x2
  - heading: Troubleshooting
    level: 2
    optional: true
    blocks:
      - kind: bullet-list
        guidance: Each item gives an error that the reader might see during this task, with its key and its fix.
        example: '`TL_97`: the schema names a format that no file defines. Pass the formats file with `--formats`.'
  - heading: See also
    level: 2
    blocks:
      - kind: bullet-list
        guidance: Each item links one reference page or explanation that relates to the task.
        example: '[Line formats][sokf:reference-formats]'
````
