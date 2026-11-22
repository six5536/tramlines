---
type: Schema
id: schema-coding-standard
title: Coding standard schema
description: A coding standard holds the project's rules for code in one language, grouped by topic, which a developer or an agent reads before writing or reviewing that code.
---

<!-- prettier-ignore -->
````yaml
line-limit: 200
labels: [human, agent]
root: /knowledge/coding-standards

guidance: >
  A coding standard holds the rules for the code of one language in the project, for a
  developer or an agent who writes or reviews that code. The standard has one section for
  each topic, such as error handling or memory.

frontmatter:
  type:
    const: CodingStandard
  id:
    template: "coding-standard-${:slug}"
  language:
    template: '${:language}'
    guidance: >
      The value of `language` is the language's name as it appears after the opening
      backticks of a code fence, in lower case, such as `c` or `rust`.
    example: c

types:
  language: '[a-z][a-z0-9+#-]*'

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section says which code the rules cover, and ends with a contents list.
    blocks:
      - kind: prose
        example: >-
          These rules cover the C code of the plumber, in `/sys/src/cmd/plumb`.
      - kind: bullet-list
        guidance: >
          The contents list has one item for each topic, in the order of the sections. Each
          item links to the topic's heading.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Error handling](#error-handling)'
            - '[Memory](#memory)'
            - '[Names and layout](#names-and-layout)'
            - '[Header files](#header-files)'
  - heading: { template: '${}', example: Error handling }
    level: 2
    repeat: true
    guidance: >
      Each section contains the rules about one topic, as a list in which each item holds one
      rule, written as an instruction. A rule that chooses one form among several carries its
      reason, and each further reason is a sentence of its own. Each rule's label says whether
      the developer set the rule or an agent proposed it.
    examples:
      - |
        ## Memory

        - [agent] Allocate with `emalloc` and `estrdup`, which never return nil, so that no caller has to check the result.
        - [agent] Free each `Plumbmsg` in the function that allocates it, unless that function queues the message on a port. The functions in `fsys.c` free a queued message after they deliver it.
      - |
        ## Names and layout

        - [agent] Put each function's name at the start of a line in its definition, so that `grep '^matchruleset('` finds the definition of `matchruleset`.
        - [agent] Name functions and variables with short lower-case words, such as `readrules`, and types with a capital initial, such as `Ruleset`.
        - [agent] Don't use `#ifdef`, because the compilers build the plumber from the same source for every Plan 9 machine.
      - |
        ## Header files

        - [agent] Declare in `plumber.h` every type and function that more than one file uses.
        - [agent] Include `<u.h>` first and `<libc.h>` second in every C file, because `<libc.h>` uses the types that `<u.h>` defines.
        - [agent] Don't include one header file in another, so that each C file lists every header that it depends on.
    blocks:
      - kind: bullet-list
        label: required
        example:
          - '[agent] Return a failure that one message causes as the error of the sender''s write to `send`, such as `no matching plumb rule`, because one bad message mustn''t stop the plumber.'
          - '[agent] Call `sysfatal` only during start-up, before the plumber serves `/mnt/plumb`, because a later call stops the plumber and so closes every port that a program has open.'
          - '[agent] Set the error string with `werrstr` before a function returns -1 or nil, so that the caller can report why the function failed.'

````
