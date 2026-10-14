---
type: HowTo
id: how-to-share-line-formats
title: Share line formats across schemas
description: Define the shape of a line once in a formats file, so that two schemas check a kind of line in the same way.
order: 20
---

# Share line formats across schemas

When two schemas check the same kind of line, write its shape once as a named format and bind it from each rule with `format`. At the end, one formats file defines the line. Every schema that names the format checks the line in the same way, and a schema that needs a different line can change the format for itself alone.

In this guide, a meeting note and a project plan share the line of an action. An action is either open or done, and a done action includes its date.

## Before you begin

- A schema with a list, as in [Your first schema][sokf:tutorial-first-schema].
- An understanding of the shapes of a line, from [Write the shape of a line][sokf:how-to-write-line-shapes].

## Steps

1. Write the formats file. It contains a `registry` map of formats and an optional `types` map of the types that the formats use. A format defines one shape and an `example`, as a rule does. For a line that comes in more than one form, a format declares `variants` instead: named shapes, each with its own `example`. The command tries the variants on a line in order, and the line conforms when one variant matches it.

   Write the example of each variant so that it matches only its own variant. Put a word that distinguishes the variants in each shape, such as `[open]` and `[done]` in the following file.

**`formats.yaml`**

```yaml
types:
  action-id: 'A-\d+'
  date: '\d{4}-\d{2}-\d{2}'
registry:
  action:
    variants:
      open:
        template: '${:action-id} [open]: ${}'
        example: 'A-1 [open]: book the venue'
      done:
        template: '${:action-id} [done] ${:date}: ${}'
        example: 'A-2 [done] 2026-09-30: send the invitations'
```

2. Bind the format from a rule of each schema with `format: <name>`. The rule takes its shape and example from the format, so it declares no shape of its own.

**`meeting.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Actions
    level: 2
    blocks:
      - kind: bullet-list
        each: { format: action }
```

**`plan.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Plan
    level: 1
    blocks:
      - kind: numbered-list
        each: { format: action }
```

3. Pass the formats file with `--formats` when you check a document.

**`meeting.md`**

```markdown
## Actions

- A-1 [open]: book the venue
- A-2 [done] 2026-09-30: send the invitations
```

**`plan.md`**

```markdown
# Plan

1. A-3 [open]: draft the budget
2. A-4 [open]: ask the speakers
```

```console
$ tramlines validate --schema meeting.yaml --formats formats.yaml meeting.md
0 errors, 0 warnings
$ tramlines validate --schema plan.yaml --formats formats.yaml plan.md
0 errors, 0 warnings
```

Without `--formats`, the schema names a format that the command can't find:

```console
$ tramlines validate --schema meeting.yaml meeting.md
meeting.yaml:7: error[TL_97]: `format` names no format `action`
1 error, 0 warnings
[exit 1]
```

4. Read the error for a line that fails a format. The command reports the line against the variant that matches it furthest, with the column where the match stops and the example of that variant. When the match stops at the same column in every variant, the command reports the line against all of them, with the example of the first:

**`meeting.md`**

```markdown
## Actions

- A-1 [open]: book the venue
- A-2 [done]: send the invitations
- Ask Sam about the budget
```

```console
$ tramlines validate --schema meeting.yaml --formats formats.yaml meeting.md
meeting.md:4: error[TL_74]: `A-2 [done]: send the invitations` does not match the `done` form: it fails at column 11: expected ` `, found `: send the invitations`; an item of this kind reads A-2 [done] 2026-09-30: send the invitations (schema meeting, section "Actions" block 1)
meeting.md:5: error[TL_74]: `Ask Sam about the budget` matches none of the forms `open`, `done`; an item of this kind reads A-1 [open]: book the venue or A-2 [done] 2026-09-30: send the invitations (schema meeting, section "Actions" block 1)
2 errors, 0 warnings
[exit 1]
```

