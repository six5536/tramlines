---
type: ErrorReference
id: errors-lifecycle-and-ids
title: 'Errors: lifecycle and ids'
description: The errors that tramlines reports for the lifecycle of a schema, for the rules that it gates, and for the id template.
order: 13
---

# Errors: lifecycle and ids

These errors come from the lifecycle of a schema, from the `only` of a rule, and from the template of the `id` key. Sections 3 and 6 of the core specification define these errors.

## TL_20

The `only` of a rule names a lifecycle dimension or a state that the schema doesn't declare ([TL_20][sokf:tramlines:tramlines-spec#tl_20]).

### Message

```text
<file>:<line>: error[TL_20]: `only` names a dimension the lifecycle does not declare: `<dimension>`
<file>:<line>: error[TL_20]: `only` names `<state>`, which is no state of `<dimension>`
```

### Cause

The key `only` restricts a rule to some states of a document. Each key of `only` is the `key` of a dimension under `lifecycle`, and each value is one of the `states` of that dimension.

### Fix

Change the schema. Correct the dimension or the state in `only`, or add it to the `lifecycle`.

### Example

This rule names a state `drafted`, which the lifecycle doesn't declare:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [drafted] }
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note

## Open questions
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TL_20]: `only` names `drafted`, which is no state of `status`
1 error, 0 warnings
[exit 1]
```

Name the state as the lifecycle does:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_111

A rule declares `only`, and the schema declares no lifecycle ([TL_111][sokf:tramlines:tramlines-spec#tl_111]).

### Message

```text
<file>:<line>: error[TL_111]: `only` without a lifecycle
```

### Cause

The key `only` restricts a rule to some states of a document, and the `lifecycle` of a schema declares those states. A schema without a `lifecycle` declares no states for `only` to name.

### Fix

Change the schema. Declare the dimension that `only` names under `lifecycle`, or remove `only` from the rule.

### Example

This schema gates a section on `status`, but declares no lifecycle:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note

## Open questions
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_111]: `only` without a lifecycle
1 error, 0 warnings
[exit 1]
```

Declare the lifecycle:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_58

The template of the `id` key contains a hole that the command can't fill for a new document ([TL_58][sokf:tramlines:tramlines-spec#tl_58]).

### Message

```text
<file>:<line>: error[TL_58]: template `<template>`: an id holds only the holes a new document is filled with, `${:seq}`, `${:slug}` and `${<lifecycle key>}`, not a `<type>`
<file>:<line>: error[TL_58]: template `<template>`: the hole `${<name>}` names no lifecycle key
```

### Cause

The command also fills the template of `id` to write the `id` of a new document. The template therefore contains only the holes that the command can fill: `${:seq}`, `${:slug}`, and a hole with no type named for a key of the `lifecycle`.

### Fix

Change the schema. Replace a hole of another type with `${:seq}` or `${:slug}`. For a hole with no type, name a key that `lifecycle` declares, or declare that key under `lifecycle`.

### Example

This `id` template numbers documents with an `int` hole:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:int}'
    example: note-1
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
id: note-1
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_58]: template `note-${:int}`: an id holds only the holes a new document is filled with, `${:seq}`, `${:slug}` and `${<lifecycle key>}`, not an `int`
1 error, 0 warnings
[exit 1]
```

Number them with a `seq` hole, and set `seq-width` to the number of digits (the default is three):

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}'
    seq-width: 1
    example: note-1
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_114

The template of the `id` key contains more than one `seq` hole ([TL_114][sokf:tramlines:tramlines-spec#tl_114]).

### Message

```text
<file>:<line>: error[TL_114]: template `<template>` has more than one `seq` hole
```

### Cause

The command fills the `seq` hole of the `id` template with the next free number when it writes a new document. A template with two `seq` holes has no single number to fill.

### Fix

Change the schema. Keep one `seq` hole, and write the other part of the id with another hole, such as `${:slug}`.

### Example

This `id` template contains two `seq` holes:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}-${:seq}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
id: note-001-002
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_114]: template `note-${:seq}-${:seq}` has more than one `seq` hole
1 error, 0 warnings
[exit 1]
```

Write the second part as a slug:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}-${:slug}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_115

The `id` key declares a template and a key beside it that `id` doesn't accept ([TL_115][sokf:tramlines:tramlines-spec#tl_115]).

### Message

```text
<file>:<line>: error[TL_115]: `<key>` beside the `id` template: `id` takes `template` alone
```

### Cause

The template of `id` defines every id that a document can have, and the id that the command writes for a new document. A `const`, `enum`, `format`, `list`, `min`, or `max` beside it would accept ids that the template writes differently.

### Fix

Change the schema. Remove the key beside the template, and write what it says into the template.

### Example

This `id` declares an `enum` beside its template:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}'
    enum: [note-001, note-002]
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
id: note-001
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:5: error[TL_115]: `enum` beside the `id` template: `id` takes `template` alone
1 error, 0 warnings
[exit 1]
```

Remove the `enum`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_116

The key `seq-width` sits beside the template of a key other than `id`, or beside an `id` template with no `seq` hole ([TL_116][sokf:tramlines:tramlines-spec#tl_116]).

### Message

```text
<file>:<line>: error[TL_116]: `seq-width` beside a template with no `seq` hole
<file>:<line>: error[TL_116]: `seq-width` applies to the `id` template alone
```

### Cause

The key `seq-width` sets the number of digits of the `seq` hole of the `id` template. Beside a template with no `seq` hole, or beside the template of another key, `seq-width` has no hole to size. For a `seq-width` beside no template at all, the command reports [TL_103][sokf:errors-schema#tl_103] instead, because the key requires `template` beside it.

### Fix

Change the schema. Remove `seq-width`, or add a `seq` hole to the `id` template.

### Example

This `id` template has no `seq` hole, and declares `seq-width`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:slug}'
    seq-width: 3
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
id: note-groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_116]: `seq-width` beside a template with no `seq` hole
1 error, 0 warnings
[exit 1]
```

