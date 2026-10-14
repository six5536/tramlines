---
type: Schema
id: schema-tramlines-specification
title: tramlines specification schema
description: The one document that specifies tramlines, with its rules as EARS criteria keyed TL_n and its fixed texts in this schema.
---

<!-- prettier-ignore -->
````yaml
line-limit: 0

guidance: >
  This schema governs the specification of tramlines, the one document of this type. Each
  normative rule is a criterion in the EARS form, with a key TL_n. A new criterion takes
  the next number, and the number of a removed criterion is never used again. The subject
  of a criterion is the part that has to conform, and each subject is a term of section 1.
  All other text is informative. The document has no table, and a named list takes the
  place of one.

frontmatter:
  type:
    const: TramlinesSpecification
  id:
    const: tramlines-spec
  title:
    const: tramlines core specification
  description:
    guidance: The description says what the specification defines.
    example: The grammar that a schema uses, how the engine builds a grammar from layers, and how the engine checks a document against a schema in any format.
  version:
    template: '${:int}.${:int}'
    example: "0.1"
    guidance: The version is the version of the format that the document defines.
  status:
    enum: [draft, stable]
    guidance: "The status is `draft` while the version is open to change, and `stable` after the version is fixed."

types:
  tl-rule-id: 'TL_\d+'
  tl-subject:
    - engine
    - grammar
    - core grammar
    - format grammar
    - host extension
    - host
    - command
  tl-property:
    - invariant
    - equivalent
    - idempotent

formats:
  tl-criterion:
    key: id
    variants:
      ubiquitous:
        template: '${id:tl-rule-id} [ubiquitous]: the ${:tl-subject} SHALL ${}.'
        example: "TL_1 [ubiquitous]: the engine SHALL read a schema before any document that it governs."
      event:
        template: '${id:tl-rule-id} [event]: WHEN ${}, the ${:tl-subject} SHALL ${}.'
        example: "TL_2 [event]: WHEN a schema names a format, the engine SHALL resolve the format through the registries."
      state:
        template: '${id:tl-rule-id} [state]: WHILE ${}, the ${:tl-subject} SHALL ${}.'
        example: "TL_3 [state]: WHILE a document has no frontmatter, the engine SHALL read its states as empty."
      unwanted:
        template: '${id:tl-rule-id} [unwanted]: IF ${}, THEN the ${:tl-subject} SHALL ${}.'
        example: "TL_4 [unwanted]: IF a schema contains a YAML tag, THEN the engine SHALL report a finding on the schema."
      optional:
        template: '${id:tl-rule-id} [optional]: WHERE ${}, the ${:tl-subject} SHALL ${}.'
        example: "TL_5 [optional]: WHERE a rule declares `only`, the engine SHALL apply the rule only in those states."
      complex-event:
        template: '${id:tl-rule-id} [complex]: WHILE ${}, WHEN ${}, the ${:tl-subject} SHALL ${}.'
        example: "TL_6 [complex]: WHILE a rule is gated, WHEN the scaffold contains its part, the engine SHALL write the marker of the gate."
      complex-unwanted:
        template: '${id:tl-rule-id} [complex]: WHILE ${}, IF ${}, THEN the ${:tl-subject} SHALL ${}.'
        example: "TL_7 [complex]: WHILE a rule is gated, IF its part has no marker, THEN the engine SHALL report a finding on the schema."
      property:
        template: '${id:tl-rule-id} [${:tl-property}]: FOR ANY ${}, the ${:tl-subject} SHALL ${}.'
        example: "TL_8 [invariant]: FOR ANY line, the engine SHALL report a span that is a substring of the line."
    prohibited-patterns:
      - pattern: '\b([Ss]hould|[Mm]ust|[Ww]ill|[Mm]ay|[Mm]ight|[Cc]an)\b'
        message: 'A criterion uses no weak modal verb, such as should, must, will, may, might, or can.'
      - pattern: '\bSHALL\b.*\bSHALL\b'
        message: 'A criterion holds one SHALL.'
      - pattern: '\b([Ff]ast|[Qq]uickly|[Ee]asy|[Ee]asily|[Uu]ser-friendly|[Aa]ppropriate|[Aa]s needed)\b'
        message: 'A criterion uses no vague word, such as fast, easy, appropriate, or as needed.'
      - pattern: '\betc\.'
        message: 'A criterion ends no list with etc.'
      - pattern: 'and/or'
        message: 'A criterion uses no and/or.'
      - pattern: '\bSHALL (NOT )?be (\w+ed|\w+[^p\W]en|\w+wn|sent|kept|built|left|lost|made|read|set|put|held|told|found|sold|run|done|shut|meant|spent|paid|laid|said|brought|bought|caught|taught|thought)\b'
        message: 'A criterion states its response in the active voice.'

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      The opening says what the specification defines, in the fixed sentence. The sentences
      that follow say how the specification relates to the other specification.
    blocks:
      - kind: prose
        rules:
          - text: "This specification defines tramlines: the grammar that a schema uses, and how the engine checks a document against a schema, whatever the format of the document."
            max: 1
          - pattern: '.'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: 'A format grammar adds what the documents of one format contain.'
  - heading: About this specification
    level: 2
    guidance: This section says how to read the specification. Its text is fixed.
    blocks:
      - kind: prose
        max: 1
        each:
          text: "Each normative rule of this specification is an EARS criterion, keyed `TL_n`."
  - heading: "1. Terminology"
    level: 2
    rule-id: section-1
    guidance: >
      Each item defines one term: its name in backticks, then what it is. The subject of each
      criterion is one of these terms, and the schema lists them as the type tl-subject.
    blocks:
      - kind: bullet-list
        each:
          template: '`${name}` - ${}'
          key: name
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: '`engine` - The library that checks a document against a schema.'
  - heading: "2. Grammars"
    level: 2
    rule-id: section-2
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_20 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "3. Schemas"
    level: 2
    rule-id: section-3
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_30 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "4. Shapes and types"
    level: 2
    rule-id: section-4
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_40 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "5. Formats"
    level: 2
    rule-id: section-5
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_50 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "6. Frontmatter and lifecycle"
    level: 2
    rule-id: section-6
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_60 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "7. Examples and guidance"
    level: 2
    rule-id: section-7
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_70 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "8. Findings"
    level: 2
    rule-id: section-8
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_80 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "9. Repairs"
    level: 2
    rule-id: section-9
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_90 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
  - heading: "10. The command"
    level: 2
    rule-id: section-10
    blocks-ordered: false
    blocks:
      - kind: prose
        optional: true
        repeat: true
        guidance: The prose is informative and explains a rule. A sentence that says what has to be true is a criterion, not prose.
        each:
          pattern: '.'
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: 'The engine reads a schema before any document that it governs.'
      - kind: bullet-list
        optional: true
        repeat: true
        guidance: >
          A list holds criteria, or it is a named list in place of a table. A named list
          describes one thing in each item: its name in backticks, then what it is. A sentence
          before the list says what the list contains. Two lists in a row read as one list, so
          a paragraph or an example separates them.
        rules:
          - format: tl-criterion
            unique: repository
            min: 0
            example: 'TL_100 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
          - template: '`${}` - ${}'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: '`text` - The exact text of a unit.'
      - kind: code
        optional: true
        repeat: true
        lang: any
        guidance: A code block is an example. It shows a rule and adds none.
        example: "line-limit: 0"
````
