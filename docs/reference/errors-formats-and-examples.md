---
type: ErrorReference
id: errors-formats-and-examples
title: 'Errors: formats'
description: The errors that tramlines reports for the line formats of a schema, and how to fix each one.
order: 20
---

# Errors: formats

These errors concern the line formats that a schema can use. They also cover a line that lacks the key captured by its rule or format. Section 5 of the core specification defines them. [A separate page][sokf:errors-examples] lists the errors for the examples of a schema.

## TL_40

A format with one shape declares no `example` ([TL_40][sokf:tramlines:tramlines-spec#tl_40]).

### Message

```text
<file>:<line>: error[TL_40]: format `<name>`: a format with one shape carries an `example`
```

### Cause

A format with one shape (a `text`, a `template`, or a `pattern`) declares no `example`. The engine shows the example beside every failure of the shape, so each such format needs one. A patch, an entry with `patch: true`, needs no example of its own. You can define a format in the `formats` map of the schema or in a formats file that you pass with `--formats`. For a format in a formats file, the command stops with the same key and message.

### Fix

Add an `example` to the format in the schema or in the formats file.

### Example

This schema defines a format `task` with no example:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    template: 'TODO ${}'
sections:
  - heading: Tasks
    level: 1
    blocks: []
```

**`tasks.md`**

```markdown
# Tasks
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:4: error[TL_40]: format `task`: a format with one shape carries an `example`
1 error, 0 warnings
[exit 1]
```

Add an example of the format:

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
    blocks: []
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_110

A format with `variants` declares an `example` of its own ([TL_110][sokf:tramlines:tramlines-spec#tl_110]).

### Message

```text
<file>:<line>: error[TL_110]: format `<name>`: a format with `variants` has no `example` of its own: each variant's example speaks for its shape
```

### Cause

Each variant of a format has a shape and an example of its own. A failure shows the example of the variant that the line comes nearest to. An `example` beside `variants` belongs to no shape. For a format in a formats file that you pass with `--formats`, the command stops with the same key and message.

### Fix

Remove the `example` from the format. If the example shows a line that no variant accepts, add a variant with that example.

### Example

This format declares an example beside its variants:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    example: TODO buy milk
    variants:
      open:
        template: 'TODO ${}'
        example: TODO buy milk
      done:
        template: 'DONE ${}'
        example: DONE buy bread
sections:
  - heading: Tasks
    level: 1
    blocks: []
```

**`tasks.md`**

```markdown
# Tasks
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:4: error[TL_110]: format `task`: a format with `variants` has no `example` of its own: each variant's example speaks for its shape
1 error, 0 warnings
[exit 1]
```

Remove the example of the format:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    variants:
      open:
        template: 'TODO ${}'
        example: TODO buy milk
      done:
        template: 'DONE ${}'
        example: DONE buy bread
sections:
  - heading: Tasks
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_41

A format entry declares `patch: true`, but no farther layer defines a format of that name for it to patch ([TL_41][sokf:tramlines:tramlines-spec#tl_41]).

### Message

```text
<file>:<line>: error[TL_41]: format `<name>` patches no farther entry
```

### Cause

A patch states only how it differs from the entry that it shadows. The `formats` map of a schema patches a format of a formats file that you pass with `--formats`. When you don't pass that file, or the file has no format of that name, the patch has nothing to merge onto. A patch in the formats file itself has no farther layer, and the command stops with the same key and message.

### Fix

Change the schema or the command. Pass the formats file that defines the entry with `--formats`. If no such file exists, remove `patch: true` and write the whole format in the schema. For a patch in the formats file, remove `patch: true` there and write the whole format.

### Example

This schema patches a format `task` that nothing defines:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    patch: true
    example: TODO buy bread
sections:
  - heading: Tasks
    level: 1
    blocks: []
```

**`tasks.md`**

```markdown
# Tasks
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:4: error[TL_41]: format `task` patches no farther entry
1 error, 0 warnings
[exit 1]
```

Write the whole format:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    template: 'TODO ${}'
    example: TODO buy bread
sections:
  - heading: Tasks
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

## TL_97

A rule binds a format by a name that no layer declares ([TL_97][sokf:tramlines:tramlines-spec#tl_97]).

### Message

```text
<file>:<line>: error[TL_97]: `format` names no format `<name>`
```

### Cause

A rule sets `format` to a name that is neither in the `formats` map of the schema nor in a formats file that you pass with `--formats`. A format of the schema can fail to load, for example when it has no `example`. That format has its own error, and the rules that bind it report nothing more.

### Fix

Change the schema or the command. Correct the name, define the format in the `formats` map of the schema, or pass the formats file that defines it with `--formats`.

### Example

This schema binds a format `task` that it doesn't define:

**`tasks.yaml`**

```yaml
line-limit: 0
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
tasks.yaml:8: error[TL_97]: `format` names no format `task`
1 error, 0 warnings
[exit 1]
```

Define the format:

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

## TL_43

The example of one variant of a format also matches another variant of the same format ([TL_43][sokf:tramlines:tramlines-spec#tl_43]).

### Message

```text
<file>:<line>: error[TL_43]: the example of variant `<variant>` of format `<name>` also matches variant `<other>`: each variant's example matches its own variant only
<file>:<line>: error[TL_43]: the example of variant `<variant>` of format `<name>` (<layer>) also matches variant `<other>`: each variant's example matches its own variant only
```

### Cause

The example of a variant distinguishes that variant from the others, so the example must match its own variant and no other. A variant with a broad shape, such as `${}`, matches the examples of the other variants too. The command reports an example that doesn't match its own variant as [TL_65][sokf:errors-examples#tl_65] instead. For a format of a formats file that you pass with `--formats`, the command reports the error on that file, at the line of the format. The message with `(<layer>)` names a format that a program adds when it uses tramlines as a library. The program reports that error on the first line of the schema.

### Fix

Change the schema or the formats file. Narrow the shape of the variant that matches too much, or change the example so that only its own variant matches it.

### Example

The variant `any` of this format matches every line, so it matches the example of `open`:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    variants:
      open:
        template: 'TODO ${}'
        example: TODO buy milk
      any:
        template: '${}'
        example: buy bread
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
tasks.yaml:6: error[TL_43]: the example of variant `open` of format `task` also matches variant `any`: each variant's example matches its own variant only
1 error, 0 warnings
[exit 1]
```

Narrow the second variant to lines that start with `DONE`:

**`tasks.yaml`**

```yaml
line-limit: 0
formats:
  task:
    variants:
      open:
        template: 'TODO ${}'
        example: TODO buy milk
      done:
        template: 'DONE ${}'
        example: DONE buy bread
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

## TL_44

A line matches the shape of its rule but lacks the key that the rule or its format captures ([TL_44][sokf:tramlines:tramlines-spec#tl_44]).

### Message

```text
<file>:<line>: error[TL_44]: `<text>` carries no key: `<key>` captures nothing; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TL_44]: `<text>` carries no key: the format `<name>` captures `<key>` (schema <id>, <place of the rule>)
```

### Cause

The rule declares `key`, or binds a format that declares one. The shape makes the capture group of the key optional, as in a `pattern` with `(?<id>…)?`. The line matches the shape but contains no key. A rule that binds a format and declares its own `key` reads the key in the shape of the format. The second message reports a key that the format declares.

### Fix

Add the key to the line in the document. If the key is optional in lines of this kind, set `key-optional: true` on the rule in the schema.

### Example

This rule captures an optional key `id` at the start of each item:

**`tasks.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        each:
          pattern: '^(?:(?<id>T-\d+): )?.+$'
          key: id
          example: 'T-1: buy milk'
```

This item has no key:

**`tasks.md`**

```markdown
# Tasks

- buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.md:3: error[TL_44]: `buy milk` carries no key: `id` captures nothing; an item of this kind reads T-1: buy milk (schema tasks, section "Tasks" block 1)
1 error, 0 warnings
[exit 1]
```

Add the key, and the anchor that must precede every key:

**`tasks.md`**

```markdown
# Tasks

- <a id="t-1"></a>T-1: buy milk
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-examples]: /docs/reference/errors-examples.md
[sokf:errors-examples#tl_65]: /docs/reference/errors-examples.md#tl_65
[sokf:tramlines:tramlines-spec#tl_110]: /knowledge/spec/tramlines-spec.md#tl_110
[sokf:tramlines:tramlines-spec#tl_40]: /knowledge/spec/tramlines-spec.md#tl_40
[sokf:tramlines:tramlines-spec#tl_41]: /knowledge/spec/tramlines-spec.md#tl_41
[sokf:tramlines:tramlines-spec#tl_43]: /knowledge/spec/tramlines-spec.md#tl_43
[sokf:tramlines:tramlines-spec#tl_44]: /knowledge/spec/tramlines-spec.md#tl_44
[sokf:tramlines:tramlines-spec#tl_97]: /knowledge/spec/tramlines-spec.md#tl_97