5. To change a format for one schema, define a format of the same name under the `formats` of the schema. The nearest layer wins: the `formats` of the schema first, then the formats file. An entry in the schema replaces the whole entry in the file and keeps nothing from it.

   To state only a difference, set `patch: true` on the entry of the schema. The command merges the patch into the entry of the file with JSON Merge Patch (RFC 7386). A key that the patch sets replaces the key of the file, and a key that the patch leaves out is kept. A key that the patch sets to `null` is dropped. A list replaces the whole list of the file. A patch needs no `example` of its own, but the merged entry needs one for each shape. This patch changes only the `done` variant, so that a done action in a plan needs no date:

**`plan.yaml`**

```yaml
line-limit: 0
formats:
  action:
    patch: true
    variants:
      done:
        template: '${:action-id} [done]: ${}'
        example: 'A-2 [done]: send the invitations'
sections:
  - heading: Plan
    level: 1
    blocks:
      - kind: numbered-list
        each: { format: action }
```

**`plan.md`**

```markdown
# Plan

1. A-3 [open]: draft the budget
2. A-4 [done]: ask the speakers
```

```console
$ tramlines validate --schema plan.yaml --formats formats.yaml plan.md
0 errors, 0 warnings
```

6. To change a type that a format uses, define the type under the `types` of the schema. Types resolve through the same layers as formats. The command compiles each format available to a schema against the types available to that schema. In this schema, an action id reads `ACT-1`. The shared examples no longer match, so a patch gives each variant a new example and keeps its template:

**`renumbered.yaml`**

```yaml
line-limit: 0
types:
  action-id: 'ACT-\d+'
formats:
  action:
    patch: true
    variants:
      open:
        example: 'ACT-1 [open]: book the venue'
      done:
        example: 'ACT-2 [done] 2026-09-30: send the invitations'
sections:
  - heading: Actions
    level: 2
    blocks:
      - kind: bullet-list
        each: { format: action }
```

**`renumbered.md`**

```markdown
## Actions

- ACT-1 [open]: book the venue
- ACT-2 [done] 2026-09-30: send the invitations
```

```console
$ tramlines validate --schema renumbered.yaml --formats formats.yaml renumbered.md
0 errors, 0 warnings
```

7. To capture a key from every line of a format, set `key` on the format to the name of a hole or a named group that every shape of the format contains. A key in a markdown document needs an anchor before it. [Capture keys and anchor them][sokf:how-to-capture-keys] explains anchors. Don't declare `key` on a rule that binds a format with a key.

**`formats.yaml`**

```yaml
types:
  action-id: 'A-\d+'
  date: '\d{4}-\d{2}-\d{2}'
registry:
  action:
    key: id
    variants:
      open:
        template: '${id:action-id} [open]: ${}'
        example: 'A-1 [open]: book the venue'
      done:
        template: '${id:action-id} [done] ${:date}: ${}'
        example: 'A-2 [done] 2026-09-30: send the invitations'
```

**`meeting.md`**

```markdown
## Actions

- <a id="a-1"></a>A-1 [open]: book the venue
- <a id="a-2"></a>A-2 [done] 2026-09-30: send the invitations
```

```console
$ tramlines validate --schema meeting.yaml --formats formats.yaml meeting.md
0 errors, 0 warnings
```

8. When the key is optional in the pattern of the format, set `key-optional: true` on the rule. Without it, a line that matches the shape but contains no key is an error. The same holds when the rule declares the key itself. This format describes a note that can start with an action id:

**`formats.yaml`**

```yaml
registry:
  note:
    key: id
    pattern: '^(?:(?P<id>A-\d+): )?[A-Z].*$'
    example: 'A-1: Book the venue'
```

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 2
    blocks:
      - kind: bullet-list
        each: { format: note }
```

**`notes.md`**

```markdown
## Notes

