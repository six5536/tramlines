---
type: Explanation
id: explanation-block-matching
title: How blocks are matched to a schema
description: How tramlines pairs the blocks of a section with the blocks that its schema declares, and how that pairing determines which block an error lands on and which kind it names.
order: 40
---

# How blocks are matched to a schema

An error can name a block that you didn't expect, or report `expected a code block; found a prose block`. Such an error follows from how tramlines matches the blocks of a section to the blocks that the schema declares. tramlines matches them one position at a time, in order. Each position takes a block of its own kind. A position gives the block back only when the block fails it and the schema made the position optional, repeating, or a group. With these steps, you can tell which rule an error comes from.

## What a block is

tramlines reads the body of a document as CommonMark, with GitHub tables, footnotes, and task lists. It splits each section into blocks. A few conventions decide where one block ends and what counts as a block:

- Consecutive paragraphs are one `prose` block. Each paragraph is one unit of that block.
- A paragraph that contains only images is an `image` block.
- Two lists with the same marker, one after the other, are one list.
- Generated blocks, the interior of a host's region, footnote definitions, and HTML comments aren't matched at all.

The kinds are `prose`, `bullet-list`, `numbered-list`, `table`, `code`, `blockquote`, `image`, `html`, `thematic-break`, `references`, and `group`. A host can add a region kind. Each block consists of units: the paragraphs of prose or a quote, the items of a list, the rows of a table, and the repetitions of a group.

## Positions

Each entry under a section's `blocks` is a position. The body must contain exactly the blocks that its positions declare, in order, unless the section sets `blocks-ordered: false`. A section with no `blocks` has one position, a prose block.

The engine walks the blocks of the section and the positions together. For each block, the kind comes first: a position takes only a block of its own kind.

## A block of a kind no position expects

When no later position declares the kind of a block, tramlines reports the block once and carries on past it. The block after it still meets its own position, and tramlines reports no further error for the unexpected block.

This schema declares one code block in a recipe's method:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Method
    level: 1
    blocks:
      - kind: code
        lang: text
        example: Whisk the eggs into the flour.
```

The document contains a paragraph before its code block:

**`pancakes.md`**

````markdown
# Method

Whisk well.

```text
Whisk the eggs into the flour.
```
````

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:3: error[TLMD_39]: unexpected prose block (schema recipe, section "Method")
1 error, 0 warnings
[exit 1]
```

When no code block follows, the paragraph is all the position meets. The error then names both kinds in one finding, the kind that the position expected and the kind that it found:

**`pancakes.md`**

```markdown
# Method

Whisk the eggs into the flour.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:3: error[TLMD_69]: expected a code block; found a prose block (schema recipe, section "Method" block 1)
1 error, 0 warnings
[exit 1]
```

## Claims

When a block meets a position of its kind, the position checks it and either claims it or releases it. The choice depends on how the schema declared the position.

**A required position that occurs once claims its block, whatever the check reports.** The schema requires the block, so tramlines takes a block of that kind in that place as the required block, even when it's malformed. The errors land on that block, at its lines.

**An optional, repeating, or group position claims a block only when the check reports no error.** When the check fails, the position releases the block. The engine throws away what the check found and offers the block to the later positions that declare its kind. When the check reports only warnings, the position still claims the block.

The second rule lets a schema declare two positions of one kind that differ by their rules. This schema allows an optional tip before a required warning, and both are blockquotes:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Method
    level: 1
    blocks:
      - kind: blockquote
        optional: true
        each:
          template: 'Tip: ${}'
          example: 'Tip: rest the batter for an hour.'
      - kind: blockquote
        each:
          template: 'Warning: ${}'
          example: 'Warning: the pan is hot.'
```

The document contains only the warning. The optional tip position meets the quote first. The quote fails the tip rule, so the tip position releases it. The warning position then receives the quote that the schema wrote it for:

**`pancakes.md`**

```markdown
# Method

> Warning: the pan is hot.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

When a quote fits neither position, the tip releases it and the warning claims it, because the warning is required once. The error comes from the warning's rule:

**`pancakes.md`**

```markdown
# Method

> Note: the pan is hot.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:3: error[TL_74]: `Note: the pan is hot.` fails at column 1: expected `Warning: `, found `Note: the pan is hot.`; a paragraph of this kind reads Warning: the pan is hot. (schema recipe, section "Method" block 2)
1 error, 0 warnings
[exit 1]
```

The same release lets a schema declare a repeating `yaml` code block before a `text` one, or a group before a block of the group's first inner kind. Each block goes to the position written for it.

### When a released block finds no home

A released block isn't lost. When no later position claims a released block, the engine reports the error that made the last position release it, at that block. When a position releases a block of its own kind and then ends up with no block, the engine reports why the position refused that block, not only that the block is missing.

The claim test is the same whether the section fixes the block order or sets `blocks-ordered: false`.

## Groups

