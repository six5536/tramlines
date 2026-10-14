---
type: HowTo
id: how-to-read-a-finding
title: Read an error and fix the document
description: Read each part of an error from tramlines, find what its key means and which rule of the schema it breaks, and change the document so that it passes.
order: 110
---

# Read an error and fix the document

When `tramlines validate` reports errors in a document that you wrote, read each error from the left: its location, which rule of the specification the document breaks, and which rule of the schema the document fails. At the end, the document passes without any change to the schema.

## Before you begin

- The `tramlines` command, from the [installation steps](/README.md#installation).
- The schema that your documents follow, and the command that checks them. The [command reference][sokf:reference-cli] lists the options.

## Steps

1. Check the document. This schema describes a recipe: a title, a list of ingredients, and a method.

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
        each:
          template: "${:int} ${unit:word} ${}"
          example: 200 g plain flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        example: Whisk the flour, eggs and milk into a smooth batter.
```

**`crepes.md`**

```markdown
---
title: Crepes
---

# Crepes

Thin pancakes from Brittany.

## Ingredients

- 250 g flour
- a pinch of salt
```

```console
$ tramlines validate --schema recipe.yaml crepes.md
crepes.md: error[TLMD_18]: section `## Method` is missing (schema recipe, section "Method")
crepes.md:12: error[TL_74]: `a pinch of salt` fails at column 1: expected an `int` (`\d+`), found `a pinch of salt`; an item of this kind reads 200 g plain flour (schema recipe, section "Ingredients" block 1)
2 errors, 0 warnings
[exit 1]
```

2. Read the parts of each error. An error has the form `path:line: severity[KEY]: message (schema id, place)`:

   - `crepes.md:12` is the file and the line number in it. An error with no line number, such as a missing section, applies to the whole document.
   - `error` is the severity. An `error` fails the check, and the command exits with `1`. A `warning` doesn't fail the check.
   - `TL_74` is the key of the rule that the document breaks.
   - The message describes the fault. A message about the shape of a line names the column where the text stops matching, what the rule expects there, and an example of a correct line: here, `200 g plain flour`.
   - `schema recipe` is the id of the schema, the name of its file without the extension.
   - The place names the rule of the schema: `section "Ingredients" block 1` is the first block under the `Ingredients` heading. A place can also name a rule by its `rule-id`, as in ``rule `reqs` ``. The place `section "<title>"` names a section whose heading the schema takes from the frontmatter.

3. Find the key on the error reference. A key that starts with `TL_` comes from the core specification, and one that starts with `TLMD_` from the markdown specification. The [Errors][sokf:reference-errors] page lists the page for each group of keys. Each entry lists the message, its cause, the fix, and an example. `TL_74` is on [Errors: lines and length][sokf:errors-lines#tl_74], and `TLMD_18` is on [Errors: headings and sections][sokf:errors-sections#tlmd_18].

4. Find the rule in the schema. In `recipe.yaml`, the section with `heading: Ingredients` contains one block, a `bullet-list`. Its template `${:int} ${unit:word} ${}` matches a number, a word, and any text. The section with `heading: Method` doesn't set `optional: true`, so every recipe needs it.

5. Fix the document, not the schema. Start the salt with a number and a word, as the example `200 g plain flour` does, and add the method:

**`crepes.md`**

```markdown
---
title: Crepes
---

# Crepes

Thin pancakes from Brittany.

## Ingredients

- 250 g flour
- 1 pinch salt

## Method

1. Whisk the flour, eggs and milk into a smooth batter.
2. Fry each crepe for a minute on each side.
```

```console
$ tramlines validate --schema recipe.yaml crepes.md
0 errors, 0 warnings
```

6. If the document is right and the rule is wrong for it, ask the author of the schema to change the rule. An error in the schema file itself names `recipe.yaml` and no place. Such an error is for the author of the schema too. The command reports it before it checks a document.

## Troubleshooting

- Every error names the schema file: the schema is wrong, and no document can pass until its author fixes it. See [Errors: the schema][sokf:errors-schema].
- The command stops with exit code `2` and one line of output: the command line is wrong, or the command can't read a file. See [The command][sokf:reference-cli].
- A message mentions a repair: run the command again with `--fix`, and it makes the repair in the file. See [Repair a document and start a new one][sokf:how-to-repair-and-start-documents].

## See also

- [Errors][sokf:reference-errors]
- [The command][sokf:reference-cli]
- [Strict by default][sokf:explanation-strict-by-default]

<!-- sokf:links -->

[sokf:errors-lines#tl_74]: /docs/reference/errors-lines.md#tl_74
[sokf:errors-schema]: /docs/reference/errors-schema.md
[sokf:errors-sections#tlmd_18]: /docs/reference/errors-sections.md#tlmd_18
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-repair-and-start-documents]: /docs/how-to/repair-and-start-documents.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-errors]: /docs/reference/errors.md
