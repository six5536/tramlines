---
type: Explanation
id: explanation-strict-by-default
title: Strict by default
description: Why tramlines requires every part that a schema declares and refuses every part that it doesn't, and how each relaxation loosens one rule at a time.
order: 20
---

# Strict by default

A schema is strict. What it declares is required, what it doesn't declare is refused, and a section contains exactly the blocks that the schema declares for it, in order. So tramlines reports a heading that the schema doesn't mention. You loosen each rule with a key that you set on purpose.

## Why strict

A schema exists to keep documents of one kind alike, so that a reader, a script, or an agent can rely on their shape. A lenient checker passes a document with an extra section, a misspelled frontmatter key, or a missing part. The documents then drift apart unnoticed until someone depends on their shape. A strict checker reports each fault at the line where it occurs.

Strictness also keeps a schema complete. Every part that a document can contain is written in the schema. The schema is therefore the full description of the kind, and the scaffold derived from it contains every part that the schema declares.

## The three rules

Three rules apply unless the schema relaxes them:

- **What is declared is required.** A declared section, frontmatter key, or block must be present exactly once.
- **What is not declared is refused.** A heading, frontmatter key, table column, or block that the schema doesn't declare is an error.
- **A declaration is exclusive.** A section's body contains exactly its declared blocks, in the declared order, and the sections come in the declared order.

This schema describes a shopping list with a title and one list:

**`list.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Groceries
sections:
  - heading: { from: title }
    level: 1
    blocks: []
  - heading: Items
    level: 2
    blocks:
      - kind: bullet-list
        example: Milk
```

This document adds a frontmatter key and a section that the schema doesn't declare:

**`list.md`**

```markdown
---
title: Groceries
store: Corner shop
---

# Groceries

## Items

- Milk
- Bread

## Notes

Pay by card.
```

```console
$ tramlines validate --schema list.yaml list.md
list.md:3: error[TL_55]: frontmatter key `store` is not declared (schema list)
list.md:13: error[TLMD_19]: heading `## Notes` is not declared (schema list)
2 errors, 0 warnings
[exit 1]
```

The command reports each undeclared part as one error, at the line of that part.

## The relaxations

Each relaxation is a key, and each loosens one rule for one part of the schema. Nothing loosens by default, so a reader of the schema sees every exception where it applies.

| Key | Loosens | Where |
| --- | ------- | ----- |
| `optional: true` | A declared part can be absent. | A section, a block, a frontmatter key, a position in a `sequence`. |
| `repeat: true` | A declared part can occur more than once. | A section, a block, a position in a `sequence`. |
| `frontmatter-open: true` | Undeclared frontmatter keys are accepted. | The schema. |
| `headings-open: true` | Undeclared headings of the section's level or deeper can follow it. | A section. |
| `subheadings-open: true` | Undeclared headings deeper than the section can follow it. | A section. |
| `columns-open: true` | Further columns after the declared ones are accepted. | A `table` block. |
| `units-open: true` | A unit that no rule under `rules` takes is accepted. | A block. |
| `lines-open: true` | A line that no rule under `lines` takes is accepted. | A unit rule with `lines`. |
| `sections-ordered: false` | Sections can come in any order. | The schema. |
| `blocks-ordered: false` | Blocks can come in any order. | A section. |

The core grammar defines `frontmatter-open`, and the Markdown grammar defines the rest. A host extension can add `-open` keys of its own. [The schema-keys reference][sokf:reference-schema-keys] describes each key in full.

The table leaves out `min` and `max`, because they are bounds. They set how many units a block contains, and they default to one and no limit.

### Relaxing the example

To accept the extra key and section, the schema relaxes the two rules that the document broke, and nothing else. `frontmatter-open` accepts any frontmatter key. A `Notes` section with `optional: true` accepts the extra section without requiring it in every list:

**`list.yaml`**

```yaml
line-limit: 0
frontmatter-open: true
frontmatter:
  title:
    example: Groceries
sections:
  - heading: { from: title }
    level: 1
    blocks: []
  - heading: Items
    level: 2
    blocks:
      - kind: bullet-list
        example: Milk
  - heading: Notes
    level: 2
    optional: true
    blocks:
      - kind: prose
        example: Pay by card.
