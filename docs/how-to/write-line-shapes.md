---
type: HowTo
id: how-to-write-line-shapes
title: Write the shape of a line
description: Describe the text of a heading, a paragraph, a list item, or a frontmatter value with exact text, a template with typed holes, or a regular expression.
order: 10
---

# Write the shape of a line

When a rule must specify the text of one line of a document, give it a shape: `text` for a line that never varies, `template` for a line with parts that vary, or `pattern` for what a template can't express. At the end, each line has the shape that you meant, and the error for a wrong line shows the example of a correct one.

A shape fits any rule that checks one unit: a heading, a paragraph, a list item, a table row, a line of a fenced block, or a frontmatter value. This guide uses list items and a paragraph of a recipe.

## Before you begin

- A schema with a section and a block, as in [Your first schema][sokf:tutorial-first-schema].
- The `tramlines` command, from the [installation steps](/README.md#installation).

## Steps

1. Pick one shape for each rule. A rule declares exactly one shape: `text`, `template`, `pattern`, or a bound `format`. [Share line formats across schemas][sokf:how-to-share-line-formats] describes formats. Two shapes on one rule are an error.

2. For a line that never varies, write it as `text`. In this recipe schema, the paragraph of the first block must be exactly `Makes eight pancakes.`

3. For a line with parts that vary, write a `template`: the line itself, with a hole at each part that varies. A template matches the whole line. In this recipe, every ingredient is a quantity, a unit, and a food. Give every template an `example`, a real line that it matches. A hole has a name, a type, both, or neither:

   - `${name:type}` matches text of the type and captures it as `name`.
   - `${name}` matches any text, as the type `text`, and captures it as `name`.
   - `${:type}` matches text of the type and captures nothing.
   - `${}` matches any text and captures nothing.

   A hole name contains only letters, digits, and `_`, and appears only once in its template. A template has at least one hole: write a line with none as `text`. `$${` is a literal `${`, and any other `$` is a literal `$`. The `key` of a rule names a captured hole. [Capture keys and anchor them][sokf:how-to-capture-keys] shows how.

4. For what a template can't express, write a `pattern`, a regular expression in the Rust dialect. The command doesn't anchor a pattern, so write `^` and `$` to match the whole line. In this recipe, each step of the method starts with a capital letter and ends with its only full stop. Give every pattern an `example` too.

5. When a hole accepts one of a few words, or text that no built-in type matches, define your own type under `types`. A type is a list of words, matched as written, or a regular expression with no anchor and no capture group. The type `unit` in the following schema is a list. Put alternatives and optional parts in a type, so that a template never needs its own regular expression syntax.

   The built-in types are `text`, `word`, `int`, `slug`, and `seq`. [Built-in types][sokf:reference-types] lists what each one matches. Your own type can't have the name of a built-in type. The types of a formats file rank below the types of the schema, so a type in the schema replaces a type of the same name in the file. [Share line formats across schemas][sokf:how-to-share-line-formats] describes formats files.

**`recipe.yaml`**

```yaml
line-limit: 100
types:
  unit: [g, ml, tsp, tbsp, whole]
sections:
  - heading: Pancakes
    level: 1
    blocks:
      - kind: prose
        each: { text: Makes eight pancakes. }
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${qty:int} ${unit:unit} ${food}'
          example: 200 g plain flour
  - heading: Method
    level: 2
    blocks:
      - kind: numbered-list
        each:
          pattern: '^[A-Z][^.]*\.$'
          example: Whisk the flour, eggs and milk into a smooth batter.
```

6. Check a document. When the command matches a shape, it reads each run of spaces and line breaks in the text as one space. A paragraph wrapped over two lines therefore matches as one line, and a double space matches a single one:

**`pancakes.md`**

```markdown
# Pancakes

Makes eight
pancakes.

## Ingredients

- 200 g plain flour
- 2 whole eggs
- 300  ml milk

## Method

1. Whisk the flour, eggs and milk into a smooth batter.
2. Fry each pancake for a minute on each side.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
0 errors, 0 warnings
```

7. Read a failure. The error for a template names the column where the text stops matching, what the template requires there, and the example of the rule. The error for a pattern names the pattern and the example. In this document, `eggs` isn't a unit, and the second step starts with a lowercase letter:

**`crepes.md`**

```markdown
# Pancakes

Makes eight pancakes.

## Ingredients

- 200 g plain flour
- 2 eggs

## Method

1. Whisk the flour, eggs and milk into a smooth batter.
2. fry each pancake for a minute on each side.
```

```console
$ tramlines validate --schema recipe.yaml crepes.md
crepes.md:8: error[TL_74]: `2 eggs` fails at column 3: expected a `unit` (one of `g`, `ml`, `tsp`, `tbsp`, `whole`), found `eggs`; an item of this kind reads 200 g plain flour (schema recipe, section "Ingredients" block 1)
crepes.md:13: error[TL_74]: `fry each pancake for a minute on each side.` does not match `^[A-Z][^.]*\.$`; an item of this kind reads Whisk the flour, eggs and milk into a smooth batter. (schema recipe, section "Method" block 1)
2 errors, 0 warnings
[exit 1]
```

## Troubleshooting

- `TL_22`: a rule declares two shapes. Keep one of `text`, `template`, `pattern`, and `format`. See [TL_22][sokf:errors-schema-shapes#tl_22].
- `TL_25`: a template has no hole. Write it as `text`. See [TL_25][sokf:errors-schema-shapes#tl_25].
- `TL_119`: a template names one hole twice. Rename one of them. See [TL_119][sokf:errors-schema-shapes#tl_119].
- `TL_120`: a hole names a type that is unknown to the schema. Define the type, or correct its name. See [TL_120][sokf:errors-schema-shapes#tl_120].
- `TL_29`: a type in the schema uses the name of a built-in type. See [TL_29][sokf:errors-schema-types-keys-lifecycle#tl_29].
- `TL_109`: the regular expression of a type contains an anchor or a capture group. See [TL_109][sokf:errors-schema-types-keys-lifecycle#tl_109].
- `TL_66`: a rule with a `template` or a `pattern` lacks an `example`. Add one. See [TL_66][sokf:errors-examples#tl_66].
- `TL_74`: a line fails its shape. Compare it with the example in the message, from the column that the message names. See [TL_74][sokf:errors-lines#tl_74].

## See also

- [Schema keys: `unit`][sokf:reference-schema-keys#unit]
- [Schema keys: `heading`][sokf:reference-schema-keys#heading]
- [Built-in types][sokf:reference-types]
- [Share line formats across schemas][sokf:how-to-share-line-formats]
- [Write examples and guidance][sokf:how-to-write-examples-and-guidance]
- [Section 4 of the core specification][sokf:tramlines:tramlines-spec#4-shapes-and-types]

<!-- sokf:links -->

[sokf:errors-examples#tl_66]: /docs/reference/errors-examples.md#tl_66
[sokf:errors-lines#tl_74]: /docs/reference/errors-lines.md#tl_74
[sokf:errors-schema-shapes#tl_119]: /docs/reference/errors-schema-shapes.md#tl_119
[sokf:errors-schema-shapes#tl_120]: /docs/reference/errors-schema-shapes.md#tl_120
[sokf:errors-schema-shapes#tl_22]: /docs/reference/errors-schema-shapes.md#tl_22
[sokf:errors-schema-shapes#tl_25]: /docs/reference/errors-schema-shapes.md#tl_25
[sokf:errors-schema-types-keys-lifecycle#tl_109]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_109
[sokf:errors-schema-types-keys-lifecycle#tl_29]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_29
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:reference-schema-keys#heading]: /docs/reference/schema-keys.md#heading
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
[sokf:reference-types]: /docs/reference/types.md
[sokf:tramlines:tramlines-spec#4-shapes-and-types]: /knowledge/spec/tramlines-spec.md#4-shapes-and-types
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