Remove `seq-width`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:slug}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_122

The template or the `const` of the `id` key admits an id that isn't a slug ([TL_122][sokf:tramlines:tramlines-spec#tl_122]).

### Message

```text
<file>:<line>: error[TL_122]: template `<template>` admits a non-slug: `<sample>` is not a slug
<file>:<line>: error[TL_122]: template `<template>`: the state `<state>` of `<key>` is not a slug, so no id can hold it
<file>:<line>: error[TL_122]: the id `<id>` is not a slug
```

### Cause

An id is a slug: lowercase ASCII letters and digits, in words joined by single hyphens, such as `note-001`. The command writes the literal text of the template and the state of each lifecycle hole into the id, so the text and the states are slugs too. A fixed id, the `const` of `id`, is a slug.

### Fix

Change the schema. Write the text of the template or the `const` as a slug, and name each state of a dimension that the template holds as a slug.

### Example

This `id` template starts with a capital letter:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'Note-${:seq}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
id: note-001
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_122]: template `Note-${:seq}` admits a non-slug: `Note-x` is not a slug
1 error, 0 warnings
[exit 1]
```

Write the template in lowercase:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  id:
    template: 'note-${:seq}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_59

The `id` of a document contains a lifecycle state other than the state of the document ([TL_59][sokf:tramlines:tramlines-spec#tl_59]).

### Message

```text
<file>:<line>: error[TL_59]: id `<id>` holds `<state>` for `<key>`, but `<key>` is `<other state>` (schema <id>, key `id`)
```

### Cause

The `template` of `id` contains a hole named for a lifecycle key, such as `${status}`. The `id` of each document therefore repeats the state of the document for that key. The state of the document changed, and its `id` still contains the old state.

### Fix

Change the document. Write the state of the document into its `id`, or set the state back to the one in the `id`.

### Example

This schema puts the `status` of a note into its `id`:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: status, states: [open, closed] }
frontmatter:
  id:
    template: 'note-${status}-${:seq}'
sections:
  - heading: Note
    level: 1
    blocks: []
```

The `status` of this note is `closed`, but its `id` contains `open`:

**`note.md`**

```markdown
---
id: note-open-001
status: closed
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:2: error[TL_59]: id `note-open-001` holds `open` for `status`, but `status` is `closed` (schema note, key `id`)
1 error, 0 warnings
[exit 1]
```

Write the state into the `id`:

**`note.md`**

```markdown
---
id: note-closed-001
status: closed
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_95

The lifecycle declares one dimension twice ([TL_95][sokf:tramlines:tramlines-spec#tl_95]).

### Message

```text
<file>:<line>: error[TL_95]: dimension `<key>` is declared twice
```

### Cause

Each dimension under `lifecycle` declares one frontmatter key, and its `states` list the values that the key can take. Two dimensions with the same key would give that key two lists of states.

### Fix

Change the schema. Merge the two dimensions into one, or give one of them another key.

### Example

This lifecycle declares `status` twice:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: status, states: [draft, done] }
  - { key: status, states: [open, closed] }
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_95]: dimension `status` is declared twice
1 error, 0 warnings
[exit 1]
```

Keep one dimension:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: status, states: [draft, done] }
sections:
  - heading: Note
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_112

The `frontmatter` of the schema declares the key of a lifecycle dimension ([TL_112][sokf:tramlines:tramlines-spec#tl_112]).

### Message

```text
<file>:<line>: error[TL_112]: `<key>` is a lifecycle dimension; it is not repeated under `frontmatter`
```

### Cause

A dimension under `lifecycle` declares its frontmatter key, and its `states` list the values that the key can take. Because the lifecycle declares the key, `frontmatter` can't declare it again.

### Fix

Change the schema. Remove the key of the dimension from `frontmatter`.

### Example

This schema declares `status` both as a lifecycle dimension and under `frontmatter`:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
frontmatter:
  status:
    enum: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TL_112]: `status` is a lifecycle dimension; it is not repeated under `frontmatter`
1 error, 0 warnings
[exit 1]
```

Remove `status` from `frontmatter`:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
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

[sokf:errors-schema#tl_103]: /docs/reference/errors-schema.md#tl_103
[sokf:tramlines:tramlines-spec#tl_111]: /knowledge/spec/tramlines-spec.md#tl_111
[sokf:tramlines:tramlines-spec#tl_112]: /knowledge/spec/tramlines-spec.md#tl_112
[sokf:tramlines:tramlines-spec#tl_114]: /knowledge/spec/tramlines-spec.md#tl_114
[sokf:tramlines:tramlines-spec#tl_115]: /knowledge/spec/tramlines-spec.md#tl_115
[sokf:tramlines:tramlines-spec#tl_116]: /knowledge/spec/tramlines-spec.md#tl_116
[sokf:tramlines:tramlines-spec#tl_122]: /knowledge/spec/tramlines-spec.md#tl_122
[sokf:tramlines:tramlines-spec#tl_20]: /knowledge/spec/tramlines-spec.md#tl_20
[sokf:tramlines:tramlines-spec#tl_58]: /knowledge/spec/tramlines-spec.md#tl_58
[sokf:tramlines:tramlines-spec#tl_59]: /knowledge/spec/tramlines-spec.md#tl_59
[sokf:tramlines:tramlines-spec#tl_95]: /knowledge/spec/tramlines-spec.md#tl_95
