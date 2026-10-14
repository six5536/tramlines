---
type: ErrorReference
id: errors-schema-shapes
title: 'Errors: shapes and templates'
description: The errors that tramlines reports for the shapes of the rules of a schema, and for the holes of its templates.
order: 11
---

# Errors: shapes and templates

These errors come from the shapes of the rules of a schema: the one shape that each rule declares, and the holes of a template. Section 4 of the core specification defines these errors. For a template or a pattern of a format, a message starts with ``format `<format>`: ``, and for a variant, with ``format `<format>`: variant `<variant>`: ``.

## TL_22

A rule of the schema declares more than one shape, or a format declares a shape beside `variants` ([TL_22][sokf:tramlines:tramlines-spec#tl_22]).

### Message

```text
<file>:<line>: error[TL_22]: `<shape>` and `<shape>` beside each other: a rule has one shape
<file>:<line>: error[TL_22]: a shape beside `format`: a rule has one shape, its own or the format's
<file>:<line>: error[TL_22]: `<rule map>` needs exactly one of <keys>
```

### Cause

A shape is `text`, `template`, or `pattern`, and a rule that binds a `format` takes the shape of the format. A rule has more than one shape when it declares two shapes, or a shape beside a `format`. A frontmatter constraint has more than one shape when it declares two shapes. Its `format`, such as `url`, checks the value beside a shape and isn't a shape of its own. The last message reports a format, a variant, or a heading with two shapes, or a format that declares a shape beside `variants`. A format, a variant, or a heading with no shape causes a separate error, [TL_105](#tl_105).

### Fix

Change the schema. Keep one shape on the rule and remove the others. To bind a format, remove the rule's own shape. A format declares either one shape or `variants`.

### Example

This frontmatter constraint declares both a `template` and a `pattern`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${}'
    pattern: '.+'
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
title: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_22]: `template` and `pattern` beside each other: a rule has one shape
1 error, 0 warnings
[exit 1]
```

Keep the `pattern`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    pattern: '.+'
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_105

A format declares no shape and no `variants`, or a variant or a heading rule declares no shape ([TL_105][sokf:tramlines:tramlines-spec#tl_105]).

### Message

```text
<file>:<line>: error[TL_105]: `<rule map>` needs exactly one of <keys>
```

### Cause

A format names the shape of one line. It must declare a `text`, a `template`, or a `pattern`, or else `variants`, each with a shape of its own. A variant, and a heading written as a map, must declare a shape too. The map in the schema declares none of them.

### Fix

Change the schema. Add the shape that the line must have: `text` for fixed text, `template` for text with holes, or `pattern` for a regular expression. A format can declare `variants` instead.

### Example

This format has an example and no shape:

**`note.yaml`**

```yaml
line-limit: 0
formats:
  day:
    example: Monday
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_105]: `format` needs exactly one of `text`, `template`, `pattern`, `variants`
1 error, 0 warnings
[exit 1]
```

Add a pattern:

**`note.yaml`**

```yaml
line-limit: 0
formats:
  day:
    pattern: '^[A-Z][a-z]+day$'
    example: Monday
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_25

A template of the schema contains no hole ([TL_25][sokf:tramlines:tramlines-spec#tl_25]).

### Message

```text
<file>:<line>: error[TL_25]: template `<template>` has no hole; write it as `text`
```

### Cause

A template is text with holes, written `${name:type}`, at the places where the text varies. A template with no hole is fixed text, and fixed text belongs in `text`. The sequence `$${` writes a literal `${`, so a template that contains only `$${` has no hole.

### Fix

Change the schema. Write the fixed text as `text`, or add a hole where the text varies.

### Example

This heading template contains fixed text only:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: Note, example: Note }
    level: 1
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:3: error[TL_25]: template `Note` has no hole; write it as `text`
1 error, 0 warnings
[exit 1]
```

Write the heading as text:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_119

A template of the schema uses one hole name twice ([TL_119][sokf:tramlines:tramlines-spec#tl_119]).

### Message

```text
<file>:<line>: error[TL_119]: template `<template>` names the hole `<name>` twice
```

### Cause

Each named hole of a template captures its own part of the text, and a `key` can name that part. Each name must therefore be unique within the template. A hole with no name, `${}` or `${:type}`, captures nothing and can appear any number of times.

### Fix

Change the schema. Rename one of the holes, or remove the name from a hole that nothing refers to.

### Example

This template names the hole `w` twice:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${w:word} ${w:word}'
    example: Weekly groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
title: Weekly groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_119]: template `${w:word} ${w:word}` names the hole `w` twice
1 error, 0 warnings
[exit 1]
```

Give each hole its own name:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${first:word} ${rest:word}'
    example: Weekly groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_120

A hole of a template names a type that isn't available to the schema ([TL_120][sokf:tramlines:tramlines-spec#tl_120]).

### Message

```text
<file>:<line>: error[TL_120]: template `<template>`: no type `<type>`
```

### Cause

Each hole matches the regular expression of its type: a built-in type such as `slug` or `int`, or a type under `types` in the schema or in the `--formats` file. The type that the hole names is none of these. A misspelt type name often causes this error.

### Fix

Change the schema. Correct the name of the type, use a built-in type, or declare the type under `types`.

### Example

This template uses a type `name` that the schema doesn't declare:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${:name}'
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
title: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_120]: template `${:name}`: no type `name`
1 error, 0 warnings
[exit 1]
```

Declare the type:

**`note.yaml`**

```yaml
line-limit: 0
types:
  name: '[A-Z][a-z]+'
frontmatter:
  title:
    template: '${:name}'
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_121

A hole of a template isn't closed, or its name isn't valid ([TL_121][sokf:tramlines:tramlines-spec#tl_121]).

### Message

```text
<file>:<line>: error[TL_121]: template `<template>` has an unclosed `${`
<file>:<line>: error[TL_121]: template `<template>`: hole name `<name>` must be letters, digits and `_`, not starting with a digit
```

### Cause

A hole opens with `${` and closes with `}`. Its name, the part before the `:`, contains only letters, digits, and `_`, and doesn't start with a digit. To write `${` as text in a template, write `$${`.

### Fix

Change the schema. Close the hole, or rename it with letters, digits, and `_`.

### Example

This hole name contains a hyphen:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${first-word} ${}'
    example: Weekly groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
title: Weekly groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_121]: template `${first-word} ${}`: hole name `first-word` must be letters, digits and `_`, not starting with a digit
1 error, 0 warnings
[exit 1]
```

Use an underscore:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    template: '${first_word} ${}'
    example: Weekly groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-spec#tl_105]: /knowledge/spec/tramlines-spec.md#tl_105
[sokf:tramlines:tramlines-spec#tl_119]: /knowledge/spec/tramlines-spec.md#tl_119
[sokf:tramlines:tramlines-spec#tl_120]: /knowledge/spec/tramlines-spec.md#tl_120
[sokf:tramlines:tramlines-spec#tl_121]: /knowledge/spec/tramlines-spec.md#tl_121
[sokf:tramlines:tramlines-spec#tl_22]: /knowledge/spec/tramlines-spec.md#tl_22
[sokf:tramlines:tramlines-spec#tl_25]: /knowledge/spec/tramlines-spec.md#tl_25
