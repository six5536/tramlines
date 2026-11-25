---
type: ErrorReference
id: errors-examples
title: 'Errors: examples and guidance'
description: The errors that tramlines reports for the examples of a schema, and for guidance that repeats an example.
order: 21
---

# Errors: examples and guidance

These errors come from the examples of a schema: the example of each rule and format, and the `examples` that the schema lists. Sections 4 and 7 of the core specification define them. [A separate page][sokf:errors-example-lists] lists the errors for the `examples` of a section and for an `example` written as a list.

## TL_64

The `example` of a rule fails the rule that declares it ([TL_64][sokf:tramlines:tramlines-spec#tl_64]).

### Message

```text
<file>:<line>: error[TL_64]: the example of `<key>` <reason> (schema <id>, key `<key>`)
<file>:<line>: error[TL_64]: the example `<example>` <reason> (schema <id>, <place of the rule>)
```

### Cause

The example of a rule shows the text that the rule requires, so the command checks it against the rule. The first message reports a frontmatter key. The second reports a heading or a unit rule. The reason names what the example fails, such as the shape, a `const`, an `enum`, a `select-pattern`, a prohibited pattern, or a key that it doesn't capture.

### Fix

In the schema, change the example so that the rule accepts it, or correct the rule if the example shows what you meant.

### Example

The template of this rule starts with `TODO`, and its example doesn't:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: buy milk
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:8: error[TL_64]: the example `buy milk` fails at column 1: expected `TODO `, found `buy milk` (schema tasks, section "Tasks" block 1)
1 error, 0 warnings
[exit 1]
```

Make the example match the template:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_65

The `example` of a format fails the shape, the prohibited patterns, or the key of that format ([TL_65][sokf:tramlines:tramlines-spec#tl_65]).

### Message

```text
<file>:<line>: error[TL_65]: the example `<example>` of format `<name>` <reason>
<file>:<line>: error[TL_65]: the example `<example>` of variant `<variant>` of format `<name>` does not match its own variant: it <reason>
<file>:<line>: error[TL_65]: the example `<example>` of format `<name>` (<layer>) <reason>
```

### Cause

The command checks the example of each format that the schema can use against that format: its shape, its bans, and its key. An example that matches the shape of a format with a key, but captures none, fails too. The command checks the example of a variant against that variant. For a format of a formats file that you pass with `--formats`, the command reports the error on that file, at the line of the format. The message with `(<layer>)` names a format that a program adds when it uses tramlines as a library. The program reports that error on the first line of the schema.

### Fix

In the schema or the formats file that defines the format, change the example so that the format accepts it, or correct the format if the example shows what you meant.

### Example

The template of this format starts with `TODO`, and its example doesn't:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    template: 'TODO ${}'
    example: buy milk
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: task
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:4: error[TL_65]: the example `buy milk` of format `task` fails at column 1: expected `TODO `, found `buy milk`
1 error, 0 warnings
[exit 1]
```

Make the example match the template:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    template: 'TODO ${}'
    example: TODO buy milk
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: task
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_66

A rule declares a shape but no example of it ([TL_66][sokf:tramlines:tramlines-spec#tl_66]).

### Message

```text
<file>:<line>: error[TL_66]: a rule that declares `<key>` carries an `example` of it: a failure shows the example, and the scaffold writes it
<file>:<line>: error[TL_66]: heading `<heading>` holds a hole: write `heading: { template: …, example: … }`, since a template carries an example
```

### Cause

A rule declares a `template`, a `pattern`, or a `source-pattern`, declares no `example`, and binds no format that declares one. A frontmatter `format` such as `url` checks the value and brings no example. The `id` key of the frontmatter is the one exception. The second message reports a `heading` written as a plain string with a hole such as `${:int}`. A plain string leaves no place for an example.

### Fix

Change the schema. Add an `example` to the rule. Write a heading with a hole as a map with `template` and `example`.

### Example

This rule declares a template and no example:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:8: error[TL_66]: a rule that declares `template` carries an `example` of it: a failure shows the example, and the scaffold writes it
1 error, 0 warnings
[exit 1]
```

Add an example:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_68

The `guidance` and the `example` of a rule contain the same words ([TL_68][sokf:tramlines:tramlines-spec#tl_68]).

### Message

```text
<file>:<line>: error[TL_68]: `guidance` and `example` hold the same words: guidance says what the example cannot
```

### Cause

The example shows the text that a rule requires, and the guidance describes what the example can't show. When the two contain the same words, the guidance adds nothing. The command reads the guidance of a section against the example of its heading. It reads the guidance of a block against the example of each of its unit rules. It pairs them this way because the scaffold writes each guidance immediately before that example.

### Fix

Change the schema. Rewrite the guidance to describe what the example can't show, such as when to write the part or what to leave out. If there is nothing to add, remove the guidance.

### Example

The guidance of this block repeats its example:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        guidance: TODO buy milk
        example: TODO buy milk
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:6: error[TL_68]: `guidance` and `example` hold the same words: guidance says what the example cannot
1 error, 0 warnings
[exit 1]
```

Describe what the example can't show:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        guidance: List one task per item, in the order you plan to do them.
        example: TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_69

A document in the `examples` of the schema fails the schema ([TL_69][sokf:tramlines:tramlines-spec#tl_69]).

### Message

```text
<file>:<line>: error[TL_69]: example <n> fails: line <line>: [<KEY>] <message>
<file>:<line>: error[TL_69]: example <n> fails: [<KEY>] <message>
```

### Cause

The command checks each entry of `examples` as a complete document at its own states, with no markers. The message names the number of the example, and the key and message of the error that the document raises. When the error has a line, the message also names the line of the example document that fails. Look up that key to find the rule that the line breaks.

### Fix

Change the schema. Correct the example document so that the schema accepts it, or correct the schema if the example shows what you meant.

### Example

The example document of this schema contains an item without `TODO`:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: TODO buy milk
examples:
  - document: |
      # Tasks

      - buy milk
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:11: error[TL_69]: example 1 fails: line 3: [TL_74] `buy milk` fails at column 1: expected `TODO `, found `buy milk`; an item of this kind reads `TODO buy milk` (schema tasks, section "Tasks" block 1)
1 error, 0 warnings
[exit 1]
```

Correct the example document:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: 'TODO ${}'
          example: TODO buy milk
examples:
  - document: |
      # Tasks

      - TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_127

The key of the example of a rule that declares `key-derives-from` doesn't start with the value that the schema states for its parent followed by the separator ([TL_127][sokf:tramlines:tramlines-spec#tl_127]).

### Message

```text
<file>:<line>: error[TL_127]: the key `<key>` of the example does not start with `<parent>`, the value that the scaffold writes for the frontmatter `<frontmatter key>`, followed by the separator `<separator>` (schema <id>, <place of the rule>)
<file>:<line>: error[TL_127]: the key `<key>` of the example does not start with `<parent>`, a key of an example of set `<set>`, followed by the separator `<separator>` (schema <id>, <place of the rule>)
```

### Cause

A key that derives from a frontmatter value or from the key of another rule starts with that prefix and the separator. The value that the schema states for a frontmatter key is the one that the scaffold writes from the schema: its `const`, else the id that its `id` template renders, else its `example`, else the first value of its `enum` ([TL_129][sokf:tramlines:tramlines-spec#tl_129]). For a rule, it is each key that the examples of the rule capture. A value of a frontmatter key reads in the fixed spelling of its markup, as the key of the example does, and a key of an example of a line of a fence and a key of an example of prose are compared by the text that each shows as prose ([TLMD_287][sokf:tramlines:tramlines-md-spec#tlmd_287]). The scaffold writes its own prefix in place of that value and the separator at the start of the example's key. An example that starts with another prefix contradicts its schema, and the scaffold can't tell which part of the key is the prefix, because a prefix can contain the separator. The first message names the value of a frontmatter key, and the second names the keys of the examples of a rule, each once, joined by `or` when there are several. A key whose value only the host supplies has no value in the schema, so its example is not checked.

### Fix

In the schema, change the example of the rule so that its key starts with the value of its parent and the separator, or change the value that the schema states for the parent.

### Example

The key of this heading derives from the frontmatter `code`, whose example is `CE-FIX`, and the example of the heading starts with `CE-BUG`:

**`bugs.yaml`**

```yaml
line-limit: 0
frontmatter:
  code:
    pattern: '^[A-Z]+(?:-[A-Z]+)*$'
    example: CE-FIX
sections:
  - heading: Bugs
    level: 1
    blocks: []
  - heading:
      pattern: '^(?<id>[A-Z]+(?:-[A-Z]+)*-[0-9]+) .+$'
      key: id
      key-derives-from: { from: code, separator: '-' }
      example: CE-BUG-1 First part
    level: 2
    repeat: true
    blocks: []
```

**`bugs.md`**

```markdown
---
code: CE-BUG
---

# Bugs

## <a id="ce-bug-1"></a>CE-BUG-1 Cache misses on a cold start
```

```console
$ tramlines validate --schema bugs.yaml bugs.md
bugs.yaml:10: error[TL_127]: the key `CE-BUG-1` of the example does not start with `CE-FIX`, the value that the scaffold writes for the frontmatter `code`, followed by the separator `-` (schema bugs, section "/^(?<id>[A-Z]+(?:-[A-Z]+)*-[0-9]+) .+$/")
1 error, 0 warnings
[exit 1]
```

Make the example of `code` the code that the example of the heading starts with:

**`bugs.yaml`**

```yaml
line-limit: 0
frontmatter:
  code:
    pattern: '^[A-Z]+(?:-[A-Z]+)*$'
    example: CE-BUG
sections:
  - heading: Bugs
    level: 1
    blocks: []
  - heading:
      pattern: '^(?<id>[A-Z]+(?:-[A-Z]+)*-[0-9]+) .+$'
      key: id
      key-derives-from: { from: code, separator: '-' }
      example: CE-BUG-1 First part
    level: 2
    repeat: true
    blocks: []
```

```console
$ tramlines validate --schema bugs.yaml bugs.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-example-lists]: /docs/reference/errors-example-lists.md
[sokf:tramlines:tramlines-md-spec#tlmd_287]: /knowledge/spec/tramlines-md-spec.md#tlmd_287
[sokf:tramlines:tramlines-spec#tl_127]: /knowledge/spec/tramlines-spec.md#tl_127
[sokf:tramlines:tramlines-spec#tl_129]: /knowledge/spec/tramlines-spec.md#tl_129
[sokf:tramlines:tramlines-spec#tl_64]: /knowledge/spec/tramlines-spec.md#tl_64
[sokf:tramlines:tramlines-spec#tl_65]: /knowledge/spec/tramlines-spec.md#tl_65
[sokf:tramlines:tramlines-spec#tl_66]: /knowledge/spec/tramlines-spec.md#tl_66
[sokf:tramlines:tramlines-spec#tl_68]: /knowledge/spec/tramlines-spec.md#tl_68
[sokf:tramlines:tramlines-spec#tl_69]: /knowledge/spec/tramlines-spec.md#tl_69
