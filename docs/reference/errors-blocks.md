---
type: ErrorReference
id: errors-blocks
title: 'Errors: blocks'
description: The errors that tramlines reports for the blocks of a section, their order, and the number of units in each block, and how to fix each one.
order: 50
---

# Errors: blocks

These errors come from the blocks of a section: which blocks the section contains, their order, and the number of units in each block. Sections 2 and 5 of the markdown specification define them. Those sections cover documents and blocks.

## TLMD_162

An item of a list contains a block beyond its text, such as a nested list, a code block, or a second paragraph, and the rule of the item declares no `blocks` ([TLMD_162][sokf:tramlines:tramlines-md-spec#tlmd_162]).

### Message

```text
<file>:<line>: error[TLMD_162]: the item carries content beyond its text (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_162]: the item's text reads as a <kind> block (schema <id>, <place of the rule>)
```

### Cause

The rule of the item, under `each`, `sequence`, or `rules`, declares no `blocks`, so the item can contain only its own text. The item in the document contains a block after its first paragraph, such as a nested list, a code block, a blockquote, a heading, or a further paragraph. The error reports the line of the document where that block starts. The second message reports an item whose text Markdown reads as a block. For example, `- 1.` starts a numbered list, and `- > note` starts a blockquote. The command then checks no text of the item against its rule. No item can meet a rule whose shape asks for such text, and the check of the schema's scaffold reports that rule.

### Fix

Remove the nested content from the item in the document. If items of this kind contain nested content, declare that content under `blocks` on the rule of the item in the schema.

### Example

This schema gives each item a rule with no `blocks`, and the document nests a list inside an item:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        each: { template: '${}.', example: Open the lid. }
```

**`note.md`**

```markdown
# Steps

- Open the lid.
  - Use both hands.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:4: error[TLMD_162]: the item carries content beyond its text (schema note, section "Steps" block 1)
1 error, 0 warnings
[exit 1]
```

Remove the nested list:

**`note.md`**

```markdown
# Steps

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_39

The body of a section contains a block that none of the blocks of the section claims ([TLMD_39][sokf:tramlines:tramlines-md-spec#tlmd_39]).

### Message

```text
<file>:<line>: error[TLMD_39]: unexpected <kind> block (schema <id>, <place of the rule>)
```

### Cause

The engine checks the body of a section against exactly the blocks that the section declares under `blocks`, in order unless the section sets `blocks-ordered: false`. The block in the document comes after the blocks that the schema declares, or no declared block takes its kind. A stray block can also come before a declared block, when no later block of the section declares its kind. The engine reports the stray block once, then checks the following declared block as usual.

### Fix

Remove the extra block from the document. If the section can contain the block, declare it under `blocks` in the schema, at its place among the other blocks. If the section can also leave the block out, set `optional: true` on it.

### Example

This schema declares one prose block, and the document contains a code block after the prose:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - { kind: prose, example: Buy milk. }
```

**`note.md`**

````markdown
# Note

Buy milk.

```text
milk
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_39]: unexpected code block (schema note, section "Note")
1 error, 0 warnings
[exit 1]
```

Remove the code block:

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_152

The body of a section lacks a block that the schema requires ([TLMD_152][sokf:tramlines:tramlines-md-spec#tlmd_152]).

### Message

```text
<file>:<line>: error[TLMD_152]: expected a <kind> block (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_152]: a <kind> block is missing (schema <id>, <place of the rule>)
```

### Cause

The section declares a block that doesn't set `optional: true`, and no block of the document is left to fill it. Either the body contains no block of that kind, or the blocks before it took them all. The first message reports that the body ends before the block. The second reports the same fault in a section that sets `blocks-ordered: false`. Where a block of another kind stands in its place, the engine reports [TLMD_69](#tlmd_69) instead.

### Fix

Add the missing block to the document. If the section can leave the block out, set `optional: true` on the block in the schema.

### Example

This schema declares a paragraph and then a list, and the document contains only the paragraph:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - { kind: prose, example: For the week. }
      - { kind: bullet-list, example: Milk }
```

**`note.md`**

```markdown
# Groceries

For the week.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:1: error[TLMD_152]: expected a bullet-list block (schema note, section "Groceries" block 2)
1 error, 0 warnings
[exit 1]
```

Add the list:

**`note.md`**

```markdown
# Groceries

For the week.

- Milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_146

A block contains fewer units than its `min` ([TLMD_146][sokf:tramlines:tramlines-md-spec#tlmd_146]).

### Message

```text
<file>:<line>: error[TLMD_146]: the <kind> block needs at least <min> <units>; it has <count> (schema <id>, <place of the rule>)
```

### Cause

The block sets `min`, the fewest units that it holds. A unit is a paragraph, an item, a row, an image, a definition, or a repetition, depending on the kind of block. The block in the document contains fewer units. A block with no `min` needs at least one unit.

### Fix

Add units to the block in the document. If the block can hold fewer units, lower `min` in the schema.

### Example

This schema requires at least two items, and the document lists one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - { kind: bullet-list, min: 2, example: Milk }
```

**`note.md`**

```markdown
# Groceries

- Milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_146]: the bullet-list block needs at least 2 items; it has 1 (schema note, section "Groceries" block 1)
1 error, 0 warnings
[exit 1]
```

Add an item:

**`note.md`**

```markdown
# Groceries

- Milk
- Eggs
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_160

A block contains more units than its `max` ([TLMD_160][sokf:tramlines:tramlines-md-spec#tlmd_160]).

### Message

```text
<file>:<line>: error[TLMD_160]: the <kind> block allows at most <max> <units>; it has <count> (schema <id>, <place of the rule>)
```

### Cause

The block sets `max`, the most units that it holds, and the block in the document contains more. A unit is a paragraph, an item, a row, an image, a definition, or a repetition, depending on the kind of block.

### Fix

Remove units from the block in the document. If the block can hold more units, raise `max` in the schema, or remove it.

### Example

This schema allows at most two items, and the document lists three:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Groceries
    level: 1
    blocks:
      - { kind: bullet-list, max: 2, example: Milk }
```

**`note.md`**

```markdown
# Groceries

- Milk
- Eggs
- Bread
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_160]: the bullet-list block allows at most 2 items; it has 3 (schema note, section "Groceries" block 1)
1 error, 0 warnings
[exit 1]
```

Remove an item:

**`note.md`**

```markdown
# Groceries

- Milk
- Eggs
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_147

A block that declares `sequence` ends before a position of the sequence ([TLMD_147][sokf:tramlines:tramlines-md-spec#tlmd_147]).

### Message

```text
<file>:<line>: error[TLMD_147]: position <position> of the sequence is missing (schema <id>, <place of the rule>)
```

### Cause

The `sequence` of a block gives one rule for each position, in order. A position that isn't `optional` takes the next unit and reports what the unit lacks. When the unit has the shape of a later position, the engine treats this position as missing, and the error names it. The block in the document can also end before the position.

### Fix

Add the missing unit to the document. If the block can leave the position out, set `optional: true` on that position in the schema.

### Example

This schema declares a sequence of two items, and the document contains only the first:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Contact
    level: 1
    blocks:
      - kind: bullet-list
        sequence:
          - { template: 'Name: ${}', example: 'Name: Ada' }
          - { template: 'Email: ${}', example: 'Email: ada@example.com' }
```

**`note.md`**

```markdown
# Contact

- Name: Ada
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_147]: position 2 of the sequence is missing (schema note, section "Contact" block 1)
1 error, 0 warnings
[exit 1]
```

Add the second item:

**`note.md`**

```markdown
# Contact

- Name: Ada
- Email: ada@example.com
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_161

A block that declares `sequence` contains a unit after the last position of the sequence ([TLMD_161][sokf:tramlines:tramlines-md-spec#tlmd_161]).

### Message

```text
<file>:<line>: error[TLMD_161]: `<unit>` is beyond the sequence (schema <id>, <place of the rule>)
```

### Cause

The `sequence` of a block gives one rule for each position, in order, and each position takes one unit unless it sets `repeat`. The block in the document contains more units than the positions take. The engine reports each unit beyond the last position.

### Fix

Remove the extra unit from the document. If the last position can take more than one unit, set `repeat: true` on it in the schema.

### Example

This schema declares a sequence of two items, and the document contains three:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Contact
    level: 1
    blocks:
      - kind: bullet-list
        sequence:
          - { template: 'Name: ${}', example: 'Name: Ada' }
          - { template: 'Email: ${}', example: 'Email: ada@example.com' }
```

**`note.md`**

```markdown
# Contact

- Name: Ada
- Email: ada@example.com
- Phone: 555 0100
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_161]: `Phone: 555 0100` is beyond the sequence (schema note, section "Contact" block 1)
1 error, 0 warnings
[exit 1]
```

Remove the third item:

**`note.md`**

```markdown
# Contact

- Name: Ada
- Email: ada@example.com
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_50

Paragraphs or items remain after the last block of their kind that takes part of the run ([TLMD_50][sokf:tramlines:tramlines-md-spec#tlmd_50]).

### Message

```text
<file>:<line>: error[TLMD_50]: `<unit>` remains after the last <kind> position that takes the run (schema <id>, <place of the rule>)
```

### Cause

The section, or a group, lists two or more blocks of one kind in order: prose, bullet list, or numbered list. The engine divides one run of that kind among those blocks. Each block takes what its `min` and `max` allow. A section that sets `blocks-ordered: false` divides a run only inside a group. The run in the document holds more units than those blocks take together. The error reports the first unit left over.

### Fix

Remove the units left over from the document. If the section can hold more, raise the `max` of the last block of that kind in the schema.

### Example

This schema declares two paragraphs, and the document contains three:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Pancakes
    level: 1
    blocks:
      - { kind: prose, max: 1, example: Thin pancakes for four people. }
      - { kind: prose, max: 1, example: Mix the batter and fry it. }
```

**`note.md`**

```markdown
# Pancakes

Thin pancakes for four people.

Mix the batter and fry it.

Serve warm.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:7: error[TLMD_50]: `Serve warm.` remains after the last prose position that takes the run (schema note, section "Pancakes")
1 error, 0 warnings
[exit 1]
```

Remove the third paragraph:

**`note.md`**

```markdown
# Pancakes

Thin pancakes for four people.

Mix the batter and fry it.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_169

A block in the document stands after a block that the schema declares later ([TLMD_169][sokf:tramlines:tramlines-md-spec#tlmd_169]).

### Message

```text
<file>:<line>: error[TLMD_169]: the <kind> block is out of place; the blocks are in the order the schema lists them (schema <id>, <place of the rule>)
```

### Cause

The section lists its blocks in order and doesn't set `blocks-ordered: false`. A required block of the section comes after a block that the section lists later. No later block of the section has its kind. The engine reports the block once, at its own line, and checks it against the rule that declares it.

### Fix

Move the block in the document to its place in the order of the schema. If the blocks of the section can come in any order, set `blocks-ordered: false` on the section in the schema.

### Example

This schema declares a paragraph, then a code block:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: prose, example: The port to serve on. }
      - { kind: code, lang: yaml, example: 'port: 80' }
```

The document puts the code block first:

**`note.md`**

````markdown
# Config

```yaml
port: 80
```

The port to serve on.
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:7: error[TLMD_169]: the prose block is out of place; the blocks are in the order the schema lists them (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Put the paragraph first:

**`note.md`**

````markdown
# Config

The port to serve on.

```yaml
port: 80
```
````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_69

The schema expects a block of one kind where the document has a block of another kind, and no block of the expected kind follows ([TLMD_69][sokf:tramlines:tramlines-md-spec#tlmd_69]).

### Message

```text
<file>:<line>: error[TLMD_69]: expected a <kind> block; found a <kind> block (schema <id>, <place of the rule>)
```

### Cause

The section declares a block of one kind. Where that block belongs, the document contains a block of another kind that no later rule declares, and no block of the declared kind comes after it. The engine names both kinds in one error and doesn't report the found block again.

### Fix

Replace the block in the document with a block of the expected kind. If the other kind belongs in the section, change `kind` in the schema.

### Example

This schema declares one code block, and the document contains a paragraph in its place:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: code, lang: yaml, example: 'port: 80' }
```

**`note.md`**

```markdown
# Config

The port to serve on.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_69]: expected a code block; found a prose block (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Replace the paragraph with a code block:

**`note.md`**

````markdown
# Config

```yaml
port: 80
```
````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_146]: /knowledge/spec/tramlines-md-spec.md#tlmd_146
[sokf:tramlines:tramlines-md-spec#tlmd_147]: /knowledge/spec/tramlines-md-spec.md#tlmd_147
[sokf:tramlines:tramlines-md-spec#tlmd_152]: /knowledge/spec/tramlines-md-spec.md#tlmd_152
[sokf:tramlines:tramlines-md-spec#tlmd_160]: /knowledge/spec/tramlines-md-spec.md#tlmd_160
[sokf:tramlines:tramlines-md-spec#tlmd_161]: /knowledge/spec/tramlines-md-spec.md#tlmd_161
[sokf:tramlines:tramlines-md-spec#tlmd_162]: /knowledge/spec/tramlines-md-spec.md#tlmd_162
[sokf:tramlines:tramlines-md-spec#tlmd_169]: /knowledge/spec/tramlines-md-spec.md#tlmd_169
[sokf:tramlines:tramlines-md-spec#tlmd_39]: /knowledge/spec/tramlines-md-spec.md#tlmd_39
[sokf:tramlines:tramlines-md-spec#tlmd_50]: /knowledge/spec/tramlines-md-spec.md#tlmd_50
[sokf:tramlines:tramlines-md-spec#tlmd_69]: /knowledge/spec/tramlines-md-spec.md#tlmd_69
