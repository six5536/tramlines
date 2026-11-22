---
type: Schema
id: schema-project-rules
title: Project rules schema
description: A project rules document holds the rules for every change to the repository, such as its commits, branches, and checks, grouped by topic. Rules for code and for writing belong in the standards.
---

<!-- prettier-ignore -->
````yaml
line-limit: 200
root: /knowledge/project-rules

guidance: >
  A project rules document lists the rules that apply to the whole repository. It shows
  only the rules that apply now:
  a rule that changes is rewritten, and a rule that no longer applies is deleted. A rule
  about code belongs in a coding standard, a rule about writing in a documentation standard.

frontmatter:
  type:
    const: ProjectRules
  id:
    template: "project-rules-${:slug}"
    guidance: >
      The slug names the set of rules in a word or two, such as `general` or `release`.

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section says what the rules apply to, and ends with a contents list. The
      sections that follow hold the rules.
    blocks:
      - kind: prose
        example: >-
          These rules apply to every change to the plumber's repository.
      - kind: bullet-list
        guidance: >
          The contents list has one item for each level-2 section, in the order of the
          sections. Each item links to the section's heading.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Checks](#checks)'
            - '[Commits](#commits)'
  - heading: Precedence
    level: 2
    optional: true
    guidance: >
      This section lists the rules documents in order, the one whose rule wins first. A
      sentence that ends with a colon introduces the list.
    blocks:
      - kind: prose
        max: { soft: 1, when-at-limit: "Put the reason for a document's place in its item." }
        each:
          pattern: '^[A-Z].*:$'
          example: 'When two rules disagree, follow the one whose document comes first:'
      - kind: numbered-list
        example:
          - This document
          - The general rules of the Plan 9 source tree
  - heading: { template: '${}', example: Checks }
    level: 2
    repeat: true
    guidance: >
      Each section contains the rules about one topic, as a list in which each item is one
      rule, written as an instruction. A reason follows only where the rule would otherwise
      surprise the reader, and each further reason is a sentence of its own. A link to a rule points to its section.
    examples:
      - |
        ## Commits

        - Name the commit's issue in the commit's body, not in its subject, so that a reader of the log finds the reason for the change.
        - Write the subject in the Conventional Commits form, such as `feat(cli): add a --json option`.
    blocks:
      - kind: bullet-list
        example:
          - 'Run `mk test` before you commit.'
          - 'Start no run of the build service unless a person asks for it, because the local checks already test the change, and a run of the service is slow.'

````
