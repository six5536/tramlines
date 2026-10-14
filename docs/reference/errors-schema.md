---
type: ErrorReference
id: errors-schema
title: 'Errors: the schema'
description: The errors that tramlines reports for the YAML, the keys, the values and the regular expressions of a schema file, and how to fix each one.
order: 10
---

# Errors: the schema

These errors come from the schema file itself: its YAML, its keys and their values, and its regular expressions. Section 3 of the core specification defines these errors. [A separate page][sokf:errors-schema-shapes] lists the errors for the shapes and the templates of its rules.

## TL_12

The schema contains a key that the grammar doesn't define for the map that holds it ([TL_12][sokf:tramlines:tramlines-spec#tl_12]).

### Message

```text
<file>:<line>: error[TL_12]: unknown key `<key>` in `<rule map>`
<file>:<line>: error[TL_12]: unknown key `heading-from` in `section` (`heading-from: <key>` became `heading: { from: <key> }`)
```

### Cause

The command checks the schema against the grammar before it uses the schema. Each map of a schema accepts only the keys that the grammar defines for it. A misspelled key is unknown, and so is a key that belongs to another kind of map. In the message, `<rule map>` names the kind of map that holds the key, such as `schema`, `section`, `block`, or `format`.

### Fix

Change the schema. Correct the spelling of the key, move it to the map where it belongs, or remove it. The schema keys reference lists each key and the maps that accept it.

### Example

This schema misspells `optional`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Links
    level: 2
    optinal: true
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_12]: unknown key `optinal` in `section`
1 error, 0 warnings
[exit 1]
```

Spell the key correctly:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Links
    level: 2
    optional: true
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_101

A value of the schema doesn't have the type that the grammar defines for its key ([TL_101][sokf:tramlines:tramlines-spec#tl_101]).

### Message

```text
<file>:<line>: error[TL_101]: `<rule map>` must be a map, not a <type>
<file>:<line>: error[TL_101]: `<key>` must be an integer
<file>:<line>: error[TL_101]: `<key>` must be true or false
<file>:<line>: error[TL_101]: `<key>` must be a list
<file>:<line>: error[TL_101]: `<key>` must be a map
<file>:<line>: error[TL_101]: `<key>` must be a string, not a <type>
<file>:<line>: error[TL_101]: `<key>` must be a string or a list
<file>:<line>: error[TL_101]: `<key>` must be a string or a map
<file>:<line>: error[TL_101]: `<key>` must be a string or `{ from: <frontmatter key> }`
<file>:<line>: error[TL_101]: `<key>` must be a string, `{ from: <frontmatter key> }` or a `<rule map>` map
<file>:<line>: error[TL_101]: `<key>` items must be strings
<file>:<line>: error[TL_101]: `<key>` items must be integers
<file>:<line>: error[TL_101]: `<key>` items must be booleans
<file>:<line>: error[TL_101]: `<key>` values must be strings or lists of strings
<file>:<line>: error[TL_101]: `types` must be a map of types
<file>:<line>: error[TL_101]: type `<name>` must be a regex or a list of choices
<file>:<line>: error[TL_101]: type `<name>`: a list of choices holds one or more non-empty strings
<file>:<line>: error[TL_101]: the format registry must be a map of formats
```

### Cause

The grammar gives each key a type: a number, `true` or `false`, a string, a list, or a map. The value in the schema is of another type, such as a word where the key takes a number, or a single string where the key takes a list.

### Fix

Change the schema. Write the value in the type that the message states. The schema keys reference lists the type of each key.

### Example

This schema sets `level` to a word instead of a number:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: one
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:4: error[TL_101]: `level` must be an integer
1 error, 0 warnings
[exit 1]
```

Write the level as a number:

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

## TL_102

A value of the schema has the right type, but it isn't one of the values that its key allows ([TL_102][sokf:tramlines:tramlines-spec#tl_102]).

### Message

```text
<file>:<line>: error[TL_102]: `<key>` must be one of <values>; `<value>` is not
<file>:<line>: error[TL_102]: `<key>` must be a slug: `<value>` is not
<file>:<line>: error[TL_102]: type `<name>` must be named by a slug
<file>:<line>: error[TL_102]: template `<template>`: `seq-width` must be positive
```

### Cause

Some keys take only a value from a list, such as `kind`, which names a kind of block. Others, such as `rule-id` and the name of a type, take only a slug: a string of lowercase letters, digits, and hyphens. A `seq-width` must be 1 or more.

### Fix

Change the schema. Write one of the values that the message lists, write the value or the name of the type as a slug, or set `seq-width` to 1 or more.

### Example