A `group` repeats a sequence of blocks as one unit, such as a step and its quote, then the next step and its quote. A group always repeats. It takes every repetition it can, and its `min` and `max` bound how many.

When a repetition is incomplete, the engine releases every block that the repetition claimed, so that later positions can take them. When releasing the blocks would hide the cause, the engine keeps them in the group and reports why the repetition failed. That happens in three cases: the group needs the repetition to reach its `min`, no later position takes the opening block of the repetition, or the group's own rule accepted that opening block. When a block fits an inner block other than the first, and no later position takes it, the engine reads it as a repetition whose opening blocks are missing. The error names the missing kind.

This schema describes a method as steps. Each step is a paragraph followed by a quote with a tip:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Method
    level: 1
    blocks:
      - kind: group
        blocks:
          - kind: prose
            max: 1
            each:
              template: 'Step ${:int}: ${}'
              example: 'Step 1: whisk the eggs into the flour.'
          - kind: blockquote
            max: 1
            example: Use a fork if you have no whisk.
```

The second step has no tip. Its paragraph matches the group's first inner block, so the group keeps it and reports the missing quote. In the error, `block 1.2` names the second inner block of the first position:

**`pancakes.md`**

```markdown
# Method

Step 1: whisk the eggs into the flour.

> Use a fork if you have no whisk.

Step 2: fry the batter.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:7: error[TLMD_152]: expected a blockquote block (schema recipe, section "Method" block 1.2)
1 error, 0 warnings
[exit 1]
```

In this document the second tip has no step. The quote fits the group's second inner block, so the engine reads it as a repetition whose opening paragraph is missing:

**`pancakes.md`**

```markdown
# Method

Step 1: whisk the eggs into the flour.

> Use a fork if you have no whisk.

> Use butter, not oil.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:7: error[TLMD_69]: expected a prose block; found a blockquote block (schema recipe, section "Method" block 1.1)
1 error, 0 warnings
[exit 1]
```

## Extents: dividing a run between rules

Some runs of units are one block even when the schema declares separate positions for them. Consecutive paragraphs are one prose block, and two consecutive lists with the same marker are one list. When a section declares two or more positions of that kind in a fixed order, the positions divide the run between them. A `group` that opens and closes with a position of that kind counts its next repetition as one more position, so its repetitions divide a run too. A `group` that opens with a position of that kind counts as one more position, whatever closes it. Sometimes the block after the run fits such a group only after the group's opening, such as a code block after the sentence that introduces it. The run then leaves to the group the units that the group's opening needs.

The first position takes its extent. The extent is the longest run of units that passes the position's rule with no error, within its `min` and `max`. The next position starts where the first one stops. When a unit is left over after the last position that allows it, tramlines reports the unit at its own line. An optional block between the two positions doesn't change the division of the run. A position with no `max` and no shape takes the whole run. So tramlines can't derive a scaffold from a schema that puts another position of the same kind directly after it, and reports [TLMD_149][sokf:errors-scaffolds#tlmd_149].

This schema gives a recipe a paragraph that states how many people the recipe serves, then its method:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Pancakes
    level: 1
    blocks:
      - kind: prose
        max: 2
        each:
          template: 'Serves ${:int}.'
          example: Serves 4.
      - kind: prose
        max: 2
        example: Whisk the eggs into the flour.
```

The first position allows two paragraphs. The second paragraph isn't a serving line, so the extent of the first position is one paragraph. The second position takes the other two:

**`pancakes.md`**

```markdown
# Pancakes

Serves 4.

Whisk the eggs into the flour.

Fry each pancake for a minute a side.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

A fourth paragraph is beyond the last position's `max`. tramlines reports it at its line, as a unit that remains after the last prose position:

**`pancakes.md`**

```markdown
# Pancakes

Serves 4.

Whisk the eggs into the flour.

Fry each pancake for a minute a side.

