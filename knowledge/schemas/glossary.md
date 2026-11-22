---
type: Schema
id: schema-glossary
title: Glossary schema
description: The canonical definitions of terms used throughout this project.
---

<!-- prettier-ignore -->
````yaml
line-limit: 2000

guidance: >
  A glossary defines each term that the documents of a knowledge store use, so that no
  other document defines it. A document links the first use of a term to the term's entry,
  for example `[port][sokf:glossary-project#port]`. A reader who meets an unfamiliar term
  follows that link. A knowledge store's glossaries are in `/knowledge/glossaries/`, such as
  `workflow.md` for the terms of the workflow, which a pack supplies, and `project.md` for
  the terms of the project's own domain. A term has one entry, in one glossary. A glossary
  defines terms and holds no rules: a rule belongs in the document that the rule governs.

root: /knowledge/glossaries

frontmatter:
  type:
    const: Glossary
  id:
    template: 'glossary-${:slug}'
  description:
    example: 'The terms that the plumber''s documents use for messages, ports, and rules.'

types:
  term: '[^*]+'
  definition: '[A-Z].*'

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section ends with a contents list.
    blocks:
      - kind: prose
        optional: true
        guidance: >
          The prose says what the glossary defines, where its title and description leave it
          unclear. Each term's entry holds the term's detail.
        example: 'This glossary defines each term that the documents of the plumber use for its own domain.'
      - kind: bullet-list
        guidance: The contents list has one item for each level-2 section, in order, and each item links to its section's heading.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Messages](#messages)'
            - '[Rules](#rules)'

  - heading: { template: '${}', example: Messages }
    level: 2
    optional: true
    repeat: true
    guidance: >
      Each section holds the terms of one area, such as the messages of a system or the
      people who work on a project, and the sections come in the order that a new reader
      meets their areas. A glossary has as many sections as its terms need.
    examples:
      - |
        ## Rules

        - **Object**: The part of a message that a rule tests or uses, such as `src`, `type`, or `data`.
        - **Rule**: One line of a rule set, with an object, a verb, and an argument, such as `data matches '[a-zA-Z0-9_\-./]+'` or `plumb to edit`.
        - **Rule set**: A group of rules, separated from the next group by a blank line. The plumber tries the rule sets in order, and the first set whose patterns all succeed decides where the message goes.
        - **Verb**: The word of a rule that says what the rule does with its object, such as `matches`, `isfile`, or `to`.
    blocks:
      - kind: bullet-list
        guidance: >
          Each item defines one term, and the items are in alphabetical order. Each term is
          in sentence case. A term's slug differs from the slug of every heading, because
          the term's anchor and the heading's anchor would otherwise be the same. A
          definition says what the term means to a reader who is new to the system. The
          document that the term belongs to holds more. An item ends with
          `Also known as …` when people use a different word for the same thing, and with
          `Not to be confused with …` and a link when a reader might confuse the term with
          another. When a specification defines a term, the item summarises the definition,
          and a footnote and a `sources` entry cite the specification.
        each:
          template: '**${term:term}**: ${:definition}'
          key: term
          unique: knowledge
          example:
            - '**Click attribute**: The attribute `click`, whose value is the position of the cursor in the data of a message. When a message has this attribute, the plumber sends only the part of the data that the patterns of the matching rule set match around that position.'
            - '**Message**: The unit that the plumber passes between programs. It has six header lines, followed by the data. The header lines give the source application, the destination port, the working directory, the type of the data, the attributes, and the length of the data.'
            - '**Port**: A file in `/mnt/plumb`, such as `edit` or `image`, from which programs read the messages for one kind of work. Each program that has opened the port receives one copy of each message sent to the port.'
````
