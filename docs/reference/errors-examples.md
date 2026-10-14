---
type: ErrorReference
id: errors-examples
title: 'Errors: examples and guidance'
description: The errors that tramlines reports for the examples of a schema, and for guidance that repeats an example.
order: 21
---

# Errors: examples and guidance

These errors come from the examples of a schema: the example of each rule and format, the `examples` that the schema lists, and the `examples` of a section. Section 7 of the core specification defines the `TL` errors, and section 9 of the markdown specification defines the `TLMD` errors.

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
tasks.yaml:11: error[TL_69]: example 1 fails: line 3: [TL_74] `buy milk` fails at column 1: expected `TODO `, found `buy milk`; an item of this kind reads TODO buy milk (schema tasks, section "Tasks" block 1)
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

## TLMD_180

A section declares `examples`, but it doesn't repeat ([TLMD_180][sokf:tramlines:tramlines-md-spec#tlmd_180]).

### Message

```text
<file>:<line>: error[TLMD_180]: `examples` holds further occurrences of a section, so the section declares `repeat: true`
```

### Cause

The `examples` of a section are further occurrences of the section. The scaffold writes them after the occurrence that it derives. Only a section that repeats can occur more than once.

### Fix

In the schema, declare `repeat: true` on the section, or remove `examples`.

### Example

This section lists a further occurrence but doesn't repeat:

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 1
    blocks: []
  - heading: { template: '${}', example: Milk }
    level: 2
    examples:
      - |
        ## Bread

        Buy a loaf.
    blocks:
      - kind: prose
        example: Buy two litres.
```

**`notes.md`**

```markdown
# Notes

## Milk

Buy two litres.
```

```console
$ tramlines validate --schema notes.yaml notes.md
notes.yaml:9: error[TLMD_180]: `examples` holds further occurrences of a section, so the section declares `repeat: true`
1 error, 0 warnings
[exit 1]
```

Let the section repeat:

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 1
    blocks: []
  - heading: { template: '${}', example: Milk }
    level: 2
    repeat: true
    examples:
      - |
        ## Bread

        Buy a loaf.
    blocks:
      - kind: prose
        example: Buy two litres.
```

```console
$ tramlines validate --schema notes.yaml notes.md
0 errors, 0 warnings
```

## TLMD_181

A text of the `examples` of a section doesn't begin with a heading of the level of the section ([TLMD_181][sokf:tramlines:tramlines-md-spec#tlmd_181]).

### Message

```text
<file>:<line>: error[TLMD_181]: item <n> of `examples` begins with `<line>`, but each occurrence begins with a heading of level <level>
```

### Cause

The scaffold writes each text of `examples` as a whole occurrence of the section, so the text starts with the heading of that occurrence.

### Fix

In the schema, start the text with a heading of the level of the section.

### Example

The further occurrence of this section starts with its body:

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 1
    blocks: []
  - heading: { template: '${}', example: Milk }
    level: 2
    repeat: true
    examples:
      - Buy a loaf.
    blocks:
      - kind: prose
        example: Buy two litres.
```

**`notes.md`**

```markdown
# Notes

## Milk

Buy two litres.
```

```console
$ tramlines validate --schema notes.yaml notes.md
notes.yaml:10: error[TLMD_181]: item 1 of `examples` begins with `Buy a loaf.`, but each occurrence begins with a heading of level 2
1 error, 0 warnings
[exit 1]
```

Start the text with its heading:

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 1
    blocks: []
  - heading: { template: '${}', example: Milk }
    level: 2
    repeat: true
    examples:
      - |
        ## Bread

        Buy a loaf.
    blocks:
      - kind: prose
        example: Buy two litres.
```

```console
$ tramlines validate --schema notes.yaml notes.md
0 errors, 0 warnings
```

## TLMD_182

The `example` of a block or a unit rule is an empty list ([TLMD_182][sokf:tramlines:tramlines-md-spec#tlmd_182]).

### Message

```text
<file>:<line>: error[TLMD_182]: `example` is an empty list: write at least one text, or leave `example` out
```

### Cause

The scaffold writes one unit for each text of a list `example`. An empty list gives the scaffold nothing to write, and a failure has no example to show.

### Fix

In the schema, write at least one text in the list, or remove `example`.

### Example

The example of this rule is an empty list:

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
          example: []
```

**`tasks.md`**

```markdown
# Tasks

- TODO buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:9: error[TLMD_182]: `example` is an empty list: write at least one text, or leave `example` out
1 error, 0 warnings
[exit 1]
```

Write the texts of the list:

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
          example:
            - TODO buy milk
            - TODO call the plumber
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TLMD_183

A `code` block that doesn't repeat carries a list `example` of more than one text ([TLMD_183][sokf:tramlines:tramlines-md-spec#tlmd_183]).

### Message

```text
<file>:<line>: error[TLMD_183]: a list `example` on a `code` block writes one fence for each text, so the block declares `repeat: true`
```

### Cause

The scaffold writes each text of a list `example` of a `code` block as one fence of its own. A block that doesn't repeat holds one fence.

### Fix

In the schema, declare `repeat: true` on the block, or give the block one text.

### Example

This block lists two fences but doesn't repeat:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks:
      - kind: code
        lang: sh
        example:
          - make bread
          - make toast
```

**`recipe.md`**

````markdown
# Recipe

```sh
make bread
```
````

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.yaml:6: error[TLMD_183]: a list `example` on a `code` block writes one fence for each text, so the block declares `repeat: true`
1 error, 0 warnings
[exit 1]
```

Let the block repeat:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks:
      - kind: code
        lang: sh
        repeat: true
        example:
          - make bread
          - make toast
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_180]: /knowledge/spec/tramlines-md-spec.md#tlmd_180
[sokf:tramlines:tramlines-md-spec#tlmd_181]: /knowledge/spec/tramlines-md-spec.md#tlmd_181
[sokf:tramlines:tramlines-md-spec#tlmd_182]: /knowledge/spec/tramlines-md-spec.md#tlmd_182
[sokf:tramlines:tramlines-md-spec#tlmd_183]: /knowledge/spec/tramlines-md-spec.md#tlmd_183
[sokf:tramlines:tramlines-spec#tl_64]: /knowledge/spec/tramlines-spec.md#tl_64
[sokf:tramlines:tramlines-spec#tl_65]: /knowledge/spec/tramlines-spec.md#tl_65
[sokf:tramlines:tramlines-spec#tl_66]: /knowledge/spec/tramlines-spec.md#tl_66
[sokf:tramlines:tramlines-spec#tl_68]: /knowledge/spec/tramlines-spec.md#tl_68
[sokf:tramlines:tramlines-spec#tl_69]: /knowledge/spec/tramlines-spec.md#tl_69