This schema names a kind of block that doesn't exist:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: paragraph
        example: Buy milk.
```

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TL_102]: `kind` must be one of `prose`, `bullet-list`, `numbered-list`, `table`, `code`, `blockquote`, `image`, `html`, `thematic-break`, `references`, `group`; `paragraph` is not
1 error, 0 warnings
[exit 1]
```

Use the kind `prose`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_103

A map of the schema lacks a key that the grammar requires ([TL_103][sokf:tramlines:tramlines-spec#tl_103]).

### Message

```text
<file>:<line>: error[TL_103]: `<rule map>` requires `<key>`
<file>:<line>: error[TL_103]: `<key>` requires `<other key>` beside it
<file>:<line>: error[TL_103]: `<rule map>` needs at least one of <keys>
<file>:<line>: error[TL_103]: `<rule map>` needs exactly one of <keys>
<file>:<line>: error[TL_103]: `lines-open` without `lines`
```

### Cause

The grammar requires some keys in every map of a kind, such as `line-limit` in the schema and `level` in a section. Some keys need another key beside them, such as `number-from-parent`, which needs `parent`. Some maps need one key of a group, such as a `key-derives-from`, which takes `rule` or `from`. The map in the schema lacks the key, or has no key of the group.

### Fix

Change the schema. Add the key that the message names, or one key of the group that it lists. If the schema doesn't need a key that requires another, remove that key.

### Example

This schema has no `line-limit`:

**`note.yaml`**

```yaml
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
note.yaml:1: error[TL_103]: `schema` requires `line-limit`
1 error, 0 warnings
[exit 1]
```

Add the limit. A limit of `0` sets none:

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

## TL_104

A map of the schema contains more than one key of a group that allows only one ([TL_104][sokf:tramlines:tramlines-spec#tl_104]).

### Message

```text
<file>:<line>: error[TL_104]: `<rule map>` needs exactly one of <keys>
```

### Cause

Some maps take exactly one key of a group. A `key-derives-from`, for example, takes its prefixes either from a rule, with `rule`, or from a frontmatter key, with `from`. The map in the schema contains two keys of that group. Two shapes on one rule cause a separate error, [TL_22][sokf:errors-schema-shapes#tl_22].

### Fix

Change the schema. Keep one key of the group that the message lists, and remove the others.

### Example

This `key-derives-from` names both a rule and a frontmatter key:

**`note.yaml`**

```yaml
line-limit: 0
types:
  code: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    example: NOTE
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:code} ${}'
          key: id
          key-derives-from: { rule: other, from: code, separator: '-' }
          example: NOTE-1 Buy milk
```

**`note.md`**

```markdown
---
code: NOTE
---

# Note

- <a id="note-1"></a>NOTE-1 Buy milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:15: error[TL_104]: `key-derivation` needs exactly one of `rule`, `from`
1 error, 0 warnings
[exit 1]
```

Keep `from`:

**`note.yaml`**

```yaml
line-limit: 0
types:
  code: '[A-Z]+-[0-9]+'
frontmatter:
  code:
    example: NOTE
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:code} ${}'
          key: id
          key-derives-from: { from: code, separator: '-' }
          example: NOTE-1 Buy milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_13

The schema file isn't valid YAML, so the command can't read it and stops ([TL_13][sokf:tramlines:tramlines-spec#tl_13]).

### Message

```text
error[TL_13]: <file>: YAML does not parse at line <line>: <reason>
error[TL_13]: <file>: YAML mapping key at line <line> is not a string
```

### Cause

The command reads the schema as YAML 1.2 with the core schema. A syntax error, such as a quote that isn't closed or a key that appears twice in one map, stops the command with exit code 2 before it checks anything. A map key that isn't a string stops it the same way. An anchor, an alias, or a tag causes a separate error, [TL_14](#tl_14).

### Fix

Fix the YAML of the schema at the line that the message names.

### Example

This schema opens a quote and doesn't close it:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: "Note
    level: 1
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
error[TL_13]: note.yaml: YAML does not parse at line 3: invalid indentation in quoted scalar
[exit 2]
```

Close the quote:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: "Note"
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_14

The schema or the formats file uses a YAML anchor, alias, or tag, which neither file can contain, so the command stops ([TL_14][sokf:tramlines:tramlines-spec#tl_14]).

### Message

```text
error[TL_14]: <file>: YAML anchor at line <line>: anchors are not read
error[TL_14]: <file>: YAML alias at line <line>: aliases are not read
error[TL_14]: <file>: YAML tag at line <line>: tags are not read
```

### Cause

An anchor (`&name`) names a value, and an alias (`*name`) repeats it. A tag (`!name`) gives a value a type. A schema, and the formats file that `--formats` names, can't use any of them: each value must appear in full where it's written. The command stops with exit code 2 before it checks anything.

### Fix

Change the file that the message names. Write out in full each value that an alias repeats, and remove the anchor and any tag.

### Example

This schema anchors the level of one section and repeats it in another with an alias:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: &top 1
    blocks: []
  - heading: Summary
    level: *top
    blocks: []
```

**`note.md`**

```markdown
# Note

# Summary
```

```console
$ tramlines validate --schema note.yaml note.md
error[TL_14]: note.yaml: YAML anchor at line 4: anchors are not read
[exit 2]
```

Write the level out each time:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Summary
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_15

Two rules of the schema declare the same `rule-id` ([TL_15][sokf:tramlines:tramlines-spec#tl_15]).

### Message

```text
<file>:<line>: error[TL_15]: `rule-id` `<id>` is declared twice
```

### Cause

Other rules refer to a rule by its `rule-id`, so each `rule-id` must name exactly one rule. The message names the id, on the line of the second rule that declares it.

### Fix

Change the schema. Give one of the two rules another `rule-id`, and update each reference to that rule.

### Example

Both sections of this schema declare `rule-id: note`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    rule-id: note
    blocks: []
  - heading: Notes
    level: 2
    rule-id: note
    optional: true
    blocks: []
```

**`note.md`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:7: error[TL_15]: `rule-id` `note` is declared twice
1 error, 0 warnings
[exit 1]
```

Rename the second one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    rule-id: note
    blocks: []
  - heading: Notes
    level: 2
    rule-id: notes
    optional: true
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_118

A regular expression in the schema, or the expression that a template compiles to, doesn't compile in the regular expression dialect of tramlines ([TL_118][sokf:tramlines:tramlines-spec#tl_118]).

### Message

```text
<file>:<line>: error[TL_118]: `<key>` is not a valid regex: <reason>
<file>:<line>: error[TL_118]: type `<name>`: not a valid regex: <reason>
<file>:<line>: error[TL_118]: template `<template>` does not compile: <reason>
```

### Cause

The command reads each regular expression in the Rust dialect. That dialect has no lookaround, such as `(?=…)` or `(?<!…)`, and no backreference, such as `\1`. A `pattern`, a `select-pattern`, a `source-pattern`, an entry of `prohibited-patterns`, or a type doesn't compile if it uses either of these or isn't a valid regular expression at all. The `<reason>` comes from the regular expression library. For a template or a pattern of a format, the message starts with ``format `<format>`: ``, and for a variant, with ``format `<format>`: variant `<variant>`: ``.

### Fix

Change the schema. Correct the syntax that the `<reason>` names. Rewrite lookaround or a backreference without it: a word boundary `\b` or a character class often does the same work.

### Example

This schema bans `TODO` with a lookaround:

**`note.yaml`**

```yaml
line-limit: 0
prohibited-patterns: [{pattern: '(?<![A-Za-z])TODO', message: 'A note contains no TODO.'}]
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
note.yaml:2: error[TL_118]: `pattern` is not a valid regex: look-around, including look-ahead and look-behind, is not supported
1 error, 0 warnings
[exit 1]
```

Use a word boundary instead:

**`note.yaml`**

```yaml
line-limit: 0
prohibited-patterns: [{pattern: '\bTODO', message: 'A note contains no TODO.'}]
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

[sokf:errors-schema-shapes]: /docs/reference/errors-schema-shapes.md
[sokf:errors-schema-shapes#tl_22]: /docs/reference/errors-schema-shapes.md#tl_22
[sokf:tramlines:tramlines-spec#tl_101]: /knowledge/spec/tramlines-spec.md#tl_101
[sokf:tramlines:tramlines-spec#tl_102]: /knowledge/spec/tramlines-spec.md#tl_102
[sokf:tramlines:tramlines-spec#tl_103]: /knowledge/spec/tramlines-spec.md#tl_103
[sokf:tramlines:tramlines-spec#tl_104]: /knowledge/spec/tramlines-spec.md#tl_104
[sokf:tramlines:tramlines-spec#tl_118]: /knowledge/spec/tramlines-spec.md#tl_118
[sokf:tramlines:tramlines-spec#tl_12]: /knowledge/spec/tramlines-spec.md#tl_12
[sokf:tramlines:tramlines-spec#tl_13]: /knowledge/spec/tramlines-spec.md#tl_13
[sokf:tramlines:tramlines-spec#tl_14]: /knowledge/spec/tramlines-spec.md#tl_14
[sokf:tramlines:tramlines-spec#tl_15]: /knowledge/spec/tramlines-spec.md#tl_15
