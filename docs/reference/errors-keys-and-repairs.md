---
type: ErrorReference
id: errors-keys-and-repairs
title: 'Errors: keys and repairs'
description: The errors that tramlines reports for keys and their anchors, and for the text that a repair rewrites to match the frontmatter or the schema.
order: 72
---

# Errors: keys and repairs

These errors come from keys and their anchors, and from headings and text that must equal a frontmatter value or the current form of a heading. The command `tramlines validate --fix` repairs the anchors. It also rewrites each heading and text that the frontmatter or the schema determines. Sections 7 and 10 of the Markdown specification define these errors.

## TLMD_101

A key has the same slug as another key or as a heading of the document, so the two would share one anchor ([TLMD_101][sokf:tramlines:tramlines-md-spec#tlmd_101]).

### Message

```text
<file>:<line>: error[TLMD_101]: key `<key>` has the slug `<slug>` of the heading at line <line>; keys and headings share one namespace (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_101]: key `<key>` has the slug `<slug>` of key `<other key>` at line <line>; each key needs a slug of its own (schema <id>, <place of the rule>)
```

### Cause

Each key receives an anchor whose id is the slug of the key. Each heading also receives a slug. The slug is the text in lowercase. Spaces become hyphens, and most punctuation drops out. Two keys that differ only in case, or a key that reads like a heading of the document, produce the same id twice. A key that a rule with `for-each-key-of` captures refers to a key and receives no anchor. The command doesn't check its slug for this error.

### Fix

In the document, rename the key or the heading so that each slug is different.

### Example

This schema captures a key from each item of a list:

**`pantry.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Pantry
    level: 1
    blocks:
      - kind: bullet-list
        each:
          template: "${name:word}: ${}"
          key: name
          example: "milk: two litres"
```

The key `pantry` has the slug of the heading `Pantry`:

**`pantry.md`**

```markdown
# Pantry

- <a id="pantry"></a>pantry: the top shelf
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
pantry.md:3: error[TLMD_101]: key `pantry` has the slug `pantry` of the heading at line 1; keys and headings share one namespace (schema pantry, section "Pantry")
1 error, 0 warnings
[exit 1]
```

Rename the key, and its anchor with it:

**`pantry.md`**

```markdown
# Pantry

- <a id="shelf"></a>shelf: the top shelf
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
0 errors, 0 warnings
```

## TLMD_148

Rules capture one key twice in the scope that `unique` declares ([TLMD_148][sokf:tramlines:tramlines-md-spec#tlmd_148]).

### Message

```text
<file>:<line>: error[TLMD_148]: key `<key>` is captured more than once; it was first captured at line <line> (schema <id>, <place of the rule>)
```

### Cause

A key identifies one part of a document. Each key that the rules capture must therefore be different within its scope, whichever rule captures it. The scope is the document unless `unique` names a wider one that a host adds. The document contains two items, headings, or lines with the same key. Suppose a heading repeats, its section doesn't declare `repeat`, and the body of the repeat captures a key that the first body captured. The command reports that key with the heading, as [TLMD_151][sokf:errors-sections#tlmd_151]. For a key that pairs with another rule through `for-each-key-of`, the command reports [TLMD_29][sokf:tramlines:tramlines-md-spec#tlmd_29] instead.

### Fix

In the document, merge the two parts, or rename one of the keys.

### Example

This example uses the `pantry.yaml` of [TLMD_101](#tlmd_101). This document lists `milk` twice:

**`pantry.md`**

```markdown
# Pantry

- <a id="milk"></a>milk: two litres
- milk: one more
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
pantry.md:4: error[TLMD_148]: key `milk` is captured more than once; it was first captured at line 3 (schema pantry, section "Pantry")
1 error, 0 warnings
[exit 1]
```

Merge the two items:

**`pantry.md`**

```markdown
# Pantry

- <a id="milk"></a>milk: three litres
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
0 errors, 0 warnings
```

## TLMD_109

A key has no anchor, an anchor isn't where its key is, an anchor names no key, or an anchor is written twice ([TLMD_109][sokf:tramlines:tramlines-md-spec#tlmd_109]).

### Message

```text
<file>:<line>: error[TLMD_109]: key `<key>` has no anchor; its repair writes `<a id="<slug>"></a>` before it
<file>:<line>: error[TLMD_109]: the anchor `#<slug>` is not where its key `<key>` is; its repair moves it
<file>:<line>: error[TLMD_109]: the anchor `#<slug>` names no key of this document; its repair removes it
<file>:<line>: error[TLMD_109]: the anchor `#<slug>` occurs more than once; its repair keeps one
```

### Cause

Each key that a rule captures needs one anchor, `<a id="<slug>"></a>`, so that a link can open the page at the key. The anchor goes at the start of the text of a heading, or directly before the text of an item, a paragraph, or a cell. Before a code block, the anchor goes on a separate line. A key that a unit rule with `for-each-key-of` captures refers to the key and needs no anchor. In the document, the anchor is missing, in the wrong place, or written twice, or it belongs to a key that no longer exists. The command reports an anchor written again beside a second capture of a key as [TLMD_148](#tlmd_148) or [TLMD_101](#tlmd_101) instead. An anchor beside a capture with `for-each-key-of` goes with the error of that pairing. The repair removes both kinds of anchor.

### Fix

In the document, write the anchor that the message names in the place where it belongs, and delete any extra or orphaned anchor. `tramlines validate --fix` does both.

### Example

This example uses the `pantry.yaml` of [TLMD_101](#tlmd_101). The key `milk` has no anchor:

**`pantry.md`**

```markdown
# Pantry

- milk: two litres
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
pantry.md:3: error[TLMD_109]: key `milk` has no anchor; its repair writes `<a id="milk"></a>` before it
1 error, 0 warnings
[exit 1]
```

Write the anchor before the text of the item:

**`pantry.md`**

```markdown
# Pantry

- <a id="milk"></a>milk: two litres
```

```console
$ tramlines validate --schema pantry.yaml pantry.md
0 errors, 0 warnings
```

## TLMD_165

A heading or a unit that the schema binds to a frontmatter value doesn't equal that value ([TLMD_165][sokf:tramlines:tramlines-md-spec#tlmd_165]).

### Message

```text
<file>:<line>: error[TLMD_165]: heading `<heading>` must equal the frontmatter `<key>`: `<value>` (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_165]: `<text>` must equal the frontmatter `<key>`: `<value>` (schema <id>, <place of the rule>)
```

### Cause

The schema declares `heading: { from: <key> }` on a section, or `text: { from: <key> }` on a unit rule. The text in the document must equal the value of that key, and it differs.

### Fix

In the document, change the heading or the text to the value of the frontmatter key, or run `tramlines validate --fix`, which makes the change. If the value is wrong, change the frontmatter instead.

### Example

This schema requires the level-1 heading to equal the frontmatter `title`:

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
```

The heading of this document differs from its title:

**`list.md`**

```markdown
---
title: Groceries
---

# Shopping
```

```console
$ tramlines validate --schema list.yaml list.md
list.md:5: error[TLMD_165]: heading `Shopping` must equal the frontmatter `title`: `Groceries` (schema list, section "<title>")
1 error, 0 warnings
[exit 1]
```

Change the heading:

**`list.md`**

```markdown
---
title: Groceries
---

# Groceries
```

```console
$ tramlines validate --schema list.yaml list.md
0 errors, 0 warnings
```

## TLMD_168

A heading, a paragraph, or an item is bound to a frontmatter key that the document doesn't set ([TLMD_168][sokf:tramlines:tramlines-md-spec#tlmd_168]).

### Message

```text
<file>:<line>: error[TLMD_168]: heading `<heading>` is bound to the frontmatter `<key>`, which is absent (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_168]: `<text>` is bound to the frontmatter `<key>`, which is absent (schema <id>, <place of the rule>)
```

### Cause

The schema declares `{ from: <key> }` as the heading of a section or as the `text` of a unit rule. The frontmatter of the document doesn't contain the key, so the heading or the unit has no value to equal. The key is `optional`: when the schema requires the key, the command reports it missing as `TL_54` and nothing more.

### Fix

Add the key to the frontmatter of the document, with the text that the heading or the unit holds as its value. Then `tramlines validate --fix` keeps the two equal.

### Example

This schema binds the first paragraph to the optional frontmatter `description`:

**`list.yaml`**

```yaml
line-limit: 0
frontmatter:
  description:
    optional: true
    example: What to buy this week.
sections:
  - heading: List
    level: 1
    blocks:
      - kind: prose
        each:
          text: { from: description }
```

This document has no `description`:

**`list.md`**

```markdown
# List

What to buy this week.
```

```console
$ tramlines validate --schema list.yaml list.md
list.md:3: error[TLMD_168]: `What to buy this week.` is bound to the frontmatter `description`, which is absent (schema list, section "List" block 1)
1 error, 0 warnings
[exit 1]
```

Add the key:

**`list.md`**

```markdown
---
description: What to buy this week.
---

# List

What to buy this week.
```

```console
$ tramlines validate --schema list.yaml list.md
0 errors, 0 warnings
```

## TLMD_137

A heading matches an old text of its section, which the section lists under `renamed-from` ([TLMD_137][sokf:tramlines:tramlines-md-spec#tlmd_137]).

### Message

```text
<file>:<line>: error[TLMD_137]: heading `<old text>` was renamed; it is now `<new text>` (schema <id>, <place of the rule>)
```

### Cause

The schema renamed the section and lists the old text under `renamed-from`. The document still contains the old heading.

### Fix

Change the heading in the document to the text that the message names, or run `tramlines validate --fix`, which makes the change.

### Example

This schema renamed `Steps` to `Method`:

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
  - heading: Method
    level: 2
    renamed-from: Steps
    blocks: []
```

This document still contains `Steps`:

**`recipe.md`**

```markdown
# Recipe

## Steps
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:3: error[TLMD_137]: heading `Steps` was renamed; it is now `Method` (schema recipe, section "Method")
1 error, 0 warnings
[exit 1]
```

Rename the heading:

**`recipe.md`**

```markdown
# Recipe

## Method
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

## TLMD_143

A section declares `renamed-from`, but its heading is a template or a pattern ([TLMD_143][sokf:tramlines:tramlines-md-spec#tlmd_143]).

### Message

```text
<file>:<line>: error[TLMD_143]: `renamed-from` beside a heading that varies: a former heading is rewritten to exact text
```

### Cause

`renamed-from` lists the former text of a heading, so that a document with the old heading is reported and repaired to the current one. A heading with a `template` or a `pattern` has no single current text to repair it to. The error is on the schema file.

### Fix

In the schema, delete `renamed-from` from the section, or give the section a literal heading.

### Example

This section declares a heading that varies, and a former heading:

**`plan.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Week ${n:int}", example: Week 1 }
    renamed-from: Plan
    level: 1
    blocks: []
```

**`plan.md`**

```markdown
# Week 1
```

```console
$ tramlines validate --schema plan.yaml plan.md
plan.yaml:4: error[TLMD_143]: `renamed-from` beside a heading that varies: a former heading is rewritten to exact text
1 error, 0 warnings
[exit 1]
```

Delete `renamed-from`:

**`plan.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Week ${n:int}", example: Week 1 }
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema plan.yaml plan.md
0 errors, 0 warnings
```

## TLMD_138

A heading is a setext heading, underlined with `=` or `-`, where tramlines expects an ATX heading that begins with `#` ([TLMD_138][sokf:tramlines:tramlines-md-spec#tlmd_138]).

### Message

```text
<file>:<line>: error[TLMD_138]: heading `<heading>` is setext; write it as ATX (schema <id>)
```

### Cause

The document writes a heading as a line of text with a line of `=` or `-` under it. The tramlines Markdown specification accepts only ATX headings, whatever the schema declares.

### Fix

In the document, write the heading with `#` signs: one for level 1, two for level 2. Alternatively, run `tramlines validate --fix`, which rewrites the heading.

### Example

**`recipe.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Recipe
    level: 1
    blocks: []
```

This document underlines its heading:

**`recipe.md`**

```markdown
Recipe
======
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
recipe.md:1: error[TLMD_138]: heading `Recipe` is setext; write it as ATX (schema recipe)
1 error, 0 warnings
[exit 1]
```

Write it with a `#`:

**`recipe.md`**

```markdown
# Recipe
```

```console
$ tramlines validate --schema recipe.yaml recipe.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-sections#tlmd_151]: /docs/reference/errors-sections.md#tlmd_151
[sokf:tramlines:tramlines-md-spec#tlmd_101]: /knowledge/spec/tramlines-md-spec.md#tlmd_101
[sokf:tramlines:tramlines-md-spec#tlmd_109]: /knowledge/spec/tramlines-md-spec.md#tlmd_109
[sokf:tramlines:tramlines-md-spec#tlmd_137]: /knowledge/spec/tramlines-md-spec.md#tlmd_137
[sokf:tramlines:tramlines-md-spec#tlmd_138]: /knowledge/spec/tramlines-md-spec.md#tlmd_138
[sokf:tramlines:tramlines-md-spec#tlmd_143]: /knowledge/spec/tramlines-md-spec.md#tlmd_143
[sokf:tramlines:tramlines-md-spec#tlmd_148]: /knowledge/spec/tramlines-md-spec.md#tlmd_148
[sokf:tramlines:tramlines-md-spec#tlmd_165]: /knowledge/spec/tramlines-md-spec.md#tlmd_165
[sokf:tramlines:tramlines-md-spec#tlmd_168]: /knowledge/spec/tramlines-md-spec.md#tlmd_168
[sokf:tramlines:tramlines-md-spec#tlmd_29]: /knowledge/spec/tramlines-md-spec.md#tlmd_29
