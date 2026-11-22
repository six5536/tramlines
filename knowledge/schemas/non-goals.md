---
type: Schema
id: schema-non-goals
title: Non-goals schema
description: A non-goals document lists what the project's system leaves out on purpose, each with its reason, for a developer or agent to check before adding a feature.
---

<!-- prettier-ignore -->
````yaml
line-limit: 200
labels: [human, agent]

guidance: >
  The non-goals document lists what the system doesn't do, each thing with its reason. A
  developer reads it before adding a feature, so that nobody builds something that the
  system leaves out on purpose. A knowledge store has one non-goals document, with the id
  `non-goals`.

frontmatter:
  type:
    const: NonGoals
  id:
    const: non-goals
  title:
    template: "What ${} doesn't do"
    example: What the plumber doesn't do
  description:
    example: The things that the plumber doesn't do, each with its reason.

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        guidance: >
          The opening says what the list contains, the things that the system leaves out on
          purpose, each with its reason. It also says that a developer reads the list before
          adding a feature. Each item holds the reason for its non-goal. The architecture's
          Overview holds the background on the system, such as what it is for, and the
          glossary holds the meaning of each term.
        example: This document lists the things that the plumber leaves out on purpose, each with its reason. A developer reads the list before adding a feature to the plumber.
      - kind: bullet-list
        guidance: >
          Each item has a topic in bold with a colon, one sentence about something the
          system doesn't do, and the reasons after _Why:_. Where the system does something
          else instead, a sentence of its own says what, before _Why:_. Each reason is one
          sentence, and an item can give several. An item records
          something that a developer would expect the system to do. That thing extends an
          existing feature, or someone proposed it and the developers rejected the
          proposal. A rejected way to build one building block belongs in that block's
          design, as a decision, and stays in this list while the block has no design. The
          change that makes the system do one of these things also removes that item. Its
          label says whether the developer set the non-goal or an agent proposed it.
        label: required
        rules:
          - template: '**${}**: ${}. Why: ${}.'
            rule-id: non-goal
            min: 1
            example:
              - '[agent] **Delivery confirmation**: The plumber doesn''t tell a sender when the receiving program has read the message. A write to `send` returns when the plumber has queued the message. Why: in a user interface, a write to `send` that never waits matters more than certain delivery.'
              - '[agent] **Data interpretation**: The plumber doesn''t interpret the data of a message beyond matching the rules'' patterns against it. Why: the rules alone decide where a message goes, and the receiving program decides what the data means.'
              - '[agent] **Other users**: The plumber doesn''t pass a message to another user''s programs. Why: one plumber runs for each user on each terminal, and it applies only that user''s rules.'
````
