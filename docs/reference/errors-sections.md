---
type: ErrorReference
id: errors-sections
title: 'Errors: headings and sections'
description: The errors that tramlines reports for the headings of a document and the sections they open, and how to fix each one.
order: 40
---

# Errors: headings and sections

These errors concern the headings of a document and the sections that the schema declares for those headings. Section 4 of the Markdown specification defines these errors. [Errors: sections inside sections][sokf:errors-nested-sections] lists the errors for a section inside a parent section. [Errors: keys and repairs][sokf:errors-keys-and-repairs] lists the headings that a repair rewrites.

## TLMD_18

A section that the schema requires is missing from the document, or from an occurrence of its parent ([TLMD_18][sokf:tramlines:tramlines-md-spec#tlmd_18]).

### Message

```text
<file>: error[TLMD_18]: section `<heading>` is missing (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_18]: section `<heading>` is missing under `<parent heading>` (schema <id>, <place of the rule>)
```

### Cause

The schema declares the section without `optional: true`, and the document contains no heading for it. For a section that declares `parent`, `optional` applies within each occurrence of the parent: the second message names the parent heading that lacks the section.

### Fix

Add the missing heading to the document. If the message names a parent heading, put the missing heading under it. If documents of this kind can leave the section out, set `optional: true` on the section in the schema.

### Example

This schema requires a `Steps` section:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 2
    blocks: []
```

This document lacks a `Steps` heading:

**`recipe.md`**

```markdown
# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md: error[TLMD_18]: section `## Steps` is missing (schema recipe, section "Steps")
1 error, 0 warnings
[exit 1]
```

Add the heading:

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_151

The heading of a section that doesn't repeat occurs more than once in the document, or under one occurrence of its parent ([TLMD_151][sokf:tramlines:tramlines-md-spec#tlmd_151]).

### Message

```text
<file>:<line>: error[TLMD_151]: heading `<heading>` occurs more than once (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_151]: heading `<heading>` occurs more than once under `<parent heading>` (schema <id>, <place of the rule>)
```

### Cause

The schema declares the section without `repeat: true`, and the document contains its heading more than once. For a section that declares `parent`, `repeat` applies within each occurrence of the parent: the second message names the parent heading that contains the section more than once. A repeated heading stands after the sections that follow its first occurrence. The command reports it with this error, not as out of place ([TLMD_22](#tlmd_22)). The body of a repeated heading can capture a key that the first body captured. The command reports that key with this error too, not as captured twice ([TLMD_29](#tlmd_29)).

### Fix

Remove the second heading, or merge the two sections into one. If the section can occur more than once, set `repeat: true` on the section in the schema.

### Example

In this schema, each step has one `Check`:

**`guide.yaml`**

```yaml
line-limit: 0
sections-ordered: false
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 2
    parent: step
    blocks: []
```

This document contains two `Check` headings under `Step 1`:

**`guide.md`**

```markdown
# Step 1

## Check

## Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.md:5: error[TLMD_151]: heading `## Check` occurs more than once under `# Step 1` (schema guide, section "Check")
1 error, 0 warnings
[exit 1]
```

Remove the second heading:

**`guide.md`**

```markdown
# Step 1

## Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

## TLMD_19

A heading matches no section of the schema, and no open region allows it ([TLMD_19][sokf:tramlines:tramlines-md-spec#tlmd_19]).

### Message

```text
<file>:<line>: error[TLMD_19]: heading `<heading>` is not declared (schema <id>)
```

### Cause

The document contains a heading that no section of the schema matches, outside the open region of any section that sets `headings-open` or `subheadings-open`.

### Fix

Remove the heading from the document, or change it to a heading that the schema declares. If the heading belongs in documents of this kind, declare a section for it in the schema, or open a region for it with `headings-open` or `subheadings-open` on the section before it.

### Example

This schema declares `Recipe` and `Steps`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 2
    blocks: []
```

This document adds a `Tips` heading:

**`recipe.md`**

```markdown
# Recipe

## Steps

## Tips
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:5: error[TLMD_19]: heading `## Tips` is not declared (schema recipe)
1 error, 0 warnings
[exit 1]
```

Remove the heading:

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_20

A heading is more than one level deeper than the heading before it ([TLMD_20][sokf:tramlines:tramlines-md-spec#tlmd_20]).

### Message

```text
<file>:<line>: error[TLMD_20]: heading `<heading>` is more than one level deeper than `<heading before>` (schema <id>)
```

### Cause

The document skips a level, such as a level 3 heading directly after a level 1 heading. tramlines reports this error for every schema, whatever its sections declare. For a heading that matches a section at a level other than the declared one, the command reports [TLMD_144](#tlmd_144) instead. For a subsection outside every occurrence of its parent, it reports [TLMD_31][sokf:errors-nested-sections#tlmd_31].

### Fix

Change the level of the heading in the document, so that it's at most one level deeper than the heading before it.

### Example

This schema lets any deeper heading follow `Recipe`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    subheadings-open: true
    blocks: []
```

This document puts a level 3 heading after the level 1 heading:

**`recipe.md`**

```markdown
# Recipe

### Tips
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:3: error[TLMD_20]: heading `### Tips` is more than one level deeper than `# Recipe` (schema recipe)
1 error, 0 warnings
[exit 1]
```

Make it a level 2 heading:

**`recipe.md`**

```markdown
# Recipe

## Tips
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_21

In a schema that orders its sections, a section is more than one level deeper than the section before it ([TLMD_21][sokf:tramlines:tramlines-md-spec#tlmd_21]).

### Message

```text
<file>:<line>: error[TLMD_21]: section `<section>` skips a level after `<section before>`
```

### Cause

The schema lists a section more than one level deeper than the section before it, and doesn't set `sections-ordered: false`. A document that follows the schema would skip a level too.

### Fix

Change the `level` of the section in the schema, so that it's at most one level deeper than the section before it.

### Example

In this schema, `Steps` is level 3 after the level 1 `Recipe`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 3
    blocks: []
```

**`recipe.md`**

```markdown
# Recipe

### Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.yaml:6: error[TLMD_21]: section `Steps` skips a level after `Recipe`
1 error, 0 warnings
[exit 1]
```

Make `Steps` level 2 in the schema and in the document:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 2
    blocks: []
```

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_22

A section appears before another section that the schema lists earlier ([TLMD_22][sokf:tramlines:tramlines-md-spec#tlmd_22]).

### Message

```text
<file>:<line>: error[TLMD_22]: heading `<heading>` is out of place; the sections are in the order the schema lists them (schema <id>, <place of the rule>)
```

### Cause

The schema doesn't set `sections-ordered: false`, so its sections must appear in the order that it lists them. The document places them in another order. The error names only the first heading that is out of place. For a heading that occurs again where its section doesn't declare `repeat`, the command reports [TLMD_151](#tlmd_151) instead.

### Fix

Move the sections of the document into the order in which the schema lists them. If the sections of documents of this kind can appear in any order, set `sections-ordered: false` in the schema.

### Example

This schema lists `Recipe`, then `Steps`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 2
    blocks: []
```

This document puts `Steps` first:

**`recipe.md`**

```markdown
## Steps

# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:3: error[TLMD_22]: heading `# Recipe` is out of place; the sections are in the order the schema lists them (schema recipe, section "Recipe")
1 error, 0 warnings
[exit 1]
```

Put the sections in order:

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_144

A heading matches a section, but its level differs from the level that the section declares ([TLMD_144][sokf:tramlines:tramlines-md-spec#tlmd_144]).

### Message

```text
<file>:<line>: error[TLMD_144]: heading `<heading>` must be level <level> (schema <id>, <place of the rule>)
```

### Cause

The text of the heading matches a section of the schema, and the number of `#` signs differs from the `level` of that section.

### Fix

Change the level of the heading in the document to the level that the message names. If the other level is correct for documents of this kind, change the `level` of the section in the schema.

### Example

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Steps
    level: 2
    blocks: []
```

This document writes `Steps` at level 1:

**`recipe.md`**

```markdown
# Recipe

# Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:3: error[TLMD_144]: heading `# Steps` must be level 2 (schema recipe, section "Steps")
1 error, 0 warnings
[exit 1]
```

Make it level 2:

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_26

A heading in an open region matches the text of a heading that the schema prohibits ([TLMD_26][sokf:tramlines:tramlines-md-spec#tlmd_26]).

### Message

```text
<file>:<line>: error[TLMD_26]: heading `<heading>` is prohibited (schema <id>)
```

### Cause

A section sets `headings-open` or `subheadings-open`, so undeclared headings can follow it. The schema lists the text of this heading under `sections-prohibited`.

### Fix

Remove the heading from the document, or change its text. If documents of this kind can contain the heading, remove its text from `sections-prohibited` in the schema.

### Example

This schema lets any deeper heading follow `Recipe`, except `Draft`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    subheadings-open: true
    blocks: []
sections-prohibited:
  - heading: Draft
```

This document contains a `Draft` heading:

**`recipe.md`**

```markdown
# Recipe

## Draft
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:3: error[TLMD_26]: heading `## Draft` is prohibited (schema recipe)
1 error, 0 warnings
[exit 1]
```

Rename the heading:

**`recipe.md`**

```markdown
# Recipe

## Tips
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_170

The schema prohibits a heading that one of its sections declares ([TLMD_170][sokf:tramlines:tramlines-md-spec#tlmd_170]).

### Message

```text
<file>:<line>: error[TLMD_170]: `sections-prohibited` names `<heading>`, which a section declares
```

### Cause

A heading of `sections-prohibited` matches the heading of a declared section, and the two apply in the same lifecycle states. The engine matches such a heading to its section, even inside an open region, so the prohibition never applies.

### Fix

If documents of this kind can contain the section, remove the heading from `sections-prohibited`. Otherwise, remove the section from the schema. If the two apply in different states, give each an `only` for its states.

### Example

This schema declares a `Tips` section and also prohibits a `Tips` heading:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    subheadings-open: true
    blocks: []
  - heading: Tips
    level: 2
    optional: true
    blocks: []
sections-prohibited:
  - heading: Tips
```

**`recipe.md`**

```markdown
# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.yaml:12: error[TLMD_170]: `sections-prohibited` names `Tips`, which a section declares
1 error, 0 warnings
[exit 1]
```

Remove the prohibition:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    subheadings-open: true
    blocks: []
  - heading: Tips
    level: 2
    optional: true
    blocks: []
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_29

Two sections, or two units, paired with one rule share a key ([TLMD_29][sokf:tramlines:tramlines-md-spec#tlmd_29]).

### Message

```text
<file>:<line>: error[TLMD_29]: key `<key>` is captured more than once; it was first captured at line <line> (schema <id>, <place of the rule>)
```

### Cause

A section or a unit rule sets `for-each-key-of`, so the document must contain one section or unit for each key that the named rule captures. The document contains two with the same key, in one scope. A heading can repeat although its section doesn't declare `repeat`. If the body of the repeated heading captures a key that the first body captured, the command reports the key with the heading, as [TLMD_151](#tlmd_151).

### Fix

Remove the second section from the document, or merge it into the first one.

### Example

In this schema, a notes section follows for each part in the list:

**`bike.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Parts
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: parts
        each:
          template: "${name:slug}"
          key: name
          example: wheel
  - heading: { template: "${name:slug} notes", key: name, example: wheel notes }
    level: 2
    repeat: true
    for-each-key-of: parts
    blocks: []
```

This document contains two sections for `wheel`:

**`bike.md`**

```markdown
# Parts

- wheel
- brake

## <a id="wheel"></a>wheel notes

## <a id="brake"></a>brake notes

## wheel notes
```

```console
$ tramlines validate --schema bike.yaml bike.md
bike.md:10: error[TLMD_29]: key `wheel` is captured more than once; it was first captured at line 6 (schema bike, section "${name:slug} notes")
1 error, 0 warnings
[exit 1]
```

Remove the second section:

**`bike.md`**

```markdown
# Parts

- wheel
- brake

## <a id="wheel"></a>wheel notes

## <a id="brake"></a>brake notes
```

```console
$ tramlines validate --schema bike.yaml bike.md
0 errors, 0 warnings
```

## TLMD_38

The document contains content before its first heading, and the schema declares no `preamble` ([TLMD_38][sokf:tramlines:tramlines-md-spec#tlmd_38]).

### Message

```text
<file>:<line>: error[TLMD_38]: content before the first heading, with no preamble declared (schema <id>)
```

### Cause

The body of the document begins with a block, such as a paragraph, before its first heading. Without a `preamble`, the schema declares no rule for that block.

### Fix

Remove the content from the document, or move it under a heading. If documents of this kind begin with content, declare a `preamble` with its `blocks` in the schema.

### Example

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
```

This document begins with a paragraph:

**`recipe.md`**

```markdown
Serves two.

# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:1: error[TLMD_38]: content before the first heading, with no preamble declared (schema recipe)
1 error, 0 warnings
[exit 1]
```

Remove the paragraph:

**`recipe.md`**

```markdown
# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-keys-and-repairs]: /docs/reference/errors-keys-and-repairs.md
[sokf:errors-nested-sections]: /docs/reference/errors-nested-sections.md
[sokf:errors-nested-sections#tlmd_31]: /docs/reference/errors-nested-sections.md#tlmd_31
[sokf:tramlines:tramlines-md-spec#tlmd_144]: /knowledge/spec/tramlines-md-spec.md#tlmd_144
[sokf:tramlines:tramlines-md-spec#tlmd_151]: /knowledge/spec/tramlines-md-spec.md#tlmd_151
[sokf:tramlines:tramlines-md-spec#tlmd_170]: /knowledge/spec/tramlines-md-spec.md#tlmd_170
[sokf:tramlines:tramlines-md-spec#tlmd_18]: /knowledge/spec/tramlines-md-spec.md#tlmd_18
[sokf:tramlines:tramlines-md-spec#tlmd_19]: /knowledge/spec/tramlines-md-spec.md#tlmd_19
[sokf:tramlines:tramlines-md-spec#tlmd_20]: /knowledge/spec/tramlines-md-spec.md#tlmd_20
[sokf:tramlines:tramlines-md-spec#tlmd_21]: /knowledge/spec/tramlines-md-spec.md#tlmd_21
[sokf:tramlines:tramlines-md-spec#tlmd_22]: /knowledge/spec/tramlines-md-spec.md#tlmd_22
[sokf:tramlines:tramlines-md-spec#tlmd_26]: /knowledge/spec/tramlines-md-spec.md#tlmd_26
[sokf:tramlines:tramlines-md-spec#tlmd_29]: /knowledge/spec/tramlines-md-spec.md#tlmd_29
[sokf:tramlines:tramlines-md-spec#tlmd_38]: /knowledge/spec/tramlines-md-spec.md#tlmd_38
