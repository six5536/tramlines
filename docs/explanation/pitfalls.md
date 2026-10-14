---
type: Explanation
id: explanation-pitfalls
title: Pitfalls in writing a schema
description: The mistakes that schema authors make most often, why each one goes wrong, and the error or silence it causes.
order: 50
---

# Pitfalls in writing a schema

Most mistakes in a schema come from two sources: a regular expression that matches more than you meant, and a key that you left out. A key that you leave out keeps its strict default. Each of the following sections shows one pitfall, a short example, and what the check reports.

## An unanchored pattern

A `pattern` isn't anchored, so a line passes when the pattern matches anywhere in it. A pattern that you meant to match a whole line also accepts any longer text that contains a match.

The author of this schema wants each ingredient to read as a weight and a name:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
    blocks:
      - kind: bullet-list
        each:
          pattern: '[0-9]+ g'
          example: 200 g flour
```

**`pancakes.md`**

```markdown
# Ingredients

- 200 g flour
- flour, about 200 g or so
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

The second item passes, because it contains `200 g`. Anchor the ends with `^` and `$`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
    blocks:
      - kind: bullet-list
        each:
          pattern: '^[0-9]+ g [a-z]+$'
          example: 200 g flour
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:4: error[TL_74]: `flour, about 200 g or so` does not match `^[0-9]+ g [a-z]+$`; an item of this kind reads 200 g flour (schema recipe, section "Ingredients" block 1)
1 error, 0 warnings
[exit 1]
```

The same applies to a heading: a heading `pattern: 'Notes'` accepts every heading that contains the word. Write a fixed heading as a string, `heading: Notes`.

## A pattern where a template would do

tramlines matches a template whole, so a template needs no anchors. The error for a template shows where the text stops matching and what the template expects there. The error for a pattern can only quote the pattern. This schema writes the anchored pattern of the preceding section as a template:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${:int} g ${}'
          example: 200 g flour
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:4: error[TL_74]: `flour, about 200 g or so` fails at column 1: expected an `int` (`\d+`), found `flour, about 200 g or so`; an item of this kind reads 200 g flour (schema recipe, section "Ingredients" block 1)
1 error, 0 warnings
[exit 1]
```

Compare the two errors: the template error names column 1 and the `int` that it expected there. Use a pattern only for what a template can't express. When a part of a line is one of a few choices, name it once as a type, such as `unit: [g, kg, ml]`, and write `${:unit}` in the template. [Write the shape of a line][sokf:how-to-write-line-shapes] shows how.

## A guard that repeats the rule's shape

Under `rules`, a rule with a shape and no `select-pattern` already takes the units that its shape matches. A `select-pattern` that restates the shape adds a second copy to keep in step with the first. The check finds the same faults without it.

**`method.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Method
    level: 1
    blocks:
      - kind: numbered-list
        rules:
          - select-pattern: '^Step '
            template: 'Step ${:int}: ${}'
            min: 1
            example: 'Step 1: whisk the eggs.'
```

**`pancakes.md`**

```markdown
# Method

1. Step 1: whisk the eggs.
2. Fry the batter.
```

```console
$ tramlines validate --schema method.yaml pancakes.md
pancakes.md:4: error[TLMD_53]: `Fry the batter.` matches no rule of the block; declare it or set `units-open` (schema method, section "Method" block 1)
1 error, 0 warnings
[exit 1]
```

Drop the guard, and the rule selects units by its template:

**`method.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Method
    level: 1
    blocks:
      - kind: numbered-list
        rules:
          - template: 'Step ${:int}: ${}'
            min: 1
            example: 'Step 1: whisk the eggs.'
```

```console
$ tramlines validate --schema method.yaml pancakes.md
pancakes.md:4: error[TLMD_53]: `Fry the batter.` matches no rule of the block; declare it or set `units-open` (schema method, section "Method" block 1)
1 error, 0 warnings
[exit 1]
```

Keep a guard for a rule that picks its units by something other than its shape, such as a list item picked by its label to check its nested list, or a counter that bans a line.

## Lookaround in a regular expression

tramlines reads regular expressions in the Rust dialect, which has no look-ahead, no look-behind, and no backreferences. A pattern that uses one is an error in the schema:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
    blocks:
      - kind: bullet-list
        each:
          pattern: '^(?!TODO).+$'
          example: 200 g flour
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
recipe.yaml:8: error[TL_118]: `pattern` is not a valid regex: look-around, including look-ahead and look-behind, is not supported
1 error, 0 warnings
[exit 1]
```

In place of a negative look-ahead, ban the text with `prohibited-patterns`. [Ban text in a document][sokf:how-to-ban-text] shows how.

## Leaving a key out

A key that you leave out never loosens a rule. The rule keeps its strict default.

- A section with no `blocks` must contain exactly one prose block.
- A part with no `optional` is required.
- A table with no `columns-open` contains exactly its declared columns.

A section with no `blocks` causes two errors. One is in a document that has a list in that section. The other is in the schema itself, because the scaffold has no text to write for the prose block:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
```

**`pancakes.md`**

