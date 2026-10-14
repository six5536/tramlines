---
type: Schema
id: schema-tutorial
title: Tutorial schema
description: A step-by-step lesson in which a reader who is new to tramlines builds one thing that works.
---

<!-- prettier-ignore -->
````yaml
line-limit: 600

guidance: >
  A tutorial is a lesson. The reader follows it from the first step to the last and
  finishes with one thing that works. Each step builds on the step before it. Each step
  ends with a run that the reader can compare with their own. The tutorial explains only
  what each step needs, and it links to the explanations for the rest. Every example runs,
  because the test suite runs each one and compares its output. Each file sits under a
  caption line that contains only its name in bold code. Each run of `tramlines` is in a
  `console` block with its output.

frontmatter:
  type:
    const: Tutorial
  id:
    template: 'tutorial-${:slug}'
  title:
    example: Your first schema
  description:
    guidance: The description says what the reader builds, in one sentence.
    example: Write a schema for a shopping list, one rule at a time, and check a list against it.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the tutorial among the tutorials, smallest first.
    example: '10'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 2
        guidance: The opening says what the reader builds, and what they can do at the end.
        example: In this tutorial you write a schema for a shopping list and check a list against it. At the end you can write a schema for a document of your own.
  - heading: Before you begin
    level: 2
    blocks:
      - kind: bullet-list
        guidance: This section lists what the reader needs before the first step.
        example: tramlines is installed, as the [README](https://github.com/six5536/tramlines#installation) shows.
  - heading:
      template: 'Step ${n:int}: ${}'
      example: 'Step 1: write a schema with one heading'
    level: 2
    repeat: true
    blocks-ordered: false
    blocks:
      - kind: prose
        repeat: true
        guidance: The prose says what the reader does in the step and why. A caption comes before each file.
        example: A schema starts with the heading that every list has.
      - kind: code
        repeat: true
        lang: [console]
        guidance: This block holds the run that ends the step and its output, as the example harness writes them.
        example: |
          $ tramlines validate --schema list.yaml groceries.md
          0 errors, 0 warnings
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: Each file follows the caption that names it.
        example: |
          line-limit: 50
          sections: []
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: The list says what the reader can see in the output.
        example: The first line names the heading that is missing.
  - heading: What's next
    level: 2
    blocks:
      - kind: bullet-list
        guidance: This section says where the reader can go next, with one link in each item.
        example: '[Share line formats across schemas][sokf:how-to-share-line-formats]'
````