```

```console
$ tramlines validate --schema list.yaml list.md
0 errors, 0 warnings
```

A declaration of `store` under `frontmatter`, with `optional: true`, is the narrower choice. It accepts that one key and still refuses a misspelling of it.

### An open region

An open section accepts undeclared headings after it, up to the next declared heading. tramlines checks the bodies under those headings only against the rules that apply to the whole document, such as `prohibited-patterns` and `line-limit`. Use an open region for a part whose structure the schema can't predict, such as free notes, and keep it as small as you can. To ban particular headings inside an open region, list them under `sections-prohibited`.

## Absence is strict too

A key that you leave out never relaxes a rule. A section with no `blocks` must contain exactly one prose block. A block with no `optional` is required. A table with no `columns-open` contains exactly its declared columns. [Pitfalls in writing a schema][sokf:explanation-pitfalls] shows the errors that follow.

## See also

- [TL_54][sokf:tramlines:tramlines-spec#tl_54] and [TL_55][sokf:tramlines:tramlines-spec#tl_55]: a declared frontmatter key is required, and an undeclared one is refused unless `frontmatter-open` is set.
- [TLMD_18][sokf:tramlines:tramlines-md-spec#tlmd_18], [TLMD_151][sokf:tramlines:tramlines-md-spec#tlmd_151], and [TLMD_19][sokf:tramlines:tramlines-md-spec#tlmd_19]: a declared section is required once, and an undeclared heading is refused.
- [TLMD_22][sokf:tramlines:tramlines-md-spec#tlmd_22]: sections come in the declared order.
- [TLMD_24][sokf:tramlines:tramlines-md-spec#tlmd_24] and [TLMD_25][sokf:tramlines:tramlines-md-spec#tlmd_25]: what an open region accepts.
- [TLMD_39][sokf:tramlines:tramlines-md-spec#tlmd_39], [TLMD_152][sokf:tramlines:tramlines-md-spec#tlmd_152], and [TLMD_40][sokf:tramlines:tramlines-md-spec#tlmd_40]: a body has exactly its declared blocks, and no `blocks` means one prose block.
- [TLMD_44][sokf:tramlines:tramlines-md-spec#tlmd_44]: a table has its declared columns.
- [Describe sections and headings][sokf:how-to-describe-sections]
- [Constrain frontmatter and ids][sokf:how-to-constrain-frontmatter]

<!-- sokf:links -->

[sokf:explanation-pitfalls]: /docs/explanation/pitfalls.md
[sokf:how-to-constrain-frontmatter]: /docs/how-to/constrain-frontmatter.md
[sokf:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:reference-schema-keys]: /docs/reference/schema-keys.md
[sokf:tramlines:tramlines-md-spec#tlmd_151]: /knowledge/spec/tramlines-md-spec.md#tlmd_151
[sokf:tramlines:tramlines-md-spec#tlmd_152]: /knowledge/spec/tramlines-md-spec.md#tlmd_152
[sokf:tramlines:tramlines-md-spec#tlmd_18]: /knowledge/spec/tramlines-md-spec.md#tlmd_18
[sokf:tramlines:tramlines-md-spec#tlmd_19]: /knowledge/spec/tramlines-md-spec.md#tlmd_19
[sokf:tramlines:tramlines-md-spec#tlmd_22]: /knowledge/spec/tramlines-md-spec.md#tlmd_22
[sokf:tramlines:tramlines-md-spec#tlmd_24]: /knowledge/spec/tramlines-md-spec.md#tlmd_24
[sokf:tramlines:tramlines-md-spec#tlmd_25]: /knowledge/spec/tramlines-md-spec.md#tlmd_25
[sokf:tramlines:tramlines-md-spec#tlmd_39]: /knowledge/spec/tramlines-md-spec.md#tlmd_39
[sokf:tramlines:tramlines-md-spec#tlmd_40]: /knowledge/spec/tramlines-md-spec.md#tlmd_40
[sokf:tramlines:tramlines-md-spec#tlmd_44]: /knowledge/spec/tramlines-md-spec.md#tlmd_44
[sokf:tramlines:tramlines-spec#tl_54]: /knowledge/spec/tramlines-spec.md#tl_54
[sokf:tramlines:tramlines-spec#tl_55]: /knowledge/spec/tramlines-spec.md#tl_55
