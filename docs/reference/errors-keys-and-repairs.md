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

Each key receives an anchor whose id is the slug of the key. Each heading also receives a slug. The slug is the text in lowercase. Spaces become hyphens, and most punctuation drops out. A heading's slug comes from its plain text, without the markers of emphasis, code, links, and inline HTML, as GitHub forms the ids of headings: `## _Foo_` has the slug `foo`, and `## See [the guide](x)` the slug `see-the-guide`. Two keys that differ only in case, or a key that reads like a heading of the document, produce the same id twice. A key that a rule with `for-each-key-of` captures refers to a key and receives no anchor. The command doesn't check its slug for this error.

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

Each key that a rule captures needs one anchor, `<a id="<slug>"></a>`, so that a link can open the page at the key. The anchor goes at the start of the text of a heading, or directly before the text of an item, a paragraph, or a cell. Before a code block, the anchor goes on a separate line. A key that a unit rule with `for-each-key-of` captures refers to the key and needs no anchor. In the document, the anchor is missing, in the wrong place, or written twice, or it belongs to a key that no longer exists. The command reports an anchor written again beside a second capture of a key as [TLMD_148](#tlmd_148) or [TLMD_101](#tlmd_101) instead. An anchor beside a capture with `for-each-key-of` goes with the error of that pairing. The repair removes both kinds of anchor. An item, a paragraph, a row, or a line of a code block whose text doesn't match its rule captures no key. The command leaves the anchors of such a unit alone while it reports an error or a warning on the unit that the repair can't correct, or reports that no rule takes the unit. A heading whose text doesn't match the template of its section captures no key either. The command leaves the anchors of a heading that captures no key alone while it reports an error or a warning on where the heading stands that the repair can't correct, such as that the heading isn't declared ([TLMD_19][sokf:errors-sections#tlmd_19]) or is at the wrong level ([TLMD_144][sokf:errors-sections#tlmd_144]). While no declared section takes such a heading and the command reports it, such as that it isn't declared, that it skips a level ([TLMD_20][sokf:errors-sections#tlmd_20]), or that it is prohibited in an open region ([TLMD_26][sokf:errors-sections#tlmd_26]), the command leaves alone the anchors of each unit between it and the next heading too, because it checks none of them. The anchors under a heading that an open region admits, and on which the command reports nothing, follow the usual rule. Where the blocks under a heading hold an error that can give a block to another position, such as an item that breaks the pattern of its list, the command leaves the anchors of each unit of those blocks alone too, because it can't be sure which key a unit carries ([TLMD_109][sokf:tramlines:tramlines-md-spec#tlmd_109]). It reports no anchor of the unit or the heading that names no key, and the repair leaves each such anchor where it is, so that links to it keep working while you correct the text. The repair that writes a setext heading as ATX writes the heading's anchors at the start of its text. A heading inside a list item, and outside every quote, keeps the anchors on its text in the same way while the command reports an error on it, such as that no position takes it, but not a line of anchors before it, as a paragraph doesn't.

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

A heading or a unit that the schema binds to a frontmatter value doesn't equal that value ([TLMD_165][sokf:tramlines:tramlines-md-spec#tlmd_165]), and the repair rewrites it to the value ([TLMD_283][sokf:tramlines:tramlines-md-spec#tlmd_283]).

### Message

```text
<file>:<line>: error[TLMD_165]: heading `<heading>` must equal the frontmatter `<key>`: `<value>` (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_165]: `<text>` must equal the frontmatter `<key>`: `<value>` (schema <id>, <place of the rule>)
```

### Cause

The schema declares `heading: { from: <key> }` on a section, or `text: { from: <key> }` on a unit rule. The text in the document must equal the value of that key, and it differs.

### Fix

In the document, change the heading or the text to the value of the frontmatter key, or run `tramlines validate --fix`, which makes the change. A heading whose value ends in a space and `#` is written with that `#` escaped, as `# Notes \#`, because `# Notes #` reads as `Notes`. A heading holds the value without the spaces at its ends, so `title: "Notes "` matches `# Notes` ([TLMD_282][sokf:tramlines:tramlines-md-spec#tlmd_282]). A paragraph or an item whose value would start another block, such as `# x`, `- x`, or `1. x`, is written with a backslash before the punctuation character that starts the block, as `\# x`, `\- x`, or `1\. x`. On the line after a link reference definition, a value that the definition would read as its title, such as `(Right)`, or that would make a table of the definition's line, such as `|---|`, is written with the first backslash that keeps it the unit's text, as `\(Right)` or `|\---|` ([TLMD_291][sokf:tramlines:tramlines-md-spec#tlmd_291]). `--fix` leaves the text as it stands where no backslash does, and where an empty value would make the first item of a list after a paragraph or a definition part of it, or leave out of the item what it holds after a blank line. If the value is wrong, change the frontmatter instead. `--fix` leaves the text as it stands where the check can't be sure that the unit belongs to the bound rule: where the blocks under the same heading hold another error that decides which position takes a block, such as an item that breaks the pattern of its list, or where the unit has the shape of another rule of the section ([TLMD_279][sokf:tramlines:tramlines-md-spec#tlmd_279], [TLMD_280][sokf:tramlines:tramlines-md-spec#tlmd_280]). Correct that error first, and run `--fix` again. It leaves a heading as it stands where an open region of the section before it admits the heading, or where another heading of the document already reads as the value ([TLMD_285][sokf:tramlines:tramlines-md-spec#tlmd_285]): rename or delete the heading that you don't mean.

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

## TLMD_276

A paragraph or an item is bound to a frontmatter value that no paragraph or item can hold, so the repair can't write it ([TLMD_276][sokf:tramlines:tramlines-md-spec#tlmd_276]).

### Message

```text
<file>:<line>: error[TLMD_276]: the frontmatter `<key>` is empty, which no <unit> can hold (schema <id>, key `<key>`)
<file>:<line>: error[TLMD_276]: the frontmatter `<key>` holds `<value>`, which no <unit> can hold: on a line of its own it starts <block> (schema <id>, key `<key>`)
```

### Cause

The schema declares `text: { from: <key> }` on the rule of a paragraph or an item, and no text of that kind reads as the value of the key. A paragraph holds some text, so an empty value has none. A line that starts with an HTML tag, such as `<div>`, starts an HTML block, and an escaped `\<div>` reads as other text. The error is on the line of the key in the frontmatter, and `tramlines validate --fix` leaves the unit as it stands. A value that starts another block, such as `# x` or `- x`, is no error: the repair writes it with a backslash before the punctuation character that starts the block.

### Fix

In the frontmatter, give the key a value that a paragraph or an item can hold, such as text before the tag.

### Example

This schema binds the paragraph of the list to the frontmatter `description`:

**`list.yaml`**

```yaml
line-limit: 0
frontmatter:
  description:
    example: What to buy this week.
sections:
  - heading: List
    level: 1
    blocks:
      - kind: prose
        each:
          text: { from: description }
```

The description of this document is an HTML tag:

**`list.md`**

```markdown
---
description: "<div>"
---

# List

What to buy this week.
```

```console
$ tramlines validate --schema list.yaml list.md
list.md:2: error[TLMD_276]: the frontmatter `description` holds `<div>`, which no paragraph can hold: on a line of its own it starts an HTML block (schema list, key `description`)
1 error, 0 warnings
[exit 1]
```

Give the description text that a paragraph holds:

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

## TLMD_312

A paragraph or an item reads as the frontmatter value that it is bound to, and its block refuses what the value holds ([TLMD_312][sokf:tramlines:tramlines-md-spec#tlmd_312]).

### Message

```text
<file>:<line>: error[TLMD_312]: the frontmatter `<key>` holds `<value>`, which the <unit> bound to it can't hold: <reason> (schema <id>, key `<key>`)
```

### Cause

The schema declares `text: { from: <key> }` on the rule of a paragraph or an item, and the unit reads as the value, but the value holds what the unit's block refuses: inline HTML in the unit's own text, in a block without `inline-html`, or a text that a ban of the rule matches. The reason names the refusal. A ban of the whole document judges the whole line, with the item's marker and any label beside the value, so it stays on the unit's line as `TL_75`. The frontmatter dictates the unit's text, and `tramlines validate --fix` writes the value back, so the finding is on the line of the key in the frontmatter, once for each key, and the unit draws none. It is a warning only where each rule that refuses the value reports a warning.

### Fix

In the frontmatter, change the value so that the unit's block accepts it, such as by writing the HTML as text, or by leaving out the banned words.

### Example

This schema binds the paragraph of the list to the frontmatter `description`, and accepts no inline HTML:

**`list.yaml`**

```yaml
line-limit: 0
frontmatter:
  description:
    example: What to buy this week.
sections:
  - heading: List
    level: 1
    blocks:
      - kind: prose
        each:
          text: { from: description }
```

The description holds a tag:

**`list.md`**

```markdown
---
description: "What to buy <b>this</b> week."
---

# List

What to buy <b>this</b> week.
```

```console
$ tramlines validate --schema list.yaml list.md
list.md:2: error[TLMD_312]: the frontmatter `description` holds `What to buy <b>this</b> week.`, which the paragraph bound to it can't hold: inline HTML is not accepted here (schema list, key `description`)
1 error, 0 warnings
[exit 1]
```

Write the description, and the paragraph, without the tag:

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

[sokf:errors-sections#tlmd_144]: /docs/reference/errors-sections.md#tlmd_144
[sokf:errors-sections#tlmd_151]: /docs/reference/errors-sections.md#tlmd_151
[sokf:errors-sections#tlmd_19]: /docs/reference/errors-sections.md#tlmd_19
[sokf:errors-sections#tlmd_20]: /docs/reference/errors-sections.md#tlmd_20
[sokf:errors-sections#tlmd_26]: /docs/reference/errors-sections.md#tlmd_26
[sokf:tramlines:tramlines-md-spec#tlmd_101]: /knowledge/spec/tramlines-md-spec.md#tlmd_101
[sokf:tramlines:tramlines-md-spec#tlmd_109]: /knowledge/spec/tramlines-md-spec.md#tlmd_109
[sokf:tramlines:tramlines-md-spec#tlmd_137]: /knowledge/spec/tramlines-md-spec.md#tlmd_137
[sokf:tramlines:tramlines-md-spec#tlmd_138]: /knowledge/spec/tramlines-md-spec.md#tlmd_138
[sokf:tramlines:tramlines-md-spec#tlmd_143]: /knowledge/spec/tramlines-md-spec.md#tlmd_143
[sokf:tramlines:tramlines-md-spec#tlmd_148]: /knowledge/spec/tramlines-md-spec.md#tlmd_148
[sokf:tramlines:tramlines-md-spec#tlmd_165]: /knowledge/spec/tramlines-md-spec.md#tlmd_165
[sokf:tramlines:tramlines-md-spec#tlmd_168]: /knowledge/spec/tramlines-md-spec.md#tlmd_168
[sokf:tramlines:tramlines-md-spec#tlmd_276]: /knowledge/spec/tramlines-md-spec.md#tlmd_276
[sokf:tramlines:tramlines-md-spec#tlmd_279]: /knowledge/spec/tramlines-md-spec.md#tlmd_279
[sokf:tramlines:tramlines-md-spec#tlmd_280]: /knowledge/spec/tramlines-md-spec.md#tlmd_280
[sokf:tramlines:tramlines-md-spec#tlmd_282]: /knowledge/spec/tramlines-md-spec.md#tlmd_282
[sokf:tramlines:tramlines-md-spec#tlmd_283]: /knowledge/spec/tramlines-md-spec.md#tlmd_283
[sokf:tramlines:tramlines-md-spec#tlmd_285]: /knowledge/spec/tramlines-md-spec.md#tlmd_285
[sokf:tramlines:tramlines-md-spec#tlmd_29]: /knowledge/spec/tramlines-md-spec.md#tlmd_29
[sokf:tramlines:tramlines-md-spec#tlmd_291]: /knowledge/spec/tramlines-md-spec.md#tlmd_291
[sokf:tramlines:tramlines-md-spec#tlmd_312]: /knowledge/spec/tramlines-md-spec.md#tlmd_312