Serve warm.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:9: error[TLMD_50]: `Serve warm.` remains after the last prose position that takes the run (schema recipe, section "Pancakes")
1 error, 0 warnings
[exit 1]
```

A required position that occurs once claims its block, whatever the check reports. When no extent passes such a position, the position takes the longest extent that its bounds allow and reports the errors in that extent. A document that passes under the bounds alone never fails because of this division.

The division applies only to `prose`, `bullet-list`, and `numbered-list` blocks, only where the block order is fixed, and only where the section declares two or more positions of the kind. With `blocks-ordered: false`, a run is never divided.

## Reading a surprising error

When a block error surprises you, ask these questions in order:

1. Is the block the kind you think? Two paragraphs with nothing between them are one prose block, and two consecutive lists with the same marker are one list.
2. Which position met it first? The positions run in schema order.
3. Was that position required once? Then it kept the block, and the error is that position's.
4. Was it optional, repeating, or a group? Then it released the block on an error, and the error comes from a later position, or from the last one that tried the block.

[Read an error and fix the document][sokf:how-to-read-a-finding] walks through a finding, and [the error reference][sokf:errors-blocks] lists each block error with its fix.

## See also

- [TLMD_5][sokf:tramlines:tramlines-md-spec#tlmd_5]: consecutive paragraphs are one prose block.
- [TLMD_39][sokf:tramlines:tramlines-md-spec#tlmd_39], [TLMD_152][sokf:tramlines:tramlines-md-spec#tlmd_152], and [TLMD_41][sokf:tramlines:tramlines-md-spec#tlmd_41]: a body has exactly its declared blocks, and which blocks are exempt.
- [TLMD_50][sokf:tramlines:tramlines-md-spec#tlmd_50]: the units of a run that no position takes are reported at the first of them.
- [TLMD_169][sokf:tramlines:tramlines-md-spec#tlmd_169]: a required block that stands after a block declared later is reported once, as out of place.
- [TLMD_59][sokf:tramlines:tramlines-md-spec#tlmd_59] to [TLMD_67][sokf:tramlines:tramlines-md-spec#tlmd_67]: claims, releases, and what a released block reports.
- [TLMD_68][sokf:tramlines:tramlines-md-spec#tlmd_68] to [TLMD_70][sokf:tramlines:tramlines-md-spec#tlmd_70]: a position steps over a stray block, and a block of an unexpected kind is named with the expected kind in one error.
- [TLMD_71][sokf:tramlines:tramlines-md-spec#tlmd_71] to [TLMD_73][sokf:tramlines:tramlines-md-spec#tlmd_73]: groups.
- [TLMD_48][sokf:tramlines:tramlines-md-spec#tlmd_48] to [TLMD_51][sokf:tramlines:tramlines-md-spec#tlmd_51] and [TLMD_74][sokf:tramlines:tramlines-md-spec#tlmd_74] to [TLMD_80][sokf:tramlines:tramlines-md-spec#tlmd_80], [TLMD_193][sokf:tramlines:tramlines-md-spec#tlmd_193], and [TLMD_194][sokf:tramlines:tramlines-md-spec#tlmd_194]: extents.
- [Describe blocks and their units][sokf:how-to-describe-blocks]

<!-- sokf:links -->

[sokf:errors-blocks]: /docs/reference/errors-blocks.md
[sokf:errors-scaffolds#tlmd_149]: /docs/reference/errors-scaffolds.md#tlmd_149
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-read-a-finding]: /docs/how-to/read-a-finding.md
[sokf:tramlines:tramlines-md-spec#tlmd_152]: /knowledge/spec/tramlines-md-spec.md#tlmd_152
[sokf:tramlines:tramlines-md-spec#tlmd_169]: /knowledge/spec/tramlines-md-spec.md#tlmd_169
[sokf:tramlines:tramlines-md-spec#tlmd_193]: /knowledge/spec/tramlines-md-spec.md#tlmd_193
[sokf:tramlines:tramlines-md-spec#tlmd_194]: /knowledge/spec/tramlines-md-spec.md#tlmd_194
[sokf:tramlines:tramlines-md-spec#tlmd_39]: /knowledge/spec/tramlines-md-spec.md#tlmd_39
[sokf:tramlines:tramlines-md-spec#tlmd_41]: /knowledge/spec/tramlines-md-spec.md#tlmd_41
[sokf:tramlines:tramlines-md-spec#tlmd_48]: /knowledge/spec/tramlines-md-spec.md#tlmd_48
[sokf:tramlines:tramlines-md-spec#tlmd_5]: /knowledge/spec/tramlines-md-spec.md#tlmd_5
[sokf:tramlines:tramlines-md-spec#tlmd_50]: /knowledge/spec/tramlines-md-spec.md#tlmd_50
[sokf:tramlines:tramlines-md-spec#tlmd_51]: /knowledge/spec/tramlines-md-spec.md#tlmd_51
[sokf:tramlines:tramlines-md-spec#tlmd_59]: /knowledge/spec/tramlines-md-spec.md#tlmd_59
[sokf:tramlines:tramlines-md-spec#tlmd_67]: /knowledge/spec/tramlines-md-spec.md#tlmd_67
[sokf:tramlines:tramlines-md-spec#tlmd_68]: /knowledge/spec/tramlines-md-spec.md#tlmd_68
[sokf:tramlines:tramlines-md-spec#tlmd_70]: /knowledge/spec/tramlines-md-spec.md#tlmd_70
[sokf:tramlines:tramlines-md-spec#tlmd_71]: /knowledge/spec/tramlines-md-spec.md#tlmd_71
[sokf:tramlines:tramlines-md-spec#tlmd_73]: /knowledge/spec/tramlines-md-spec.md#tlmd_73
[sokf:tramlines:tramlines-md-spec#tlmd_74]: /knowledge/spec/tramlines-md-spec.md#tlmd_74
[sokf:tramlines:tramlines-md-spec#tlmd_80]: /knowledge/spec/tramlines-md-spec.md#tlmd_80
