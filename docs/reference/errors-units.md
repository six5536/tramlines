---
type: ErrorReference
id: errors-units
title: 'Errors: units'
description: The errors that tramlines reports for the paragraphs, items, and rows that the rules of a block select and count, and how to fix each one.
order: 60
---

# Errors: units

These errors come from the rules that a block applies to its units. Sections 5 and 6 of the markdown specification define these errors: section 5 covers blocks, and section 6 covers units and lines.

## TLMD_53

A unit of a block matches none of the `rules` of the block ([TLMD_53][sokf:tramlines:tramlines-md-spec#tlmd_53]).

### Message

```text
<file>:<line>: error[TLMD_53]: `<unit>` matches no rule of the block; declare it or set `units-open` (schema <id>, <place of the rule>)
```

### Cause

The block declares `rules`, and these rules work as a grammar: every unit must go to a rule. No rule takes this unit, and the unit doesn't start to match any rule with a shape. A rule with only `select-pattern` and a count takes no unit. The engine therefore also reports a unit that only such a rule selects. When a unit partly matches a rule, the engine reports [TLMD_57](#tlmd_57) instead.

### Fix

Change the unit in the document so that a rule takes it, or remove it. If the block can contain units of this kind, add a rule for them in the schema. To accept any unit that no rule takes, set `units-open: true` on the block instead.

### Example

This schema accepts items that start with `STEP`, and the document contains an item that doesn't:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: 'STEP ${}', example: STEP Open the lid. }
```

**`note.md`**

```markdown
# Steps

- STEP Open the lid.
- Close it.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:4: error[TLMD_53]: `Close it.` matches no rule of the block; declare it or set `units-open` (schema note, section "Steps" block 1)
1 error, 0 warnings
[exit 1]
```

Start the item with `STEP`:

**`note.md`**

```markdown
# Steps

- STEP Open the lid.
- STEP Close it.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_57

A unit or a line matches part of a rule and then stops, and no rule takes it ([TLMD_57][sokf:tramlines:tramlines-md-spec#tlmd_57]).

### Message

```text
<file>:<line>: error[TLMD_57]: `<unit>` matches no rule of the block; the nearest <failure>; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_57]: `<unit>` matches no rule of the fence; the nearest <failure>; a <unit> of this kind reads <example> (schema <id>, <place of the rule>)
```

### Cause

The block declares `rules`, or a rule of a code block declares `lines`, and no rule takes the unit or the line. The engine finds the rule whose shape the text matches furthest. It reports where the text stops matching that rule, and it gives the example of the rule. The message names `the block` for a unit and `the fence` for a line of a code block.

### Fix

Change the text in the document so that it matches the rule, as the example shows. Start at the column that the message names. If the text is right, add a rule to the schema that takes it, or set `units-open` or `lines-open`.

### Example

This schema accepts lines of the form `name=value` in a code block, and the document contains a line with a colon:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - kind: code
        lang: text
        each:
          lines:
            - { template: '${:word}=${}', example: port=80 }
```

**`note.md`**

````markdown
# Config

```text
port=80
host: example.com
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_57]: `host: example.com` matches no rule of the fence; the nearest fails at column 5: expected `=`, found `: example.com`; a line of this kind reads port=80 (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Write the line in the form of the example:

**`note.md`**

````markdown
# Config

```text
port=80
host=example.com
```
````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_58

A rule under `rules` or `lines` can take nothing, because an earlier rule takes its example ([TLMD_58][sokf:tramlines:tramlines-md-spec#tlmd_58]).

### Message

```text
<file>:<line>: error[TLMD_58]: the example `<example>` is taken by the rule at line <n> first, so this rule takes nothing: order the rules from the narrowest (schema <id>, <place of the rule>)
```

### Cause

The error is on the schema. Under `rules` or `lines`, a rule with a shape and no `select-pattern` takes the units that its shape matches, and each unit goes to the first such rule. An earlier rule of this kind matches the example of a later one, so the later rule never takes a unit.

### Fix

In the schema, put the narrower rule first, or narrow the shape of the earlier rule so that it doesn't match the example of the later one.

### Example

In this schema, a rule that takes any item comes before a rule for notes, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: '${}', example: Open the lid. }
          - { template: 'NOTE: ${}', min: 0, example: 'NOTE: Use both hands.' }
```

**`note.md`**

```markdown
# Steps

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:9: error[TLMD_58]: the example `NOTE: Use both hands.` is taken by the rule at line 8 first, so this rule takes nothing: order the rules from the narrowest (schema note, section "Steps" block 1)
1 error, 0 warnings
[exit 1]
```

Put the rule for notes first:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: 'NOTE: ${}', min: 0, example: 'NOTE: Use both hands.' }
          - { template: '${}', example: Open the lid. }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_91

A unit rule declares `select-pattern`, `min`, or `max` where these keys don't apply ([TLMD_91][sokf:tramlines:tramlines-md-spec#tlmd_91]).

### Message

```text
<file>:<line>: error[TLMD_91]: `select-pattern` applies to a rule under `rules` or `lines`, which chooses what it governs
<file>:<line>: error[TLMD_91]: `min` applies to a rule under `rules` or `lines`; elsewhere the block bounds its units
<file>:<line>: error[TLMD_91]: `max` applies to a rule under `rules` or `lines`; elsewhere the block bounds its units
```

### Cause

The error is on the schema. A rule under `rules` or `lines` selects or counts units, so it accepts a guard, `min`, and `max`. A rule under `each` or `sequence` applies to units that the block already bounds, so none of these keys apply to it.

### Fix

In the schema, move `min` or `max` from the unit rule to the block. To select units by a guard, move the rule under `rules`.

### Example

This schema sets `max` on the rule of `each`, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - kind: bullet-list
        each: { template: '${}', max: 3, example: Milk }
```

**`note.md`**

```markdown
# Groceries

- Milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:7: error[TLMD_91]: `max` applies to a rule under `rules` or `lines`; elsewhere the block bounds its units
1 error, 0 warnings
[exit 1]
```

Move `max` to the block:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - kind: bullet-list
        max: 3
        each: { template: '${}', example: Milk }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_92

A rule with a `select-pattern` doesn't state its `min` ([TLMD_92][sokf:tramlines:tramlines-md-spec#tlmd_92]).

### Message

```text
<file>:<line>: error[TLMD_92]: a guarded rule states `min`: `min: 0` governs such units without demanding one, and a number demands that many
```

### Cause

The error is on the schema. A rule under `rules` or `lines` has no default `min` when it declares `select-pattern`. A guard selects the units that the rule applies to, but doesn't require that any such unit exists.

### Fix

In the schema, add `min` to the rule: `min: 0` to apply the rule to the units that the guard selects without requiring any, or a number to require that many.

### Example

This schema bans items that start with `TODO` but doesn't state `min`, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: '${}', example: Open the lid. }
          - { select-pattern: '^TODO', max: 0 }
```

**`note.md`**

```markdown
# Steps

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:9: error[TLMD_92]: a guarded rule states `min`: `min: 0` governs such units without demanding one, and a number demands that many
1 error, 0 warnings
[exit 1]
```

Add `min: 0`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: '${}', example: Open the lid. }
          - { select-pattern: '^TODO', min: 0, max: 0 }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_142

A rule declares `text` beside `select-pattern` ([TLMD_142][sokf:tramlines:tramlines-md-spec#tlmd_142]).

### Message

```text
<file>:<line>: error[TLMD_142]: `text` beside `select-pattern`: the repair rewrites the unit whole and would drop what the guard selected on
```

### Cause

The error is on the schema. The key `text` specifies the exact text of a unit, and a repair rewrites the unit to match it. A guard selects units by how they start. A repair would therefore drop the text that the guard matched.

### Fix

In the schema, remove `select-pattern` from the rule, or replace `text` with a `template` or a `pattern`.

### Example

This schema selects the item that starts with `Done` and specifies its exact text, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { select-pattern: '^Done', min: 1, text: Done. }
```

**`note.md`**

```markdown
# Steps

- Done.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TLMD_142]: `text` beside `select-pattern`: the repair rewrites the unit whole and would drop what the guard selected on
1 error, 0 warnings
[exit 1]
```

Remove `select-pattern`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { text: Done. }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_93

A rule under `rules` or `lines` takes or counts fewer units than its `min` ([TLMD_93][sokf:tramlines:tramlines-md-spec#tlmd_93]).

### Message

```text
<file>:<line>: error[TLMD_93]: <selector>: at least <min> are needed; there are <count> (schema <id>, <place of the rule>)
```

### Cause

The rule sets `min`, the fewest units that it takes or counts, and the block contains fewer. The selector names the units by the guard or the shape of the rule, such as ``the items of the template `NOTE: ${}` `` or ``the lines of the template `${:word}=${}` ``.

### Fix

Add units of this kind to the document. If the block can contain fewer, lower `min` on the rule in the schema.

### Example

This schema requires at least two items of the shape `NOTE: ${}`, and the document contains one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: 'NOTE: ${}', min: 2, example: 'NOTE: The lid is hot.' }
          - { template: '${}', example: Open the lid. }
```

**`note.md`**

```markdown
# Steps

- Open the lid.
- NOTE: The lid is hot.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_93]: the items of the template `NOTE: ${}`: at least 2 are needed; there is 1 (schema note, section "Steps" block 1)
1 error, 0 warnings
[exit 1]
```

Add a second note:

**`note.md`**

```markdown
# Steps

- Open the lid.
- NOTE: The lid is hot.
- NOTE: Wear gloves.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_159

A rule under `rules` or `lines` takes or counts more units than its `max` ([TLMD_159][sokf:tramlines:tramlines-md-spec#tlmd_159]).

### Message

```text
<file>:<line>: error[TLMD_159]: <selector>: at most <max> are allowed; there are <count> (schema <id>, <place of the rule>)
```

### Cause

The rule sets `max`, the most units that it takes or counts, and the block contains more. The selector names the units by the guard or the shape of the rule. The error points at the first unit over the limit. A rule with `max: 0` bans the units that it selects.

### Fix

Remove units of this kind from the document. If the block can contain more, raise `max` on the rule in the schema.

### Example

This schema bans items that start with `TODO`, and the document contains one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: '${}', example: Open the lid. }
          - { select-pattern: '^TODO', min: 0, max: 0 }
```

**`note.md`**

```markdown
# Steps

- Open the lid.
- TODO close it
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:4: error[TLMD_159]: the items matching `^TODO`: at most 0 are allowed; there is 1 (schema note, section "Steps" block 1)
1 error, 0 warnings
[exit 1]
```

Remove the item:

**`note.md`**

```markdown
# Steps

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_142]: /knowledge/spec/tramlines-md-spec.md#tlmd_142
[sokf:tramlines:tramlines-md-spec#tlmd_159]: /knowledge/spec/tramlines-md-spec.md#tlmd_159
[sokf:tramlines:tramlines-md-spec#tlmd_53]: /knowledge/spec/tramlines-md-spec.md#tlmd_53
[sokf:tramlines:tramlines-md-spec#tlmd_57]: /knowledge/spec/tramlines-md-spec.md#tlmd_57
[sokf:tramlines:tramlines-md-spec#tlmd_58]: /knowledge/spec/tramlines-md-spec.md#tlmd_58
[sokf:tramlines:tramlines-md-spec#tlmd_91]: /knowledge/spec/tramlines-md-spec.md#tlmd_91
[sokf:tramlines:tramlines-md-spec#tlmd_92]: /knowledge/spec/tramlines-md-spec.md#tlmd_92
[sokf:tramlines:tramlines-md-spec#tlmd_93]: /knowledge/spec/tramlines-md-spec.md#tlmd_93
