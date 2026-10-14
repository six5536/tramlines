---
type: Reference
id: reference-types
title: Built-in types
description: The types that a hole of a template can have, how a schema or a formats file defines its own, and the order in which a name is looked up.
order: 40
---

# Built-in types

A type defines what a template hole matches: `${amount:int}` is a hole of the type `int`. tramlines defines five types. A schema or a formats file can define more types under names that aren't reserved. tramlines looks up each type by name in a fixed order.

[Write the shape of a line][sokf:how-to-write-line-shapes] shows how to write a template with holes.

## Types that tramlines defines

| Type | Regular expression | Matches |
| --- | --- | --- |
| `text` | `.+` | Any text of one or more characters. A hole with no type, such as `${item}` or `${}`, has this type. |
| `word` | `\w+` | One or more letters, digits, or underscores, including letters outside ASCII, such as `é`. |
| `int` | `\d+` | One or more digits. No sign and no decimal point. |
| `slug` | `[a-z0-9]+(?:-[a-z0-9]+)*` | Lowercase ASCII letters and digits, in words joined by single hyphens, such as `plain-flour`. |
| `seq` | `\d+` | The sequence number of an `id` template. In the `id` template of the frontmatter, it matches exactly `seq-width` digits, or three digits when the schema doesn't set `seq-width`. |

The regular expressions use the Rust dialect. A hole matches only its type, because tramlines matches the template against the whole text of the line.

An error names the type and its regular expression, so you can see what the hole accepts:

```text
expected a `slug` (`[a-z0-9]+(?:-[a-z0-9]+)*`), found `Bad-Slug 3`
```

## Holes

| Hole | Type | Captured as |
| --- | --- | --- |
| `${name:type}` | `type` | `name` |
| `${name}` | `text` | `name` |
| `${:type}` | `type` | Nothing |
| `${}` | `text` | Nothing |

A rule's `key` can name a captured hole. `$${` is a literal `${`, and any other `$` is a literal `$`.

## Define a type

A schema defines its own types in a `types` map, at the top level of the schema. Each entry maps a name to one of two kinds of value:

- A regular expression, as a string. It can't contain an anchor (`^` or `$`) or a capture group: the template already matches the whole line and names its own groups. Write a group as `(?:…)`.
- A list of choices, each a non-empty string. A choice matches exactly as written: tramlines escapes every character, so `b.c` matches only `b.c`.

A type name must be a slug and can't be a reserved name.

Write the alternatives and optional parts of a hole in a type, so that a template never needs regular expression syntax.

The following schema defines `unit` as a list of choices and `amount` as a regular expression, then uses both in a template:

**`recipe.yaml`**

```yaml
line-limit: 100
types:
  unit: [g, ml, tsp, tbsp]
  amount: '\d+(?:\.\d+)?'
sections:
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${:amount} ${:unit} ${food}'
          example: 1.5 tsp baking powder
```

**`scones.md`**

```markdown
## Ingredients

- 250 g flour
- 1.5 tsp baking powder
- 2 cups milk
```

```console
$ tramlines validate --schema recipe.yaml scones.md
scones.md:5: error[TL_74]: `2 cups milk` fails at column 3: expected a `unit` (one of `g`, `ml`, `tsp`, `tbsp`), found `cups milk`; an item of this kind reads 1.5 tsp baking powder (schema recipe, section "Ingredients" block 1)
1 error, 0 warnings
[exit 1]
```

An error shows a list of choices as `one of`, followed by the choices.

## Types in a formats file

A formats file, which you pass with `tramlines validate --formats <FILE>`, contains a `registry` map of line formats and an optional `types` map. Its `types` map takes the same form as the `types` of a schema. Every schema checked with that file can use its types. [Share line formats across schemas][sokf:how-to-share-line-formats] shows how to write one.

## Reserved names

tramlines reserves the names `text`, `word`, `int`, `slug`, and `seq`: a schema or a formats file can't define them. A program that embeds tramlines can reserve types of its own. The `tramlines` command reserves only these five.

The check of a schema fails if the schema defines a reserved name, a type with an anchor or a capture group, a name that isn't a slug, or an empty list of choices:

**`broken.yaml`**

```yaml
line-limit: 100
types:
  int: '[0-9]+'
  amount: '^\d+$'
  unit: '(g|ml)'
  Unit: [g]
  size: []
sections:
  - heading: Ingredients
    level: 2
    blocks: []
```

