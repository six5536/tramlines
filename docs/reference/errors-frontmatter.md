---
type: ErrorReference
id: errors-frontmatter
title: 'Errors: frontmatter'
description: The errors that tramlines reports for the frontmatter of a document, and how to fix each one.
order: 30
---

# Errors: frontmatter

These errors come from the frontmatter of a document. The frontmatter can fail to parse, or it can hold keys and values that break their constraints. Section 6 of the core specification defines these errors. [A separate page][sokf:errors-lines] lists the errors for the lines of a document and its length.

## TL_53

The frontmatter of the document isn't valid YAML ([TL_53][sokf:tramlines:tramlines-spec#tl_53]).

### Message

```text
<file>:<line>: error[TL_53]: the frontmatter does not read: <reason>
```

### Cause

The frontmatter doesn't parse as YAML 1.2. A common cause is a plain value that contains `: `, which YAML reads as the start of a map. A map key that isn't a string, such as `[a]: b`, is also an error. The error reports the first line of the frontmatter, and the command skips every frontmatter rule.

### Fix

Correct the YAML of the frontmatter in the document. Quote a value that contains `: `, and write each key as plain text.

### Example

This schema requires a `title`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

The `title` of this document contains `: `:

**`note.md`**

```markdown
---
title: Groceries: weekly
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:1: error[TL_53]: the frontmatter does not read: YAML does not parse at line 2: mapping values are not allowed in this context
1 error, 0 warnings
[exit 1]
```

Quote the value:

**`note.md`**

```markdown
---
title: 'Groceries: weekly'
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_123

The frontmatter of the document contains a YAML anchor, an alias, or a tag ([TL_123][sokf:tramlines:tramlines-spec#tl_123]).

### Message

```text
<file>:<line>: error[TL_123]: the frontmatter does not read: <reason>
```

### Cause

The frontmatter uses an anchor (`&name`), an alias (`*name`), or a tag (`!tag`). Frontmatter is plain YAML: each value is written where it applies. The error reports the first line of the frontmatter, and the command skips every frontmatter rule.

### Fix

In the document, replace each alias with the value that its anchor names, written out in full, then remove each anchor and each tag.

### Example

This schema requires a `title` and a `subtitle`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
  subtitle:
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

The `subtitle` of this document repeats the `title` through an alias:

**`note.md`**

```markdown
---
title: &name Groceries
subtitle: *name
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:1: error[TL_123]: the frontmatter does not read: YAML anchor at line 2: anchors are not read
1 error, 0 warnings
[exit 1]
```

Write the value out in full:

**`note.md`**

```markdown
---
title: Groceries
subtitle: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_54

