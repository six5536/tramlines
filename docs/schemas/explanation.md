---
type: Schema
id: schema-explanation
title: Explanation schema
description: Why tramlines works the way it does, for a reader who wants to understand one part of tramlines.
---

<!-- prettier-ignore -->
````yaml
line-limit: 400

guidance: >
  An explanation describes why tramlines works the way it does. It covers the idea behind
  one part, the choices in its design, and their consequences for the reader. It links to
  the how-to guides for steps. It links to each criterion of the specification that it
  discusses, instead of restating the rule, because the criterion is the authority.

frontmatter:
  type:
    const: Explanation
  id:
    template: 'explanation-${:slug}'
  title:
    example: How the grammar layers fit together
  description:
    guidance: The description says what the explanation clarifies, in one sentence.
    example: Why tramlines checks a schema against three grammars merged into one, and what each one adds.
  order:
    pattern: '^[1-9][0-9]*$'
    guidance: The order gives the place of the explanation among the explanations, smallest first.
    example: '10'

sections:
  - heading: { from: title }
    level: 1
    subheadings-open: true
    blocks:
      - kind: prose
        guidance: The opening states the question that the explanation answers, and a short answer.
        example: A schema is written in the vocabulary of three grammars at once. Each one adds keys to the one under it, and none takes a key away.
  - heading: See also
    level: 2
    blocks:
      - kind: bullet-list
        guidance: The list links at least one criterion of the specification that the page explains, then the guides that apply those criteria. Each item is a link.
        rules:
          - pattern: '\[TL(MD)?_[0-9]+\]\[sokf:tramlines:tramlines(-md)?-spec#'
            min: 1
            example: '[TL_3][sokf:tramlines:tramlines-spec#tl_3]: how a format grammar is merged onto the core grammar.'
          - pattern: '^\['
            min: 0
            example: '[Embed the engine and extend the grammar][sokf:how-to-embed-the-engine]'
````
