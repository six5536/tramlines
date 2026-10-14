---
type: Schema
id: schema-tramlines-md-specification
title: tramlines-md specification schema
description: The one document that specifies the markdown format of tramlines, with its rules as EARS criteria keyed TLMD_n and its fixed texts in this schema.
---

<!-- prettier-ignore -->
````yaml
line-limit: 0

guidance: >
  This schema governs the specification of the markdown format of tramlines, the one
  document of this type. Each normative rule is a criterion in the EARS form, with a key
  TLMD_n. A new criterion takes the next number, and the number of a removed criterion is
  never used again. The subject of a criterion is the part that has to conform, and each
  subject is a term of section 1. All other text is informative. The document has no
  table, and a named list takes the place of one.

frontmatter:
  type:
    const: TramlinesMdSpecification
  id:
    const: tramlines-md-spec
  title:
    const: tramlines-md markdown format specification
  description:
    guidance: The description says what the specification defines.
    example: 'The markdown format of tramlines: how the engine reads a markdown document, what its schema specifies about the body, its keys and anchors, its scaffold, and its repairs.'
  version:
    template: '${:int}.${:int}'
    example: "0.1"
    guidance: The version is the version of the format that the document defines.
  status:
    enum: [draft, stable]
    guidance: "The status is `draft` while the version is open to change, and `stable` after the version is fixed."

types:
  tlmd-rule-id: 'TLMD_\d+'
  tlmd-subject:
    - engine
    - host
  tlmd-property:
    - invariant
    - equivalent
    - idempotent

formats:
  tlmd-criterion:
    key: id
    variants:
      ubiquitous:
        template: '${id:tlmd-rule-id} [ubiquitous]: the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_1 [ubiquitous]: the engine SHALL read a schema before any document that it governs."
      event:
        template: '${id:tlmd-rule-id} [event]: WHEN ${}, the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_2 [event]: WHEN a schema names a format, the engine SHALL resolve the format through the registries."
      state:
        template: '${id:tlmd-rule-id} [state]: WHILE ${}, the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_3 [state]: WHILE a document has no frontmatter, the engine SHALL read its states as empty."
      unwanted:
        template: '${id:tlmd-rule-id} [unwanted]: IF ${}, THEN the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_4 [unwanted]: IF a schema contains a YAML tag, THEN the engine SHALL report a finding on the schema."
      optional:
        template: '${id:tlmd-rule-id} [optional]: WHERE ${}, the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_5 [optional]: WHERE a rule declares `only`, the engine SHALL apply the rule only in those states."
      complex-event:
        template: '${id:tlmd-rule-id} [complex]: WHILE ${}, WHEN ${}, the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_6 [complex]: WHILE a rule is gated, WHEN the scaffold contains its part, the engine SHALL write the marker of the gate."
      complex-unwanted:
        template: '${id:tlmd-rule-id} [complex]: WHILE ${}, IF ${}, THEN the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_7 [complex]: WHILE a rule is gated, IF its part has no marker, THEN the engine SHALL report a finding on the schema."
      property:
        template: '${id:tlmd-rule-id} [${:tlmd-property}]: FOR ANY ${}, the ${:tlmd-subject} SHALL ${}.'
        example: "TLMD_8 [invariant]: FOR ANY line, the engine SHALL report a span that is a substring of the line."
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
          - text: "This specification defines tramlines-md: the markdown format of tramlines, what a schema specifies about the body of a markdown document, and how the engine checks it."
            max: 1
          - pattern: '.'
            min: 0
            prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
            example: 'The core specification defines what every format shares.'
  - heading: About this specification
    level: 2
    guidance: This section says how to read the specification. Its text is fixed.
    blocks:
      - kind: prose
        max: 1
        each:
          text: "Each normative rule of this specification is an EARS criterion, keyed `TLMD_n`."
  - heading: "1. Terminology"
    level: 2
    rule-id: section-1
    guidance: >
      Each item defines one term: its name in backticks, then what it is. The subject of each
      criterion is one of these terms, and the schema lists them as the type tlmd-subject.
    blocks:
      - kind: bullet-list
        each:
          template: '`${name}` - ${}'
          key: name
          prohibited-patterns: [{pattern: '\b(SHALL|MUST|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL|[Ss]hall|[Mm]ust|[Ss]hould|[Mm]ay)\b', message: 'The prose of a specification uses no requirement keyword, such as SHALL or must, because only a criterion states a requirement.'}]
          example: '`engine` - The library that checks a document against a schema.'
  - heading: "2. Documents"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_20 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "3. Dialect and markers"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_30 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "4. Sections and headings"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_40 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "5. Blocks"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_50 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "6. Units and lines"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_60 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "7. Keys and anchors"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_70 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "8. Checking"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_80 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "9. Scaffolds"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_90 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
  - heading: "10. Repairs"
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
          - format: tlmd-criterion
            unique: repository
            min: 0
            example: 'TLMD_100 [ubiquitous]: the engine SHALL read a schema before any document that it governs.'
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
