---
type: ErrorReference
id: errors-lines
title: 'Errors: lines and length'
description: The errors that tramlines reports for a line that fails its shape or a ban, and for a document that is too long.
order: 31
---

# Errors: lines and length

These errors come from the text of a document: a line that doesn't match the shape of its rule, a line that a ban matches, and a document longer than its limit. Sections 3 and 8 of the core specification define these errors.

## TL_21

The document contains more lines than the `line-limit` of its schema allows ([TL_21][sokf:tramlines:tramlines-spec#tl_21]).

### Message

```text
<file>: error[TL_21]: the document has <lines> lines; the limit is <limit>; split it (schema <id>)
```

### Cause

The schema sets `line-limit` to a number greater than zero, and the document is longer. The command doesn't count the marker lines of a scaffold, such as a guidance line or the ends of an `optional` region. It doesn't count the blank line that sets each marker apart either. A `line-limit` of `0` sets no limit.

### Fix

Split the document into shorter documents, or remove what it doesn't need. If documents of this kind need more room, raise `line-limit` in the schema.

### Example

This schema allows three lines:

**`note.yaml`**

```yaml
line-limit: 3
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
```

This document contains five:

**`note.md`**

```markdown
# Note

Buy milk.

Buy bread.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md: error[TL_21]: the document has 5 lines; the limit is 3; split it (schema note)
1 error, 0 warnings
[exit 1]
```

Remove a paragraph:

**`note.md`**

```markdown
# Note

Buy milk and bread.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_74

A line of the document doesn't match the shape that its rule or its format requires ([TL_74][sokf:tramlines:tramlines-spec#tl_74]).

### Message

```text
<file>:<line>: error[TL_74]: `<text>` fails at column <n>: expected <expected>, found <found>; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TL_74]: `<text>` does not match `<pattern>`; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TL_74]: `<text>` must read `<expected>` (schema <id>, <place of the rule>)
<file>:<line>: error[TL_74]: `<text>` does not match the `<variant>` form: it <failure>; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TL_74]: `<text>` matches none of the forms <variants>[: each fails at column <n>]; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
```

### Cause

The rule declares a `template`, a `pattern`, or a `text`, or binds a format, and the line doesn't match it. For a template, the message gives the column where the line stops matching and what the template expects there. A line that lacks only the end of the template stops matching at its own end. When a hole doesn't take the value that the line gives it, the message names the column where the value starts. For a format with `variants`, the message names the variant that matches the line furthest, or every such variant when several stop matching at the same column. The message ends with the example of the shape, when the rule declares one, or of each variant that it names. `<unit>` is what the line is: an item, a paragraph, a row, or a line of a fence. Brackets mark a part that appears only sometimes.

### Fix

Rewrite the line in the document to match the example. If the rule is wrong for documents of this kind, correct its shape in the schema.

### Example

This rule requires each item to start with `TODO`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: TODO buy milk
```

This item doesn't:

**`note.md`**

```markdown
# Note

- buy milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TL_74]: `buy milk` fails at column 1: expected `TODO `, found `buy milk`; an item of this kind reads TODO buy milk (schema note, section "Note" block 1)
1 error, 0 warnings
[exit 1]
```

Start the item with `TODO`:

**`note.md`**

```markdown
# Note

- TODO buy milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_75

A line of the document matches one of the `prohibited-patterns` of the schema or of its rule ([TL_75][sokf:tramlines:tramlines-spec#tl_75]).

### Message

```text
<file>:<line>: error[TL_75]: `<span>` is refused: <message> (schema <id>)
<file>:<line>: error[TL_75]: `<span>` is refused: <message>[ A <unit> of this kind reads <example>] (schema <id>, <place of the rule>)
```

### Cause

The schema, or the rule that applies to the line, declares `prohibited-patterns`, and the line matches one of them. The finding quotes the part of the line that matched. It also gives the message of the ban, which states the ban's rule. The first message reports a schema-wide pattern, which applies to every line of the body outside code. The second message reports a pattern on a rule or a format. It ends with the example of the rule when the rule has one, and the brackets in the message mark that part.

### Fix

Rewrite the line in the document so that it doesn't match the pattern. If the pattern refuses text that documents of this kind need, correct it in the schema.

### Example

This schema refuses `TBD` anywhere in the body:

**`note.yaml`**

```yaml
line-limit: 0
prohibited-patterns: [{pattern: '\bTBD\b', message: 'A note decides everything that it states.'}]
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
```

This document leaves a gap:

**`note.md`**

```markdown
# Note

Buy TBD.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TL_75]: `TBD` is refused: A note decides everything that it states. (schema note)
1 error, 0 warnings
[exit 1]
```

Fill the gap:

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-spec#tl_21]: /knowledge/spec/tramlines-spec.md#tl_21
[sokf:tramlines:tramlines-spec#tl_74]: /knowledge/spec/tramlines-spec.md#tl_74
[sokf:tramlines:tramlines-spec#tl_75]: /knowledge/spec/tramlines-spec.md#tl_75
