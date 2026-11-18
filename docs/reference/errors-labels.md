---
type: ErrorReference
id: errors-labels
title: 'Errors: labels'
description: The errors that tramlines reports for the labels that a schema declares and that the units of a document carry.
order: 73
---

# Errors: labels

These errors come from the labels of a schema: the keys under `labels`, the `label` of a block or a unit rule, and the label that each unit of a document carries, such as `[human]`. Section 11 of the Markdown specification defines these errors. [Label the units of a part][sokf:how-to-label-units] shows how to declare labels.

## TLMD_241

A block or a unit rule declares `label`, and its schema declares no `labels` ([TLMD_241][sokf:tramlines:tramlines-md-spec#tlmd_241]).

### Message

```text
<file>:<line>: error[TLMD_241]: `label` on a rule of a schema that declares no `labels`: declare the key of each label under `labels`
```

### Cause

The error is on the schema. A unit carries one of the labels of its schema, and a schema with no `labels`, or with an empty list, has none for it to carry.

### Fix

In the schema, declare the key of each label under `labels`, such as `labels: [human, agent]`, or remove `label` from the rule.

### Example

This schema requires a label on each decision, and declares no labels:

**`decisions.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Decisions
    level: 1
    blocks:
      - kind: bullet-list
        label: required
        example: '[human] The parser reads one line at a time.'
```

**`decisions.md`**

```markdown
# Decisions

- [human] The parser reads one line at a time.
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
decisions.yaml:6: error[TLMD_241]: `label` on a rule of a schema that declares no `labels`: declare the key of each label under `labels`
1 error, 0 warnings
[exit 1]
```

Declare the labels:

**`decisions.yaml`**

```yaml
line-limit: 0
labels: [human, agent]
sections:
  - heading: Decisions
    level: 1
    blocks:
      - kind: bullet-list
        label: required
        example: '[human] The parser reads one line at a time.'
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
0 errors, 0 warnings
```

## TLMD_249

A unit whose rule declares `label: required` carries no label ([TLMD_249][sokf:tramlines:tramlines-md-spec#tlmd_249]).

### Message

```text
<file>:<line>: error[TLMD_249]: `<text>` carries no label; its rule requires one: `[<words>]` or `[<words>]`
```

### Cause

The rule of the unit, or the block that holds it, requires a label on each of its units, and the unit has the shape of its rule but carries none. The label stands straight after the key of a unit that begins with the key that its rule captures, and otherwise at the start of the unit, after the box of a task item. A label written elsewhere is no label, and the unit then fails its shape instead.

### Fix

Write one of the labels that the message lists at its place in the unit.

### Example

This schema requires a label after the key of each decision, and the second decision carries none:

**`decisions.yaml`**

```yaml
line-limit: 0
labels: [human, agent]
types:
  decision-id: 'D-[1-9][0-9]*'
sections:
  - heading: Decisions
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${id:decision-id} ${}, because ${}.'
          key: id
          label: required
          example: 'D-1 [human] The parser reads one line at a time, because a line is a unit.'
```

**`decisions.md`**

```markdown
# Decisions

- <a id="d-1"></a>D-1 [human] The parser reads one line at a time, because a line is a unit.
- <a id="d-2"></a>D-2 The report names the line, because a writer looks for it.
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
decisions.md:4: error[TLMD_249]: `D-2 The report names the line, because a writer looks for i…` carries no label; its rule requires one: `[human]` or `[agent]` (schema decisions, section "Decisions" block 1)
1 error, 0 warnings
[exit 1]
```

Write the label after the key:

**`decisions.md`**

```markdown
# Decisions

- <a id="d-1"></a>D-1 [human] The parser reads one line at a time, because a line is a unit.
- <a id="d-2"></a>D-2 [agent] The report names the line, because a writer looks for it.
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
0 errors, 0 warnings
```

## TLMD_250

A unit whose rule declares no `label` carries a label of its schema ([TLMD_250][sokf:tramlines:tramlines-md-spec#tlmd_250]).

### Message

```text
<file>:<line>: error[TLMD_250]: `<text>` carries the label `[<words>]`, and its rule declares no `label`
```

### Cause

The unit carries a label of its schema, its words or its key, and neither its rule nor its block declares `label`, so its units carry no label. Only the words of a label of the schema read as a label there, so other words in square brackets, such as `[sic]`, stay text.

### Fix

Remove the label from the unit, or, in the schema, declare `label` on the rule or on its block.

### Example

This schema labels the decisions, and not the paragraph of notes after them:

**`decisions.yaml`**

```yaml
line-limit: 0
labels: [human, agent]
sections:
  - heading: Decisions
    level: 1
    blocks:
      - kind: bullet-list
        label: required
        example: '[human] The parser reads one line at a time.'
      - kind: prose
        example: The notes say what the decisions leave open.
```

**`decisions.md`**

```markdown
# Decisions

- [human] The parser reads one line at a time.

[agent] The decisions leave the report open.
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
decisions.md:5: error[TLMD_250]: `The decisions leave the report open.` carries the label `[agent]`, and its rule declares no `label` (schema decisions, section "Decisions" block 2)
1 error, 0 warnings
[exit 1]
```

Remove the label:

**`decisions.md`**

```markdown
# Decisions

- [human] The parser reads one line at a time.

The decisions leave the report open.
```

```console
$ tramlines validate --schema decisions.yaml decisions.md
0 errors, 0 warnings
```

## TLMD_251

The label of a unit is none of the labels of its schema, in the words that the host gives for the language of the schema ([TLMD_251][sokf:tramlines:tramlines-md-spec#tlmd_251]).

### Message

```text
<file>:<line>: error[TLMD_251]: the label `[<words>]` is none of the labels of the schema: `[<words>]` or `[<words>]`
```

### Cause

A unit whose rule declares `label` carries words in square brackets at the place of its label, and the words are none of the labels that the message lists. A document accepts the words that the host gives for each label in the language of the schema, and a label's key only where the host gives no words for it. The `tramlines` command takes the words from the file that `--texts` names, and has none of its own.

### Fix

Write one of the labels that the message lists. A label written as its key, where the host gives other words for it, is repaired by `tramlines validate --fix`, which writes the host's words in its place. Any other words are yours to settle.

### Example

This schema in German labels each decision, and the file of texts gives the German words of the labels:

**`decisions.yaml`**

```yaml
line-limit: 0
language: de
labels: [human, agent]
sections:
  - heading: Entscheidungen
    level: 1
    blocks:
      - kind: bullet-list
        label: required
        example: '[Mensch] Der Parser liest eine Zeile nach der anderen.'
```

**`texts.yaml`**

```yaml
de:
  labels:
    human: Mensch
    agent: KI
```

The first decision carries the key of its label, and the second carries words of no label:

**`decisions.md`**

```markdown
# Entscheidungen

- [human] Der Parser liest eine Zeile nach der anderen.
- [Person] Der Bericht nennt die Zeile.
```

```console
$ tramlines validate --schema decisions.yaml --texts texts.yaml decisions.md
decisions.md:3: error[TLMD_251]: the label `[human]` is none of the labels of the schema: `[Mensch]` or `[KI]` (schema decisions, section "Entscheidungen" block 1)
decisions.md:4: error[TLMD_251]: the label `[Person]` is none of the labels of the schema: `[Mensch]` or `[KI]` (schema decisions, section "Entscheidungen" block 1)
2 errors, 0 warnings
[exit 1]
```

`--fix` writes `[Mensch]` in place of `[human]`, and leaves `[Person]` for you:

```console
$ tramlines validate --schema decisions.yaml --texts texts.yaml --fix decisions.md
rewrote decisions.md
decisions.md:4: error[TLMD_251]: the label `[Person]` is none of the labels of the schema: `[Mensch]` or `[KI]` (schema decisions, section "Entscheidungen" block 1)
1 error, 0 warnings
[exit 1]
```

Write the label of the second decision:

**`decisions.md`**

```markdown
# Entscheidungen

- [Mensch] Der Parser liest eine Zeile nach der anderen.
- [KI] Der Bericht nennt die Zeile.
```

```console
$ tramlines validate --schema decisions.yaml --texts texts.yaml decisions.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:how-to-label-units]: /docs/how-to/label-units.md
[sokf:tramlines:tramlines-md-spec#tlmd_241]: /knowledge/spec/tramlines-md-spec.md#tlmd_241
[sokf:tramlines:tramlines-md-spec#tlmd_249]: /knowledge/spec/tramlines-md-spec.md#tlmd_249
[sokf:tramlines:tramlines-md-spec#tlmd_250]: /knowledge/spec/tramlines-md-spec.md#tlmd_250
[sokf:tramlines:tramlines-md-spec#tlmd_251]: /knowledge/spec/tramlines-md-spec.md#tlmd_251