```markdown
# Ingredients

- 200 g flour
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:3: error[TLMD_69]: expected a prose block; found a bullet-list block (schema recipe, section "Ingredients" block 1)
recipe.yaml:3: error[TLMD_116]: the schema derives no scaffold: a section with no `blocks` holds one prose block, and the scaffold writes no text of its own; declare the block and its `example`
2 errors, 0 warnings
[exit 1]
```

Declare each block and its `example`. [Strict by default][sokf:explanation-strict-by-default] lists every relaxation.

## Overriding a shared format in one schema

A format can come from a file of shared formats, passed with `--formats`, or from a schema's own `formats`. The nearer one wins, and an entry in a schema replaces the shared entry whole, for that schema only. When you write into one schema a change that you meant for every schema, the other schemas keep the old format, and nothing reports the difference. Put a change meant for every schema in the shared file, and keep a schema's own entry for what that schema alone does differently. [Share line formats across schemas][sokf:how-to-share-line-formats] shows the layers.

## Copying markers from a scaffold

A scaffold is the document that a new document starts from. It contains markers: an `only` marker before each part that some states need, and a `guidance` line before each part that has guidance. The markers guide the person who fills in the scaffold. A new document is the scaffold gated to its states. It holds the parts that its states need, with no `only` markers, and it keeps the guidance lines for you to read, act on, and delete. `tramlines new` prints it. A marker that you copy by hand, or a guidance line that you leave behind, stays in the document. The check then fails at the marker's line:

**`meeting.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Agenda
    level: 1
    guidance: One item for each decision the meeting must make.
    blocks:
      - kind: bullet-list
        example: Pick the release date.
```

**`planning.md`**

```markdown
<!-- tramlines:guidance One item for each decision the meeting must make. -->
# Agenda

- Pick the release date.
```

```console
$ tramlines validate --schema meeting.yaml planning.md
planning.md:1: error[TLMD_158]: a `tramlines:guidance` line is left here: write the part it guides, then delete the line
1 error, 0 warnings
[exit 1]
```

Write the part that the line guides, then delete the line.

## An example shaped like a prompt

The `example` of a rule is the text that the scaffold writes, so every new document starts with a copy of it. An example written as a slot (`<ingredient>`), an instruction (`Add an ingredient here.`), or filler (`One item.`) passes a loose rule. The check then reports nothing, even when a document keeps the example word for word:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Ingredients
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: '${}'
          example: Add an ingredient here.
```

**`pancakes.md`**

```markdown
# Ingredients

- Add an ingredient here.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

A person or an agent who fills the scaffold imitates what it shows, and a prompt-shaped example comes back word for word. Write the line that a real document contains, such as `200 g flour`, and put what an example can't show in `guidance`. [Write examples and guidance][sokf:how-to-write-examples-and-guidance] shows how.

## See also

- [TL_16][sokf:tramlines:tramlines-spec#tl_16] and [TL_118][sokf:tramlines:tramlines-spec#tl_118]: the regular expression dialect, with no lookaround and no anchor you didn't write.
- [TL_24][sokf:tramlines:tramlines-spec#tl_24] and [TL_26][sokf:tramlines:tramlines-spec#tl_26]: a template is matched whole, and its failure names the column.
- [TL_36][sokf:tramlines:tramlines-spec#tl_36]: a nearer format replaces a farther one whole.
- [TLMD_10][sokf:tramlines:tramlines-md-spec#tlmd_10] and [TLMD_158][sokf:tramlines:tramlines-md-spec#tlmd_158]: an `only` or `guidance` marker in a document is an error.
- [TLMD_40][sokf:tramlines:tramlines-md-spec#tlmd_40] and [TLMD_116][sokf:tramlines:tramlines-md-spec#tlmd_116]: no `blocks` means one prose block, and a scaffold with no text for a part fails the schema.
- [TLMD_55][sokf:tramlines:tramlines-md-spec#tlmd_55]: a rule with a shape and no guard takes the units that its shape matches.

<!-- sokf:links -->

[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-ban-text]: /docs/how-to/ban-text.md
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:tramlines:tramlines-md-spec#tlmd_10]: /knowledge/spec/tramlines-md-spec.md#tlmd_10
[sokf:tramlines:tramlines-md-spec#tlmd_116]: /knowledge/spec/tramlines-md-spec.md#tlmd_116
[sokf:tramlines:tramlines-md-spec#tlmd_158]: /knowledge/spec/tramlines-md-spec.md#tlmd_158
[sokf:tramlines:tramlines-md-spec#tlmd_40]: /knowledge/spec/tramlines-md-spec.md#tlmd_40
[sokf:tramlines:tramlines-md-spec#tlmd_55]: /knowledge/spec/tramlines-md-spec.md#tlmd_55
[sokf:tramlines:tramlines-spec#tl_118]: /knowledge/spec/tramlines-spec.md#tl_118
[sokf:tramlines:tramlines-spec#tl_16]: /knowledge/spec/tramlines-spec.md#tl_16
[sokf:tramlines:tramlines-spec#tl_24]: /knowledge/spec/tramlines-spec.md#tl_24
[sokf:tramlines:tramlines-spec#tl_26]: /knowledge/spec/tramlines-spec.md#tl_26
[sokf:tramlines:tramlines-spec#tl_36]: /knowledge/spec/tramlines-spec.md#tl_36