A key that the schema requires is missing from the frontmatter of the document ([TL_54][sokf:tramlines:tramlines-spec#tl_54]).

### Message

```text
<file>:<line>: error[TL_54]: frontmatter key `<key>` is missing (schema <id>, key `<key>`)
```

### Cause

The schema declares the key under `frontmatter` without `optional: true`, and the document leaves it out. A document with no frontmatter at all is missing every key that the schema requires.

### Fix

Add the key to the frontmatter of the document. If documents of this kind can leave the key out, set `optional: true` on the key in the schema instead.

### Example

This schema requires a `title`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document leaves out `title`:

**`note.md`**

```markdown
---
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:1: error[TL_54]: frontmatter key `title` is missing (schema note, key `title`)
1 error, 0 warnings
[exit 1]
```

Add the key:

**`note.md`**

```markdown
---
title: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_55

The frontmatter of the document contains a key that the schema doesn't declare ([TL_55][sokf:tramlines:tramlines-spec#tl_55]).

### Message

```text
<file>:<line>: error[TL_55]: frontmatter key `<key>` is not declared (schema <id>)
```

### Cause

A schema accepts only the frontmatter keys that it declares under `frontmatter`, the key of each dimension of its `lifecycle`, and the keys that a host reserves. The document contains another key, and the schema doesn't set `frontmatter-open`.

### Fix

Remove the key from the frontmatter of the document, or correct its spelling. If documents of this kind need the key, declare it under `frontmatter` in the schema. To accept any key, set `frontmatter-open: true` in the schema.

### Example

This schema declares only `title`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document also contains an `author`:

**`note.md`**

```markdown
---
title: Groceries
author: Ada
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TL_55]: frontmatter key `author` is not declared (schema note)
1 error, 0 warnings
[exit 1]
```

Remove the key:

**`note.md`**

```markdown
---
title: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_56

A frontmatter value, or an element of a list, doesn't meet the `const`, the shape, the `enum`, or the `format` of its key ([TL_56][sokf:tramlines:tramlines-spec#tl_56]).

### Message

```text
<file>:<line>: error[TL_56]: `<key>` must be `<const>`; it is `<value>` (schema <id>, key `<key>`)
<file>:<line>: error[TL_56]: `<key>` `<value>` <failure>; a value of this kind reads <example> (schema <id>, key `<key>`)
<file>:<line>: error[TL_56]: `<key>` must be one of <values>; `<value>` is not (schema <id>, key `<key>`)
<file>:<line>: error[TL_56]: `<key>` must be an absolute URL; `<value>` is not (schema <id>, key `<key>`)
<file>:<line>: error[TL_56]: `<key>` must be a slug; `<value>` is not (schema <id>, key `<key>`)
<file>:<line>: error[TL_56]: id `<value>` does not match the template `<template>` (schema <id>, key `id`)
```

### Cause

The key declares a `const`, a `template`, a `pattern`, an `enum`, or a `format` (`url` or `slug`), and the value doesn't meet it. For a key that declares `list`, the command checks each element. For an `id` key with a `template`, the command reports a message of its own. A program that uses tramlines as a library can add formats of its own. Such a program reports a value that fails one of them under this key too, in its own words.

### Fix

Change the value in the frontmatter of the document so that it meets the constraint. If the constraint is wrong for documents of this kind, correct it in the schema.

### Example

This schema allows two values of `status`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  status:
    enum: [draft, done]
    example: draft
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document uses another:

**`note.md`**

```markdown
---
status: finished
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:2: error[TL_56]: `status` must be one of `draft`, `done`; `finished` is not (schema note, key `status`)
1 error, 0 warnings
[exit 1]
```

Use one of the values:

**`note.md`**

```markdown
---
status: done
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_106

A list in the frontmatter has fewer elements than the `min` of its key ([TL_106][sokf:tramlines:tramlines-spec#tl_106]).

### Message

```text
<file>:<line>: error[TL_106]: `<key>` needs at least <min> entries; it has <n> (schema <id>, key `<key>`)
```

### Cause

The key declares `list` and a `min`, and the list in the document is shorter. An empty list, `[]`, has no elements.

### Fix

Add elements to the list in the document. If documents of this kind can hold fewer, lower `min` on the key in the schema.

### Example

This schema requires at least one tag:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  tags:
    list: true
    min: 1
    example: shopping
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document has none:

**`note.md`**

```markdown
---
tags: []
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:2: error[TL_106]: `tags` needs at least 1 entry; it has 0 (schema note, key `tags`)
1 error, 0 warnings
[exit 1]
```

Add a tag:

**`note.md`**

```markdown
---
tags: [shopping]
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_107

A list in the frontmatter has more elements than the `max` of its key ([TL_107][sokf:tramlines:tramlines-spec#tl_107]).

### Message

```text
<file>:<line>: error[TL_107]: `<key>` allows at most <max> entries; it has <n> (schema <id>, key `<key>`)
```

### Cause

The key declares `list` and a `max`, and the list in the document is longer.

### Fix

Remove elements from the list in the document. If documents of this kind need more, raise `max` on the key in the schema.

### Example

This schema allows two tags:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  tags:
    list: true
    max: 2
    example: shopping
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document has three:

**`note.md`**

```markdown
---
tags: [shopping, weekly, food]
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:2: error[TL_107]: `tags` allows at most 2 entries; it has 3 (schema note, key `tags`)
1 error, 0 warnings
[exit 1]
```

Keep two of them:

**`note.md`**

```markdown
---
tags: [shopping, weekly]
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_57

A frontmatter value, or an element of a list, is a list or a map where the schema expects one value, or one value where the schema expects a list ([TL_57][sokf:tramlines:tramlines-spec#tl_57]).

### Message

```text
<file>:<line>: error[TL_57]: `<key>` must be a list (schema <id>, key `<key>`)
<file>:<line>: error[TL_57]: `<key>` must be a scalar, not a list (schema <id>, key `<key>`)
<file>:<line>: error[TL_57]: `<key>` must be a scalar (schema <id>, key `<key>`)
```

### Cause

The key declares `list: true`, and its value isn't a list. Or the key doesn't declare `list`, and its value is a list or a map. The last message also reports an element of a list that is itself a list or a map.

### Fix

In the frontmatter of the document, write the value in the shape that the schema expects. If the key must hold a list, set `list: true` on it in the schema.

### Example

This schema expects one `title`:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
sections:
  - heading: Note
    level: 1
    blocks: []
```

The `title` of this document is a list:

**`note.md`**

```markdown
---
title:
  - Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TL_57]: `title` must be a scalar, not a list (schema note, key `title`)
1 error, 0 warnings
[exit 1]
```

Write one value:

**`note.md`**

```markdown
---
title: Groceries
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_62

The key of a lifecycle dimension is missing from the frontmatter ([TL_62][sokf:tramlines:tramlines-spec#tl_62]).

### Message

```text
<file>:<line>: error[TL_62]: lifecycle key `<key>` is missing (schema <id>)
```

### Cause

The schema declares a `lifecycle`, and each of its dimensions declares a key that the frontmatter of every document must contain. The document leaves the key out.

### Fix

Add the key to the frontmatter of the document, set to one of the states of the dimension.

### Example

This schema declares a lifecycle with the key `status`:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: status, states: [open, closed] }
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document leaves out `status`:

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:1: error[TL_62]: lifecycle key `status` is missing (schema note)
1 error, 0 warnings
[exit 1]
```

Add the key with a state:

**`note.md`**

```markdown
---
status: open
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_108

The value of the key of a lifecycle dimension isn't one of the states of the dimension ([TL_108][sokf:tramlines:tramlines-spec#tl_108]).

### Message

```text
<file>:<line>: error[TL_108]: `<key>` must be one of <states>; `<value>` is not (schema <id>)
```

### Cause

The schema declares a `lifecycle`, and the dimension lists its `states`. The document sets the key of the dimension to a value that isn't in that list. The command compares the text exactly, so `Open` isn't `open`.

### Fix

In the frontmatter of the document, set the key to one of the states that the message lists. If documents need another state, add it to `states` in the schema.

### Example

This schema declares a lifecycle with the key `status`:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: status, states: [open, closed] }
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document sets `status` to a value that isn't a state:

**`note.md`**

```markdown
---
status: done
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:2: error[TL_108]: `status` must be one of `open`, `closed`; `done` is not (schema note)
1 error, 0 warnings
[exit 1]
```

Use one of the states:

**`note.md`**

```markdown
---
status: closed
---

# Note
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-lines]: /docs/reference/errors-lines.md
[sokf:tramlines:tramlines-spec#tl_106]: /knowledge/spec/tramlines-spec.md#tl_106
[sokf:tramlines:tramlines-spec#tl_107]: /knowledge/spec/tramlines-spec.md#tl_107
[sokf:tramlines:tramlines-spec#tl_108]: /knowledge/spec/tramlines-spec.md#tl_108
[sokf:tramlines:tramlines-spec#tl_123]: /knowledge/spec/tramlines-spec.md#tl_123
[sokf:tramlines:tramlines-spec#tl_53]: /knowledge/spec/tramlines-spec.md#tl_53
[sokf:tramlines:tramlines-spec#tl_54]: /knowledge/spec/tramlines-spec.md#tl_54
[sokf:tramlines:tramlines-spec#tl_55]: /knowledge/spec/tramlines-spec.md#tl_55
[sokf:tramlines:tramlines-spec#tl_56]: /knowledge/spec/tramlines-spec.md#tl_56
[sokf:tramlines:tramlines-spec#tl_57]: /knowledge/spec/tramlines-spec.md#tl_57
[sokf:tramlines:tramlines-spec#tl_62]: /knowledge/spec/tramlines-spec.md#tl_62
