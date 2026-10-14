---
type: ErrorReference
id: errors-schema-types-keys-lifecycle
title: 'Errors: schema types and keys'
description: The errors that tramlines reports for the types, the keys, the key derivations, and the counts of a schema, and how to fix each one.
order: 12
---

# Errors: schema types and keys

These errors come from the types, the keys, and the counts that a schema declares. Sections 3, 4, and 5 of the core specification define them. One error, [TL_33](#tl_33), reports a problem in a document: a key that breaks one of the schema's key derivations.

## TL_29

A type under `types` reuses the name of a built-in type ([TL_29][sokf:tramlines:tramlines-spec#tl_29]).

### Message

```text
<file>:<line>: error[TL_29]: type `<name>` is built in and reserved; name the type otherwise
```

### Cause

tramlines reserves the names of the built-in types `text`, `word`, `int`, `slug`, and `seq`, so a schema can't define them again.

### Fix

Change the schema. Remove the type and use the built-in one, or give the type another name and use that name in each template.

### Example

This schema defines a type `int`, which is built in:

**`note.yaml`**

```yaml
line-limit: 0
types:
  int: '[0-9]+'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:int} ${}'
          example: 2 apples
```

**`note.md`**

```markdown
# Note

- 2 apples
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:3: error[TL_29]: type `int` is built in and reserved; name the type otherwise
1 error, 0 warnings
[exit 1]
```

Remove the type, and use the built-in `int`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:int} ${}'
          example: 2 apples
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_109

The regular expression of a type under `types` contains an anchor or a capture group ([TL_109][sokf:tramlines:tramlines-spec#tl_109]).

### Message

```text
<file>:<line>: error[TL_109]: type `<name>`: `<regex>` holds an anchor; a template matches the whole line itself
<file>:<line>: error[TL_109]: type `<name>`: `<regex>` holds a capture group; write `(?:…)`, since a template names its own groups
```

### Cause

The regular expression of a type matches one hole of a template. The template anchors the whole line and names its own groups, so the type can't contain a `^` or `$` anchor or a capture group. The command reports each refused type once. No template that names a refused type reports [TL_120][sokf:errors-schema-shapes#tl_120].

### Fix

Change the schema. Remove the anchors from the regular expression of the type, and write each group as `(?:…)`.

### Example

The type `code` of this schema starts with `^` and ends with `$`:

**`note.yaml`**

```yaml
line-limit: 0
types:
  code: '^[A-Z]+$'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:code} ${}'
          example: SHOP milk
```

**`note.md`**

```markdown
# Note

- SHOP milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:3: error[TL_109]: type `code`: `^[A-Z]+$` holds an anchor; a template matches the whole line itself
1 error, 0 warnings
[exit 1]
```

Remove the anchors:

**`note.yaml`**

```yaml
line-limit: 0
types:
  code: '[A-Z]+'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:code} ${}'
          example: SHOP milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_31

The `key` of a rule or a format names no hole or named group of its shape ([TL_31][sokf:tramlines:tramlines-spec#tl_31]).

### Message

```text
<file>:<line>: error[TL_31]: `key` `<key>` names no hole or named group of the rule's shape
<file>:<line>: error[TL_31]: `key` `<key>` names a hole of a shape, and the rule declares none
<file>:<line>: error[TL_31]: format `<format>`: `key` `<key>` names no hole or named group of the shape
<file>:<line>: error[TL_31]: format `<format>`: `key` `<key>` names no hole or named group of variant `<variant>`
```

### Cause

The key `key` holds the name of a hole of the template, such as `item` in `${item:slug}`, or of a named group of the pattern. The rule captures the key from that hole. When `key` names something else, or when the rule declares no shape, the rule finds no hole to read the key from. The `key` of a format names a hole or a named group of the shape of the format, and of each of its variants.

### Fix

Change the schema. Set `key` to the name of a hole or a named group of the shape, or give the rule the shape that holds the key.

### Example

This rule names its key `name`, but its template calls the hole `item`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${item:slug}: ${:int}'
          key: name
          example: 'milk: 2'
```

**`note.md`**

```markdown
# Note

- <a id="milk"></a>milk: 2
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_31]: `key` `name` names no hole or named group of the rule's shape
1 error, 0 warnings
[exit 1]
```

Name the hole:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${item:slug}: ${:int}'
          key: item
          example: 'milk: 2'
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_113

A rule declares `key` and binds a format that captures a key itself ([TL_113][sokf:tramlines:tramlines-spec#tl_113]).

### Message

```text
<file>:<line>: error[TL_113]: `key` beside format `<format>`, which captures a key itself
```

### Cause

A format that declares `key` captures that key from every line that it matches. A rule that binds the format takes the key of the format, so a `key` of its own would name a second key for the same line.

### Fix

Change the schema. Remove `key` from the rule, or bind a format that declares no key.

### Example

The format `item` captures the key `id`, and the rule that binds it declares `key` too:

**`note.yaml`**

```yaml
line-limit: 0
formats:
  item:
    template: '${id:slug}: ${:int}'
    key: id
    example: 'milk: 2'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: item
          key: id
```

**`note.md`**

```markdown
# Note

- <a id="milk"></a>milk: 2
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:13: error[TL_113]: `key` beside format `item`, which captures a key itself
1 error, 0 warnings
[exit 1]
```

Remove `key` from the rule:

**`note.yaml`**

```yaml
line-limit: 0
formats:
  item:
    template: '${id:slug}: ${:int}'
    key: id
    example: 'milk: 2'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: item
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_33

A key in the document starts with a prefix that the `key-derives-from` of its rule doesn't allow, or the key lacks the separator ([TL_33][sokf:tramlines:tramlines-spec#tl_33]).

### Message

```text
<file>:<line>: error[TL_33]: key `<key>` derives from `<prefix>`, which rule `<rule-id>` did not capture (schema <id>, <place of the rule>)
<file>:<line>: error[TL_33]: key `<key>` derives from `<prefix>`, which is not `<value>`, the value of the frontmatter `<frontmatter key>` (schema <id>, <place of the rule>)
<file>:<line>: error[TL_33]: key `<key>` derives from `<prefix>`, and the frontmatter `<frontmatter key>` that it derives from is absent (schema <id>, <place of the rule>)
<file>:<line>: error[TL_33]: key `<key>` has no `<separator>` separator to derive from (schema <id>, <place of the rule>)
```

### Cause

`key-derives-from: { rule: <rule-id>, separator: <separator> }` requires the part of each key before the separator to be a key that the named rule captured. `key-derives-from: { from: <frontmatter key>, separator: <separator> }` requires it to be the value of that frontmatter key. The message shows that value after the frontmatter key's name. The error means that a key in the document starts with another prefix, or contains no separator at all.

### Fix

Change the key in the document so that its prefix is a key of the named rule or the frontmatter value. If the schema names the wrong rule, frontmatter key, or separator, fix `key-derives-from` in the schema instead.

### Example

This schema requires the key of each item to start with the value of the frontmatter key `code`:

**`note.yaml`**

```yaml
line-limit: 0
types:
  item-id: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    pattern: '[A-Z]+'
    example: SHOP
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:item-id}: ${}'
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: 'SHOP-1: milk'
```

The second item of this document starts with `HOME`:

**`note.md`**

```markdown
---
code: SHOP
---

# Note

- <a id="shop-1"></a>SHOP-1: milk
- <a id="home-2"></a>HOME-2: soap
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:8: error[TL_33]: key `HOME-2` derives from `HOME`, which is not `SHOP`, the value of the frontmatter `code` (schema note, section "Note")
1 error, 0 warnings
[exit 1]
```

Start the key with the code:

**`note.md`**

```markdown
---
code: SHOP
---

# Note

- <a id="shop-1"></a>SHOP-1: milk
- <a id="shop-2"></a>SHOP-2: soap
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_96

A unit rule that captures no key declares `key-derives-from` ([TL_96][sokf:tramlines:tramlines-spec#tl_96]).

### Message

```text
<file>:<line>: error[TL_96]: `key-derives-from` on a rule that captures no key
```

### Cause

`key-derives-from` checks the key that its rule captures against a frontmatter value or against the keys of another rule. The rule declares no `key`, and it binds either no format or a format that declares none. It therefore captures no key for `key-derives-from` to check. On a heading, a `key-derives-from` with no `key` reports [TL_103][sokf:errors-schema#tl_103] instead, because the grammar requires `key` beside it.

### Fix

Change the schema. Give the rule a `key` that names a hole of its shape, or remove `key-derives-from`.

### Example

This rule derives its keys from the frontmatter key `code`, but declares no `key`:

**`note.yaml`**

```yaml
line-limit: 0
types:
  item-id: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    pattern: '[A-Z]+'
    example: SHOP
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:item-id}: ${}'
          key-derives-from: { from: code, separator: "-" }
          example: 'SHOP-1: milk'
```

**`note.md`**

```markdown
---
code: SHOP
---

# Note

- <a id="shop-1"></a>SHOP-1: milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:14: error[TL_96]: `key-derives-from` on a rule that captures no key
1 error, 0 warnings
[exit 1]
```

Declare the key:

**`note.yaml`**

```yaml
line-limit: 0
types:
  item-id: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    pattern: '[A-Z]+'
    example: SHOP
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:item-id}: ${}'
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: 'SHOP-1: milk'
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_117

A `key-derives-from` names a rule or a frontmatter key that the schema doesn't declare ([TL_117][sokf:tramlines:tramlines-spec#tl_117]).

### Message

```text
<file>:<line>: error[TL_117]: `key-derives-from` names no rule `<rule-id>`
<file>:<line>: error[TL_117]: `key-derives-from` names no frontmatter key `<key>`
```

### Cause

`key-derives-from: { rule: <rule-id>, separator: <separator> }` checks each key against the keys of the rule with that `rule-id`. `key-derives-from: { from: <frontmatter key>, separator: <separator> }` checks it against the value of that frontmatter key. The schema declares no rule or frontmatter key of that name.

### Fix

Change the schema. Correct the name in `key-derives-from`, or declare the rule or the frontmatter key that it names.

### Example

This rule derives its keys from a frontmatter key `code`, which the schema doesn't declare:

**`note.yaml`**

```yaml
line-limit: 0
types:
  item-id: '[A-Z]+-[0-9]+'
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:item-id}: ${}'
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: 'SHOP-1: milk'
```

**`note.md`**

```markdown
---
code: SHOP
---

# Note

- <a id="shop-1"></a>SHOP-1: milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:10: error[TL_117]: `key-derives-from` names no frontmatter key `code`
1 error, 0 warnings
[exit 1]
```

Declare `code`:

**`note.yaml`**

```yaml
line-limit: 0
types:
  item-id: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    pattern: '[A-Z]+'
    example: SHOP
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:item-id}: ${}'
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: 'SHOP-1: milk'
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_47

A rule sets `key-optional` but captures no key ([TL_47][sokf:tramlines:tramlines-spec#tl_47]).

### Message

```text
<file>:<line>: error[TL_47]: `key-optional` without a `key` or a `format` that declares one
```

### Cause

`key-optional: true` lets a unit leave out the key that its rule captures. The rule declares no `key` and binds no format that declares one. The rule therefore captures no key that a unit could leave out.

### Fix

Change the schema. Remove `key-optional`, or declare the `key` that it makes optional.

### Example

This rule sets `key-optional` and captures no key:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:slug}'
          key-optional: true
          example: milk
```

**`note.md`**

```markdown
# Note

- milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_47]: `key-optional` without a `key` or a `format` that declares one
1 error, 0 warnings
[exit 1]
```

Remove `key-optional`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:slug}'
          example: milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_98

A rule or a frontmatter key declares a `min` greater than its `max` ([TL_98][sokf:tramlines:tramlines-spec#tl_98]).

### Message

```text
<file>:<line>: error[TL_98]: `min` <min> is above `max` <max>
```

### Cause

`min` and `max` bound how many units a block or a rule takes or counts, or how many elements a frontmatter list contains. When `min` is larger than `max`, no count meets both bounds. No document can then satisfy the rule.

### Fix

Change the schema. Swap the two numbers, or correct the one that's wrong.

### Example

This rule requires at least two items and allows at most one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - template: '${:slug}'
            min: 2
            max: 1
            example: milk
```

**`note.md`**

```markdown
# Note

- milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_98]: `min` 2 is above `max` 1
1 error, 0 warnings
[exit 1]
```

Set `min` to a value no higher than `max`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - template: '${:slug}'
            min: 1
            max: 2
            example: milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-schema#tl_103]: /docs/reference/errors-schema.md#tl_103
[sokf:errors-schema-shapes#tl_120]: /docs/reference/errors-schema-shapes.md#tl_120
[sokf:tramlines:tramlines-spec#tl_109]: /knowledge/spec/tramlines-spec.md#tl_109
[sokf:tramlines:tramlines-spec#tl_113]: /knowledge/spec/tramlines-spec.md#tl_113
[sokf:tramlines:tramlines-spec#tl_117]: /knowledge/spec/tramlines-spec.md#tl_117
[sokf:tramlines:tramlines-spec#tl_29]: /knowledge/spec/tramlines-spec.md#tl_29
[sokf:tramlines:tramlines-spec#tl_31]: /knowledge/spec/tramlines-spec.md#tl_31
[sokf:tramlines:tramlines-spec#tl_33]: /knowledge/spec/tramlines-spec.md#tl_33
[sokf:tramlines:tramlines-spec#tl_47]: /knowledge/spec/tramlines-spec.md#tl_47
[sokf:tramlines:tramlines-spec#tl_96]: /knowledge/spec/tramlines-spec.md#tl_96
[sokf:tramlines:tramlines-spec#tl_98]: /knowledge/spec/tramlines-spec.md#tl_98
