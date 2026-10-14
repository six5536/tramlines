---
type: HowTo
id: how-to-describe-sections
title: Describe sections and headings
description: List the headings that a document must contain, which of them are optional or repeat, where writers can add headings of their own, and what comes before the first heading.
order: 60
---

# Describe sections and headings

When every document of a kind has the same headings, list them under `sections` in the schema. At the end, a document with a missing heading, an extra heading, or a heading in the wrong place fails. The error names the section of the schema that the document breaks.

In this guide, you build a schema for recipes. Each entry under `sections` describes one heading and its body, which runs to the next heading. [Describe blocks and their units][sokf:how-to-describe-blocks] shows how to describe a body.

## Before you begin

- A schema with frontmatter and a list, as in [Your first schema][sokf:tutorial-first-schema].
- The `tramlines` command, from the [installation steps](/README.md#installation).

## Steps

1. Give each section a `heading` and a `level` from `1` to `6`. Write the heading in one of three forms:

   - A string, for a heading that never changes, such as `Ingredients`.
   - `{ from: title }`, for a heading that must equal a frontmatter value. Here the level-1 heading is the `title` of the recipe.
   - A heading rule, for a heading that varies: a `template` or a `pattern` with an `example` of a heading that it accepts. A string can't contain a hole, so a heading that varies needs a heading rule. [Write the shape of a line][sokf:how-to-write-line-shapes] explains templates and patterns.

   When a string and a template both match a heading, the string wins.

2. Set `optional: true` on a section that a document can leave out. Set `repeat: true` on a section that can occur more than once. By default, a section must occur exactly once. In this schema, a recipe contains one or more steps and can contain notes:

**`recipe.yaml`**

```yaml
line-limit: 60
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        example: 200 g flour
  - heading: { template: "Step ${:int}: ${}", example: "Step 1: Mix the batter" }
    level: 2
    repeat: true
    blocks:
      - kind: prose
        example: Whisk the flour, eggs and milk until smooth.
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
```

3. Check a document. The sections must appear in the order in which the schema lists them. To accept them in any order, set `sections-ordered: false` at the top of the schema.

**`pancakes.md`**

```markdown
---
title: Pancakes
---

# Pancakes

Thin pancakes for a weekend breakfast.

## Ingredients

- 200 g flour
- 2 eggs

## Step 1: Mix the batter

Whisk the flour, eggs and milk until smooth.

## Step 2: Cook

Fry each pancake for a minute on each side.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

4. Read the errors for headings. The error for a missing required section shows no line number. A heading that no section matches is an error. In this document the `Ingredients` section is missing, and the number in `Step one` is a word:

**`crepes.md`**

```markdown
---
title: Crepes
---

# Crepes

Thin pancakes for a weekend breakfast.

## Step one: Mix

Whisk the flour, eggs and milk until smooth.
```

```console
$ tramlines validate --schema recipe.yaml crepes.md
crepes.md: error[TLMD_18]: section `## Ingredients` is missing (schema recipe, section "Ingredients")
crepes.md: error[TLMD_18]: section `## Step ${:int}: ${}` is missing (schema recipe, section "Step ${:int}: ${}")
crepes.md:9: error[TLMD_19]: heading `## Step one: Mix` is not declared (schema recipe)
3 errors, 0 warnings
[exit 1]
```

5. To let writers add headings that the schema doesn't list, open a region after a section:

   - `subheadings-open: true` accepts undeclared headings deeper than the level of the section.
   - `headings-open: true` accepts undeclared headings at the level of the section or deeper.

   The region runs to the next heading that the schema declares. Only the rules that apply to the whole document, such as `line-limit`, check the bodies inside it.

6. To keep some headings out of every open region, list them under `sections-prohibited`. Here writers can add any subheading under `Notes`, except `Draft`:

**`recipe.yaml`**

```yaml
line-limit: 60
frontmatter:
  title:
    example: Pancakes
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: Thin pancakes for a weekend breakfast.
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        example: 200 g flour
  - heading: { template: "Step ${:int}: ${}", example: "Step 1: Mix the batter" }
    level: 2
    repeat: true
    blocks:
      - kind: prose
        example: Whisk the flour, eggs and milk until smooth.
  - heading: Notes
    level: 2
    optional: true
    subheadings-open: true
    blocks:
      - kind: prose
        example: The batter keeps for a day in the fridge.
sections-prohibited:
  - heading: Draft
```

**`pancakes.md`**

```markdown
---
title: Pancakes
---

# Pancakes

Thin pancakes for a weekend breakfast.

## Ingredients

- 200 g flour

## Step 1: Mix the batter

Whisk the flour, eggs and milk until smooth.

## Notes

The batter keeps for a day in the fridge.

### Serving

Serve with lemon and sugar.

### Draft

Try oat milk.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:25: error[TLMD_26]: heading `### Draft` is prohibited (schema recipe)
1 error, 0 warnings
[exit 1]
```

7. To nest a section inside each occurrence of a repeating one, give the outer section a `rule-id` and name that id as the `parent` of the inner section. The inner section must have a larger level and come after its parent, with only the other subsections of the parent between them. The `optional` and `repeat` of the inner section then apply within each occurrence of the parent.

   To require the inner heading to start with the number of the parent heading, a dot, and a further number, set `number-from-parent: true`. Under `2`, for example, the inner heading starts with `2.1`. The number of a heading is the text before its first space: digits separated by dots.

8. To describe text before the first heading, add a `preamble` with its own `blocks`. Without a `preamble`, any content before the first heading is an error.

9. When you rename a fixed heading, list its old text under `renamed-from`. A document that still contains the old heading fails with an error that names the new one. Change the heading in the document to the new text.

   This menu schema describes a date line before the heading, courses that repeat, and an optional allergen list in each course:

**`menu.yaml`**

```yaml
line-limit: 60
preamble:
  blocks:
    - kind: prose
      max: 1
      example: "Updated: 2026-09-30"
sections:
  - heading: Menu
    level: 1
    renamed-from: Dishes
    blocks: []
  - heading: { template: "${:int} ${}", example: "1 Starters" }
    level: 2
    repeat: true
    rule-id: course
    blocks:
      - kind: bullet-list
        example: Tomato soup
  - heading: { template: "${:int}.${:int} Allergens", example: "1.1 Allergens" }
    level: 3
    parent: course
    number-from-parent: true
    optional: true
    blocks:
      - kind: bullet-list
        example: The soup contains celery.
```

The menu still uses its old heading, and the `Allergens` heading of the second course uses the number of the first course:

**`menu.md`**

```markdown
Updated: 2026-09-30

# Dishes

## 1 Starters

- Tomato soup

### 1.1 Allergens

- The soup contains celery.

## 2 Mains

- Fish pie

### 1.1 Allergens

- The pie contains milk.
```

```console
$ tramlines validate --schema menu.yaml menu.md
menu.md:3: error[TLMD_137]: heading `Dishes` was renamed; it is now `Menu` (schema menu, section "Menu")
menu.md:17: error[TLMD_35]: heading `### 1.1 Allergens` must be numbered `2.<n>`, under `## 2 Mains` (schema menu, section "${:int}.${:int} Allergens")
2 errors, 0 warnings
[exit 1]
```

10. Fix the document: rename `Dishes` to `Menu`, and renumber the second `Allergens` heading `2.1`.

**`menu.md`**

```markdown
Updated: 2026-09-30

# Menu

## 1 Starters

- Tomato soup

### 1.1 Allergens

- The soup contains celery.

## 2 Mains

- Fish pie

### 2.1 Allergens

- The pie contains milk.
```

```console
$ tramlines validate --schema menu.yaml menu.md
0 errors, 0 warnings
```

## Troubleshooting

- `TLMD_18`: a required section is missing. Add the section, or set `optional` on it. See [TLMD_18][sokf:errors-sections#tlmd_18].
- `TLMD_151`: a section that doesn't repeat occurs twice. Remove one, or set `repeat` on it. See [TLMD_151][sokf:errors-sections#tlmd_151].
- `TLMD_19`: no section matches the heading. Correct the heading, declare it, or open a region before it. See [TLMD_19][sokf:errors-sections#tlmd_19].
- `TLMD_20`: a heading is more than one level deeper than the heading before it, such as a level-3 heading directly after a level-1 heading. See [TLMD_20][sokf:errors-sections#tlmd_20].
- `TLMD_22`: a section comes before a section that the schema lists ahead of it. Move it, or set `sections-ordered: false`. See [TLMD_22][sokf:errors-sections#tlmd_22].
- `TLMD_144`: a heading matches the text of a section at another level. See [TLMD_144][sokf:errors-sections#tlmd_144].
- `TLMD_38`: content comes before the first heading of the document, and the schema declares no `preamble`. See [TLMD_38][sokf:errors-sections#tlmd_38].
- `TLMD_31`: a nested section isn't inside its parent. See [Sections inside sections][sokf:errors-nested-sections].

## See also

- [Schema keys: `section`][sokf:reference-schema-keys#section]
- [Schema keys: `heading`][sokf:reference-schema-keys#heading]
- [Schema keys: `preamble`][sokf:reference-schema-keys#preamble]
- [Schema keys: `prohibited-heading`][sokf:reference-schema-keys#prohibited-heading]
- [Errors: headings and sections][sokf:errors-sections]
- [Strict by default][sokf:explanation-strict-by-default]
- [Capture keys and anchor them][sokf:how-to-capture-keys]

<!-- sokf:links -->

[sokf:errors-nested-sections]: /docs/reference/errors-nested-sections.md
[sokf:errors-sections]: /docs/reference/errors-sections.md
[sokf:errors-sections#tlmd_144]: /docs/reference/errors-sections.md#tlmd_144
[sokf:errors-sections#tlmd_151]: /docs/reference/errors-sections.md#tlmd_151
[sokf:errors-sections#tlmd_18]: /docs/reference/errors-sections.md#tlmd_18
[sokf:errors-sections#tlmd_19]: /docs/reference/errors-sections.md#tlmd_19
[sokf:errors-sections#tlmd_20]: /docs/reference/errors-sections.md#tlmd_20
[sokf:errors-sections#tlmd_22]: /docs/reference/errors-sections.md#tlmd_22
[sokf:errors-sections#tlmd_38]: /docs/reference/errors-sections.md#tlmd_38
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-schema-keys#heading]: /docs/reference/schema-keys.md#heading
[sokf:reference-schema-keys#preamble]: /docs/reference/schema-keys.md#preamble
[sokf:reference-schema-keys#prohibited-heading]: /docs/reference/schema-keys.md#prohibited-heading
[sokf:reference-schema-keys#section]: /docs/reference/schema-keys.md#section
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
