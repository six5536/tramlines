---
type: Tutorial
id: tutorial-first-schema
title: Your first schema
description: Write a schema for a recipe, one rule at a time, and check recipes against it.
order: 10
---

# Your first schema

In this tutorial you write a schema for a recipe, one rule at a time, and check a recipe against it after each step. At the end you can write a schema for a document of your own, read the errors that tramlines prints, and make tramlines validate the examples in a schema.

A recipe consists of a title, a short introduction, a list of ingredients, a numbered method, and sometimes a few notes. The schema describes each of these parts.

## Before you begin

- tramlines installed, as the [README](https://github.com/six5536/tramlines#installation) shows. Run `tramlines --help` to check that the command works.
- An empty directory to work in. You create every file in this tutorial in that directory.

## Step 1: write the title heading

A schema is a YAML file. Every schema declares `line-limit`, the maximum number of lines in a document. This schema also declares the frontmatter and the sections of a recipe.

Create the schema. The frontmatter contains one key, `title`. The first section starts with a level-1 heading whose text comes from that `title`, written as `{ from: title }`. A `prose` block of at most one paragraph follows the heading:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
```

Each rule includes an `example`: a line that a real recipe would contain. tramlines validates each example against its rule, and shows the example in every error from that rule.

Create a recipe:

**`pancakes.md`**

```markdown
---
title: Pancakes
---

# Pancakes

Thin pancakes for a weekend breakfast.
```

Check the recipe against the schema:

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

- `0 errors, 0 warnings` means that the recipe follows the schema.
- tramlines validates the schema first. If the schema is wrong, the command prints the errors in the schema and stops before it checks the recipe.

## Step 2: add the required sections

A recipe needs its ingredients and its method. Add a level-2 section `Ingredients` with a bullet list, and a level-2 section `Method` with a numbered list. A schema requires every section that it declares, unless the section is marked optional:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        example: 200 g flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
```

Check the recipe from step 1 again. The recipe doesn't contain either section yet:

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md: error[TLMD_18]: section `## Ingredients` is missing (schema recipe, section "Ingredients")
pancakes.md: error[TLMD_18]: section `## Method` is missing (schema recipe, section "Method")
2 errors, 0 warnings
[exit 1]
```

- Each error line names the file, the error key in brackets, and a message.
- The end of each message names the schema rule that the recipe breaks: `section "Ingredients"`.
- The exit code is 1, because the check found errors.

Add both sections to the recipe:

**`pancakes.md`**

```markdown
---
title: Pancakes
---

# Pancakes

Thin pancakes for a weekend breakfast.

## Ingredients

- 200 g flour
- 2 medium eggs
- 300 ml milk

## Method

1. Whisk the flour, eggs and milk into a smooth batter.
2. Fry a ladle of batter for a minute on each side.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

## Step 3: give each ingredient a shape

The schema accepts any bullet item as an ingredient. Make each item an amount, a unit, and the food. A `template` is a sample line with named holes where the text varies. In the following schema, `${amount:int}` is a whole number, `${unit:word}` is one word, and `${item}` is any text. The `each` key applies the rule to every item of the list. Its `example` must match the template:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${amount:int} ${unit:word} ${item}'
          example: 200 g flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

- Every ingredient of the recipe matches the template, so the check passes.
- [Built-in types][sokf:reference-types] describes the types that a hole can take, and how to define your own.

## Step 4: add an optional section

Some recipes include notes. Add a section `Notes` with `optional: true`. A recipe can omit the section. A recipe that includes the section must follow its rule:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${amount:int} ${unit:word} ${item}'
          example: 200 g flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
```

The pancake recipe has no notes and still passes:

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

Write a second recipe that includes notes:

**`porridge.md`**

```markdown
---
title: Porridge
---

# Porridge

Oats cooked slowly in milk.

## Ingredients

- 50 g oats
- 300 ml milk

## Method

1. Simmer the oats in the milk for five minutes, stirring often.

## Notes

Use water instead of milk for a lighter porridge.
```

Check both recipes in one run:

```console
$ tramlines validate --schema recipe.yaml pancakes.md porridge.md
0 errors, 0 warnings
```

## Step 5: fix a recipe that breaks the schema

Write a recipe with three mistakes: an ingredient with no amount, a `Steps` section that the schema doesn't declare, and no `Method` section:

**`cake.md`**

```markdown
---
title: Lemon cake
---

# Lemon cake

A light sponge with a sharp lemon glaze.

## Ingredients

- 200 g flour
- a pinch of salt

## Steps

1. Beat the butter and sugar until pale.
```

```console
$ tramlines validate --schema recipe.yaml cake.md
cake.md: error[TLMD_18]: section `## Method` is missing (schema recipe, section "Method")
cake.md:12: error[TL_74]: `a pinch of salt` fails at column 1: expected an `int` (`\d+`), found `a pinch of salt`; an item of this kind reads 200 g flour (schema recipe, section "Ingredients" block 1)
cake.md:14: error[TLMD_19]: heading `## Steps` is not declared (schema recipe)
3 errors, 0 warnings
[exit 1]
```

- `cake.md:12` points to the ingredient's line in the recipe. The error names three things: the column where the line stops matching the template, what the template expects there, and the example from the schema, `200 g flour`.
- `TLMD_19` reports that the schema declares no heading `Steps`. A heading that no rule declares is an error.
- The [error reference][sokf:reference-errors] explains each key, such as [TL_74][sokf:errors-lines#tl_74] and [TLMD_19][sokf:errors-sections#tlmd_19].

Give the salt an amount and a unit, and rename `Steps` to `Method`:

**`cake.md`**

```markdown
---
title: Lemon cake
---

# Lemon cake

A light sponge with a sharp lemon glaze.

## Ingredients

- 200 g flour
- 1 pinch salt

## Method

1. Beat the butter and sugar until pale.
```

```console
$ tramlines validate --schema recipe.yaml cake.md
0 errors, 0 warnings
```

## Step 6: check the examples in the schema

tramlines validates the schema before it checks any recipe. It checks each `example` against its rule, so that every example shows a line that its rule accepts.

Change the ingredient example to `flour`, which has no amount:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${amount:int} ${unit:word} ${item}'
          example: flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
recipe.yaml:17: error[TL_64]: the example `flour` fails at column 1: expected an `int` (`\d+`), found `flour` (schema recipe, section "Ingredients" block 1)
1 error, 0 warnings
[exit 1]
```

- `TL_64` points to the line of the example in the schema and explains why the example fails its rule.
- The command reports the errors in the schema and stops. It checks `pancakes.md` only after the schema is right.

A schema can also contain whole documents of its type, under `examples`. tramlines validates each one as a document. Restore the ingredient example, and add the porridge recipe as an example, with one ingredient that breaks the template:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${amount:int} ${unit:word} ${item}'
          example: 200 g flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
examples:
  - document: |
      ---
      title: Porridge
      ---

      # Porridge

      Oats cooked slowly in milk.

      ## Ingredients

      - 50 g oats
      - a pinch of salt

      ## Method

      1. Simmer the oats in the milk for five minutes, stirring often.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
recipe.yaml:31: error[TL_69]: example 1 fails: line 12: [TL_74] `a pinch of salt` fails at column 1: expected an `int` (`\d+`), found `a pinch of salt`; an item of this kind reads 200 g flour (schema recipe, section "Ingredients" block 1)
1 error, 0 warnings
[exit 1]
```

- `example 1` is the first entry of `examples`, and `line 12` is the line inside that example document.

Fix the example document in the schema, as you fixed the cake:

**`recipe.yaml`**

```yaml
line-limit: 100
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast, ready in twenty minutes.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${amount:int} ${unit:word} ${item}'
          example: 200 g flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
examples:
  - document: |
      ---
      title: Porridge
      ---

      # Porridge

      Oats cooked slowly in milk.

      ## Ingredients

      - 50 g oats
      - 1 pinch salt

      ## Method

      1. Simmer the oats in the milk for five minutes, stirring often.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md porridge.md cake.md
0 errors, 0 warnings
```

- The schema, its example document, and all three recipes pass.

## What's next

- [Check documents in CI][sokf:tutorial-check-documents-in-ci] shows how to run this check on every change to a repository.
- [Write the shape of a line][sokf:how-to-write-line-shapes] shows templates, patterns, and exact text.
- [Describe sections and headings][sokf:how-to-describe-sections] and [Describe blocks and their units][sokf:how-to-describe-blocks] cover the rest of a document's body.
- [Write examples and guidance][sokf:how-to-write-examples-and-guidance] shows how to write examples that help the writer of a document.
- [Strict by default][sokf:explanation-strict-by-default] explains why a schema refuses what it doesn't declare.
- [Schema keys][sokf:reference-schema-keys] lists every key that a schema can contain, such as the keys of a [section][sokf:reference-schema-keys#section] and a [block][sokf:reference-schema-keys#block].

<!-- sokf:links -->

[sokf:errors-lines#tl_74]: /docs/reference/errors-lines.md#tl_74
[sokf:errors-sections#tlmd_19]: /docs/reference/errors-sections.md#tlmd_19
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-errors]: /docs/reference/errors.md
[sokf:reference-schema-keys]: /docs/reference/schema-keys.md
[sokf:reference-schema-keys#block]: /docs/reference/schema-keys.md#block
[sokf:reference-schema-keys#section]: /docs/reference/schema-keys.md#section
[sokf:reference-types]: /docs/reference/types.md
[sokf:tutorial-check-documents-in-ci]: /docs/tutorials/check-documents-in-ci.md
