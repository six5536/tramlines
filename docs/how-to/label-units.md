---
type: HowTo
id: how-to-label-units
title: Label the units of a part
description: Declare the labels that the units of a document carry, such as who wrote each one, say which parts require a label or allow one, and give their words in the language of the schema.
order: 85
---

# Label the units of a part

When each decision, criterion, or paragraph of a part needs a mark that many rules share, such as `[human]` or `[agent]` for who wrote it, declare the mark as a label. At the end, tramlines checks the label of each unit of the part in the words of the schema's language, and a new document starts with each label in place.

## Before you begin

- A schema with blocks and their unit rules. [Describe blocks and their units][sokf:how-to-describe-blocks] shows how to write one.
- A rule that captures a key, as in [Capture keys and anchor them][sokf:how-to-capture-keys], where a unit's label follows its key.

## Steps

1. List the key of each label under `labels`, at the top of the schema. A key is a slug, such as `human`. The keys mean nothing to tramlines: your schema and your program give them a meaning.

2. Set `label` on each block or unit rule whose units carry a label. `label: required` demands a label on each unit, and `label: optional` allows one. A unit rule that declares no `label` takes the one of its block, and a unit of any other part carries no label. `label` applies to the paragraphs of prose and of a block quote, the items of a list, and the rows of a table.

3. Write each example with its label, because a writer copies the example. tramlines checks the label of each example as it checks the label of a unit.

   A label is its words in square brackets followed by one space. It stands straight after the key of a unit that begins with the key that its rule captures, and otherwise at the start of the unit. In a task item, it stands after the box. The rule reads the rest of the unit as before, so a template holds no hole for the label. This schema labels the decisions, the criteria, and the paragraphs of the summary:

**`issue.yaml`**

```yaml
line-limit: 0
labels: [human, agent]
types:
  decision-id: 'D-[1-9][0-9]*'
sections:
  - heading: Summary
    level: 1
    blocks:
      - kind: prose
        label: optional
        example: '[agent] **Scope**: the parser reads one line at a time.'
  - heading: Criteria
    level: 2
    blocks:
      - kind: bullet-list
        each:
          pattern: '^\[[ x]\] WHEN .+\.$'
          label: required
          example: '[ ] [agent] WHEN a line holds a tab, the parser SHALL read it as a space.'
  - heading: Decisions
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${id:decision-id} ${}, because ${}.'
          key: id
          label: required
          example: 'D-1 [human] The parser reads one line at a time, because a line is a unit.'
```

4. Check a document. In this one, the first criterion carries its label before the box, and the second decision carries none:

**`issue.md`**

```markdown
# Summary

[human] **Scope**: the parser reads one line at a time.

A second paragraph needs no label.

## Criteria

- [agent] [ ] WHEN a line holds a tab, the parser SHALL read it as a space.
- [x] [human] WHEN a line is empty, the parser SHALL skip it.

## Decisions

- <a id="d-1"></a>D-1 [human] The parser reads one line at a time, because a line is a unit.
- <a id="d-2"></a>D-2 The report names the line, because a writer looks for it.
```

```console
$ tramlines validate --schema issue.yaml issue.md
issue.md:9: error[TL_74]: `[agent] [ ] WHEN a line holds a tab, the parser SHALL read …` does not match `^\[[ x]\] WHEN .+\.$`; an item of this kind reads `[ ] [agent] WHEN a line holds a tab, the parser SHALL read it as a space.` (schema issue, section "Criteria" block 1)
issue.md:15: error[TLMD_249]: `D-2 The report names the line, because a writer looks for i…` carries no label; its rule requires one: `[human]` or `[agent]` (schema issue, section "Decisions" block 1)
2 errors, 0 warnings
[exit 1]
```

5. Move the first label after the box, and give the second decision its label:

**`issue.md`**

```markdown
# Summary

[human] **Scope**: the parser reads one line at a time.

A second paragraph needs no label.

## Criteria

- [ ] [agent] WHEN a line holds a tab, the parser SHALL read it as a space.
- [x] [human] WHEN a line is empty, the parser SHALL skip it.

## Decisions

- <a id="d-1"></a>D-1 [human] The parser reads one line at a time, because a line is a unit.
- <a id="d-2"></a>D-2 [agent] The report names the line, because a writer looks for it.
```

```console
$ tramlines validate --schema issue.yaml issue.md
0 errors, 0 warnings
```

6. To write the labels in the language of the schema, declare its `language`, and give the words of each label under `labels` in a file of texts, the file that `--texts` names. Pass the file to `tramlines validate` and to `tramlines new`. A document then accepts only those words, and a label's key only where the file gives no words for it. `--fix` writes the words in place of a label written as its key. Without such a file, each label has its key as its words.

**`texts.yaml`**

```yaml
de:
  labels:
    human: Mensch
    agent: KI
```

7. Start a new document. The new document holds the headings and no example, and the guide that the command prints after it gives the example of each part with its label, where the rule requires a label, or where the rule allows one and the example carries one, in the words of the file that `--texts` names, or as its key:

```console
$ tramlines new --schema issue.yaml
# Summary

## Criteria

## Decisions

The guide of the new document: each part, its guidance, and its example.

Section `# Summary`:

  Block 1 (prose):
    Example:
      [agent] **Scope**: the parser reads one line at a time.

Section `## Criteria`:

  Block 1 (bullet-list):
    Example:
      - [ ] [agent] WHEN a line holds a tab, the parser SHALL read it as a space.

Section `## Decisions`:

  Block 1 (bullet-list):
    Example:
      - D-1 [human] The parser reads one line at a time, because a line is a unit.
```

A program that embeds tramlines gives the words of each label through `Host::label_words`, and names the label of a new document through `Host::scaffold_label`, as [Embed the engine in a program][sokf:how-to-embed-the-engine] shows.

## Troubleshooting

- `TLMD_241`: a rule declares `label`, and the schema declares no `labels`. List the keys under `labels`. See [TLMD_241][sokf:errors-labels#tlmd_241].
- `TLMD_249`: a unit carries no label where its rule requires one. A label before the key of a unit that begins with its key, or before the box of a task item, is no label. See [TLMD_249][sokf:errors-labels#tlmd_249].
- `TLMD_250`: a unit carries a label where its rule declares none. See [TLMD_250][sokf:errors-labels#tlmd_250].
- `TLMD_251`: a label is none of the labels of the schema in its language. See [TLMD_251][sokf:errors-labels#tlmd_251].

## See also

- [Schema keys][sokf:reference-schema-keys]
- [Errors: labels][sokf:errors-labels]
- [The command line][sokf:reference-cli]

<!-- sokf:links -->

[sokf:errors-labels]: /docs/reference/errors-labels.md
[sokf:errors-labels#tlmd_241]: /docs/reference/errors-labels.md#tlmd_241
[sokf:errors-labels#tlmd_249]: /docs/reference/errors-labels.md#tlmd_249
[sokf:errors-labels#tlmd_250]: /docs/reference/errors-labels.md#tlmd_250
[sokf:errors-labels#tlmd_251]: /docs/reference/errors-labels.md#tlmd_251
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-schema-keys]: /docs/reference/schema-keys.md