```console
$ tramlines validate --schema broken.yaml scones.md
broken.yaml:3: error[TL_29]: type `int` is built in and reserved; name the type otherwise
broken.yaml:4: error[TL_109]: type `amount`: `^\d+$` holds an anchor; a template matches the whole line itself
broken.yaml:5: error[TL_109]: type `unit`: `(g|ml)` holds a capture group; write `(?:…)`, since a template names its own groups
broken.yaml:6: error[TL_102]: type `Unit` must be named by a slug
broken.yaml:7: error[TL_101]: type `size`: a list of choices holds one or more non-empty strings
5 errors, 0 warnings
[exit 1]
```

If a formats file defines a reserved type, the command stops with exit code 2. It prints the fault under [TL_29][sokf:errors-schema-types-keys-lifecycle#tl_29], the key that the fault has in a schema, as [TL_124][sokf:tramlines:tramlines-spec#tl_124] states.

## Lookup order

tramlines looks up a type by its name in the following layers, the nearest first, and takes the first entry it finds:

1. The `types` of the schema.
2. The `types` of the formats file, when `--formats` names one.
3. The built-in types.

An entry in a nearer layer replaces an entry of the same name in a farther one. tramlines builds each format of a schema with the types that the lookup finds for that schema. A schema can therefore redefine a type in a shared format.

In the following example, the formats file defines the format `ingredient` and the types `unit` and `amount`. The schema adds `tsp` to `unit`, and uses `amount` from the formats file:

**`formats.yaml`**

```yaml
registry:
  ingredient:
    template: '${:amount} ${:unit} ${food}'
    example: 250 g flour
types:
  unit: [g, ml]
  amount: '\d+'
```

**`baking.yaml`**

```yaml
line-limit: 100
types:
  unit: [g, ml, tsp]
sections:
  - heading: Ingredients
    level: 2
    blocks:
      - kind: bullet-list
        each: { format: ingredient }
```

```console
$ tramlines validate --schema baking.yaml --formats formats.yaml scones.md
scones.md:4: error[TL_74]: `1.5 tsp baking powder` fails at column 1: expected an `amount` (`\d+`), found `1.5 tsp baking powder`; an item of this kind reads 250 g flour (schema baking, section "Ingredients" block 1)
scones.md:5: error[TL_74]: `2 cups milk` fails at column 3: expected a `unit` (one of `g`, `ml`, `tsp`), found `cups milk`; an item of this kind reads 250 g flour (schema baking, section "Ingredients" block 1)
2 errors, 0 warnings
[exit 1]
```

- The format uses the `unit` of the schema, so it accepts `tsp`, and the error for `cups` lists three choices.
- The format uses the `amount` of the formats file, `\d+`, which allows no decimal point, so `1.5` fails.

The specification defines the lookup in [TL_27][sokf:tramlines:tramlines-spec#tl_27], the reserved names in [TL_28][sokf:tramlines:tramlines-spec#tl_28], and the checks on a type in [TL_29][sokf:tramlines:tramlines-spec#tl_29], [TL_101][sokf:tramlines:tramlines-spec#tl_101], [TL_102][sokf:tramlines:tramlines-spec#tl_102], [TL_109][sokf:tramlines:tramlines-spec#tl_109], and [TL_118][sokf:tramlines:tramlines-spec#tl_118].

<!-- sokf:links -->

[sokf:errors-schema-types-keys-lifecycle#tl_29]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_29
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:tramlines:tramlines-spec#tl_101]: /knowledge/spec/tramlines-spec.md#tl_101
[sokf:tramlines:tramlines-spec#tl_102]: /knowledge/spec/tramlines-spec.md#tl_102
[sokf:tramlines:tramlines-spec#tl_109]: /knowledge/spec/tramlines-spec.md#tl_109
[sokf:tramlines:tramlines-spec#tl_118]: /knowledge/spec/tramlines-spec.md#tl_118
[sokf:tramlines:tramlines-spec#tl_124]: /knowledge/spec/tramlines-spec.md#tl_124
[sokf:tramlines:tramlines-spec#tl_27]: /knowledge/spec/tramlines-spec.md#tl_27
[sokf:tramlines:tramlines-spec#tl_28]: /knowledge/spec/tramlines-spec.md#tl_28
[sokf:tramlines:tramlines-spec#tl_29]: /knowledge/spec/tramlines-spec.md#tl_29