- <a id="a-1"></a>A-1: Book the venue
- Ask Sam about the budget
```

```console
$ tramlines validate --schema notes.yaml --formats formats.yaml notes.md
notes.md:4: error[TL_44]: `Ask Sam about the budget` carries no key: the format `note` captures `id` (schema notes, section "Notes" block 1)
1 error, 0 warnings
[exit 1]
```

With `key-optional: true`, the second line passes as a note without a key:

**`notes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 2
    blocks:
      - kind: bullet-list
        each: { format: note, key-optional: true }
```

```console
$ tramlines validate --schema notes.yaml --formats formats.yaml notes.md
0 errors, 0 warnings
```

A format can also ban text in every line bound to it, with `prohibited-patterns`, as [Ban text in a document][sokf:how-to-ban-text] shows.

## Troubleshooting

- `TL_97`: a rule binds a format that no layer defines. Pass the formats file with `--formats`, or check the name. See [TL_97][sokf:errors-formats-and-examples#tl_97].
- `TL_87`: the formats file doesn't parse, or has no `registry` map. The command stops with exit code 2. A fault inside the registry, such as a type that isn't valid, stops the command with the key of that fault. See [TL_87][sokf:errors-command#tl_87].
- `TL_40`: a format with one shape has no `example`. See [TL_40][sokf:errors-formats-and-examples#tl_40].
- `TL_110`: a format with `variants` also declares an `example` beside them. See [TL_110][sokf:errors-formats-and-examples#tl_110].
- `TL_41`: an entry with `patch: true` names a format that no farther layer defines. Remove `patch`, or fix the name. See [TL_41][sokf:errors-formats-and-examples#tl_41].
- `TL_43`: the example of one variant also matches another variant. Change the example or the shapes. See [TL_43][sokf:errors-formats-and-examples#tl_43].
- `TL_44`: a line matches the shape of a format with a key but contains no key. Add the key, or set `key-optional` where the key is optional. See [TL_44][sokf:errors-formats-and-examples#tl_44].
- `TL_65`: the example of a format fails its own shape, for example when a schema changes a type that the format uses. See [TL_65][sokf:errors-examples#tl_65].
- `TL_113`: a rule declares `key` beside a format that captures a key. Remove the `key` of the rule. See [TL_113][sokf:errors-schema-types-keys-lifecycle#tl_113].

## See also

- [Schema keys: `format`][sokf:reference-schema-keys#format]
- [Schema keys: `format-variant`][sokf:reference-schema-keys#format-variant]
- [The command][sokf:reference-cli]
- [How the grammar layers fit together][sokf:explanation-grammar-layers]
- [Section 5 of the core specification][sokf:tramlines:tramlines-spec#5-formats]

<!-- sokf:links -->

[sokf:errors-command#tl_87]: /docs/reference/errors-command.md#tl_87
[sokf:errors-examples#tl_65]: /docs/reference/errors-examples.md#tl_65
[sokf:errors-formats-and-examples#tl_110]: /docs/reference/errors-formats-and-examples.md#tl_110
[sokf:errors-formats-and-examples#tl_40]: /docs/reference/errors-formats-and-examples.md#tl_40
[sokf:errors-formats-and-examples#tl_41]: /docs/reference/errors-formats-and-examples.md#tl_41
[sokf:errors-formats-and-examples#tl_43]: /docs/reference/errors-formats-and-examples.md#tl_43
[sokf:errors-formats-and-examples#tl_44]: /docs/reference/errors-formats-and-examples.md#tl_44
[sokf:errors-formats-and-examples#tl_97]: /docs/reference/errors-formats-and-examples.md#tl_97
[sokf:errors-schema-types-keys-lifecycle#tl_113]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_113
[sokf:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:how-to-ban-text]: /docs/how-to/ban-text.md
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-schema-keys#format]: /docs/reference/schema-keys.md#format
[sokf:reference-schema-keys#format-variant]: /docs/reference/schema-keys.md#format-variant
[sokf:tramlines:tramlines-spec#5-formats]: /knowledge/spec/tramlines-spec.md#5-formats
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
