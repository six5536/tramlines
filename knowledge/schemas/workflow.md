---
type: Schema
id: schema-workflow
title: Workflow schema
description: "A workflow maps the order of work in a project, from an idea to a closed issue, in text diagrams: its stages, the path that each kind of work takes, the documents, and how the documents trace to the code."
---

<!-- prettier-ignore -->
````yaml
line-limit: 80

guidance: >
  The workflow is a compact map of the order of work in a project, from an idea to a closed
  issue, in text diagrams. It names each stage and each document, and the schema of each
  document holds the detail of how to write it. A project has one workflow, and the
  document that maps it has the id `workflow`.

frontmatter:
  type:
    const: Workflow
  id:
    const: workflow
  description:
    example: The order of work in the plumber's repository, from an idea to a closed issue.

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section says whose order of work this is, and ends with a contents list. The
      Flow section shows the order of the work.
    blocks:
      - kind: prose
        example: >-
          This map gives the order of work in the plumber's repository.
      - kind: bullet-list
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Flow](#flow)'
            - '[Paths](#paths)'
            - '[Documents](#documents)'
            - '[Traceability](#traceability)'
  - heading: Flow
    level: 2
    guidance: This section shows the stages of a piece of work in order, from the idea to the closing of the issue.
    blocks:
      - kind: code
        lang: text
        example: |
          idea    chat with the person
          issue   Plan holds open questions
          work    one path
          close   every criterion met
  - heading: Paths
    level: 2
    guidance: This section shows each path that the work of an issue can take, one line each.
    blocks:
      - kind: code
        lang: text
        example: |
          code      architecture ─▶ contract ─▶ design ─▶ code & tests
          no code   changed files
  - heading: Documents
    level: 2
    guidance: This section shows the documents of the store as a tree, each with what it is for.
    blocks:
      - kind: code
        lang: text
        example: |
          knowledge/
          ├── workflow.md      this map
          ├── architecture.md  the building blocks and how they connect
          └── issues/          one piece of work each
  - heading: Traceability
    level: 2
    guidance: This section shows how the documents and the code link.
    blocks:
      - kind: code
        lang: text
        example: |
          contract ─▶ design ─▶ code (sokf-implements:) ─▶ tests (sokf-tests:)

````
