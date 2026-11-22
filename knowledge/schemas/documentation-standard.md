---
type: Schema
id: schema-documentation-standard
title: Documentation standard schema
description: A documentation standard sets the rules for one kind of writing in the project, such as its prose, its formatting, or its schemas, grouped by topic.
---

<!-- prettier-ignore -->
````yaml
line-limit: 200
labels: [human, agent]
root: /knowledge/documentation-standards

guidance: >
  A documentation standard sets the rules for one kind of documentation. Every writer of
  that documentation, an agent most of all, reads it before writing or updating it. The standard opens with
  what it covers and lists its sections. An optional Precedence section follows, and then
  one section for each topic.

frontmatter:
  type:
    const: DocumentationStandard
  id:
    template: "documentation-standard-${:slug}"

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section says which documentation the rules cover, and names the standards that
      the rules add to. When the rules come from a published guide, the
      section cites the guide in a footnote and states its license, because the standard
      copies the guide's words. The frontmatter lists the guide in `sources`.
    blocks:
      - kind: prose
        example: 'This standard sets the rules for the manual pages of the plumber, `plumber(4)`, `plumb(1)`, `plumb(2)`, and `plumb(6)`, and for the comments in the default rules file `/sys/lib/plumb/basic`.'
      - kind: bullet-list
        guidance: >
          The contents list has one item for each level-2 section, in
          order, because a reader, an agent most of all, reads the start of a document first.
          Each item links to the heading of its section.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Precedence](#precedence)'
            - '[Files, rules, and messages](#files-rules-and-messages)'
            - '[Diagnostics](#diagnostics)'
            - '[Bugs](#bugs)'
  - heading: Precedence
    level: 2
    optional: true
    guidance: >
      This section lists the sources of style in order of precedence, with one source in each
      item, so that a writer knows which source to follow when two disagree. A sentence
      that ends with a colon introduces the list.
    blocks:
      - kind: prose
        max: { soft: 1, when-at-limit: "Put the reason for a source's place in its item." }
        each:
          pattern: '^[A-Z].*:$'
          example: 'When two sources of style disagree, follow the one that comes first:'
      - kind: numbered-list
        example:
          - This standard
          - The page `plumber(4)`
          - The other pages of the plumber, `plumb(1)`, `plumb(2)`, and `plumb(6)`
  - heading: { template: '${}', example: 'Files, rules, and messages' }
    level: 2
    repeat: true
    guidance: >
      Each topic section states the rules for one topic, one rule in each item. Each rule
      addresses the writer as _you_, in the imperative. A rule that chooses one form among
      several carries its reason, and each further reason is a sentence of its own. With the reason, a writer can apply the rule to new cases. A self-evident rule carries no reason. Where the rules share a scope or a
      principle that the heading doesn't name, a sentence that ends with a colon states it
      and introduces the list. Each rule's label says whether the developer set the rule or an
      agent proposed it.
    examples:
      - |
        ## Diagnostics

        Follow these rules for the errors that a page lists:

        - [agent] Quote each error exactly as the program writes it, such as `no matching plumb rule`, because a reader searches the page for that text.
        - [agent] Say which call fails with each error: a write to `/mnt/plumb/send` fails with `no matching plumb rule` when no rule set matches and the message names no port.
      - |
        ## Bugs

        Follow these rules for the BUGS section of a page:

        - [agent] State each limit plainly, as a fact about the program: "The plumber gives a sender no way to know when a message has been read."
        - [agent] Give the reason for a limit that the design chose, because a reader might otherwise report the limit as a fault, for example: "The design accepts the preceding limit, because in a user interface a write that never waits matters more than certain delivery."
    blocks:
      - kind: prose
        optional: true
        max: { soft: 1, when-at-limit: "Put the reason for a rule in its item." }
        each:
          pattern: '^[A-Z].*:$'
          example: 'Show each file, rule, and message in its full and exact form:'
      - kind: bullet-list
        label: required
        example:
          - '[agent] Name each file by its full path, such as `/mnt/plumb/rules`, because _rules_ alone could mean any rules file.'
          - '[agent] Show each rule as its object, verb, and argument, such as `plumb to edit`, because a reader copies the rule into a rules file in that form.'
          - '[agent] Show each message as its six header lines followed by its data, as a program writes it to `send`.'

````
