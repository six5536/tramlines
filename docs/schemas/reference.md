---
type: Schema
id: schema-reference
title: Reference schema
description: A page where a reader looks up facts, such as the keys of a schema or the options of the command.
---

<!-- prettier-ignore -->
````yaml
line-limit: 800

guidance: >
  A reference page lists facts for a reader to look up. It is complete, follows a fixed
  order, and tells no story. A generated page names its generator. A change to a generated
  page goes into the input of its generator, not into the page, because the generator
  rewrites the page. The errors have pages of their own type.

frontmatter:
  type:
    const: Reference
  id:
    template: 'reference-${:slug}'
  title:
    example: The command
  description:
    guidance: The description says what the reader looks up on the page, in one sentence.
    example: The options of `tramlines validate`, its output, and its exit codes.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the page in the reference, smallest first.
    example: '10'
  generator:
    optional: true
    enum: [schema-keys]
    guidance: The generator names the test that writes the page, when a test writes it.
    example: schema-keys

sections:
  - heading: { from: title }
    level: 1
    subheadings-open: true
    blocks:
      - kind: prose
        guidance: The opening says what the page lists, and the order of the list.
        example: This page lists the options of `tramlines validate`, then its output and its exit codes.
````
