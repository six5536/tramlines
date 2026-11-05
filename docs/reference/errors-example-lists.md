---
type: ErrorReference
id: errors-example-lists
title: 'Errors: lists of examples'
description: The errors that tramlines reports for the `examples` of a section, and for an `example` written as a list.
order: 22
---

# Errors: lists of examples

These errors come from the examples of a schema that are lists: the `examples` of a section, and the `example` of a block or a unit rule written as a list of texts. Section 9 of the markdown specification defines them. [A separate page][sokf:errors-examples] lists the errors for the example of each rule and format.

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

[sokf:errors-examples]: /docs/reference/errors-examples.md
[sokf:tramlines:tramlines-md-spec#tlmd_180]: /knowledge/spec/tramlines-md-spec.md#tlmd_180
[sokf:tramlines:tramlines-md-spec#tlmd_181]: /knowledge/spec/tramlines-md-spec.md#tlmd_181
[sokf:tramlines:tramlines-md-spec#tlmd_182]: /knowledge/spec/tramlines-md-spec.md#tlmd_182
[sokf:tramlines:tramlines-md-spec#tlmd_183]: /knowledge/spec/tramlines-md-spec.md#tlmd_183
