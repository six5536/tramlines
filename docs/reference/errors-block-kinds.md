---
type: ErrorReference
id: errors-block-kinds
title: 'Errors: kinds of block'
description: The errors that tramlines reports for what a block of each kind declares and contains, such as the tag of a code block or the header of a table, and how to fix each one.
order: 51
---

# Errors: kinds of block

These errors come from what a block of each kind must declare in the schema and contain in the document. Section 5 of the markdown specification, on blocks, defines them.

## TLMD_42

A block of the schema declares a key that doesn't apply to its kind, or contains a unit rule with a key out of place for that rule ([TLMD_42][sokf:tramlines:tramlines-md-spec#tlmd_42]).

### Message

```text
<file>:<line>: error[TLMD_42]: `<key>` does not apply to a `<kind>` block; it needs kind <required kind>
<file>:<line>: error[TLMD_42]: `text` applies to a paragraph, an item or a line, not a `<kind>` block
<file>:<line>: error[TLMD_42]: `blocks` applies to an item, not to a line
```

### Cause

The error is on the schema. Some keys of a block apply to one kind only: `lang` and `fence` to `code`, `columns` and `columns-open` to `table`, and `source-pattern` to `image`. The error also occurs when a unit rule with `text` sits under a block other than prose, a blockquote, or a list. It occurs too when a rule under `lines` declares `blocks`.

### Fix

In the schema, remove the key that doesn't apply, or change the `kind` of the block to a kind that accepts the key.

### Example

This schema declares `lang` on a prose block, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - { kind: prose, lang: text, example: Buy milk. }
```

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_42]: `lang` does not apply to a `prose` block; it needs kind code
1 error, 0 warnings
[exit 1]
```

Remove `lang`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - { kind: prose, example: Buy milk. }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_153

A block of the schema lacks a key that its kind requires ([TLMD_153][sokf:tramlines:tramlines-md-spec#tlmd_153]).

### Message

```text
<file>:<line>: error[TLMD_153]: a `code` block requires `lang`
<file>:<line>: error[TLMD_153]: a `table` block requires `columns`
<file>:<line>: error[TLMD_153]: a `group` block requires `blocks`
```

### Cause

The error is on the schema. The engine can't check a block of some kinds without one key. A `code` block needs `lang`, the tags that its fence can carry. A `table` block needs `columns`, its header row. A `group` block needs `blocks`, the blocks of one repetition. The block declares the kind and leaves the key out.

### Fix

Add the key that the message names to the block in the schema.

### Example

This schema declares a code block with no `lang`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: code, example: 'port: 80' }
```

**`note.md`**

````markdown
# Config

```yaml
port: 80
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_153]: a `code` block requires `lang`
1 error, 0 warnings
[exit 1]
```

Add `lang`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: code, lang: yaml, example: 'port: 80' }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_43

A code block lacks a language tag, or carries a tag that its block doesn't list in `lang` ([TLMD_43][sokf:tramlines:tramlines-md-spec#tlmd_43]).

### Message

```text
<file>:<line>: error[TLMD_43]: the code block has no tag (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_43]: the code block is tagged `<tag>`; expected `<tag>` or `<tag>` (schema <id>, <place of the rule>)
```

### Cause

The block of kind `code` lists its tags under `lang`. The opening fence of the code block in the document carries no tag, or a tag that isn't in the list. A `lang` of `any` accepts every tag, but the fence still needs one.

### Fix

In the document, tag the fence with one of the tags in `lang`. If the block can contain that language, add the tag to `lang` in the schema.

### Example

This schema accepts a `yaml` code block, and the document tags the block `json`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: code, lang: yaml, example: 'port: 80' }
```

**`note.md`**

````markdown
# Config

```json
{"port": 80}
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_43]: the code block is tagged `json`; expected `yaml` (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Write the block in YAML and tag it `yaml`:

**`note.md`**

````markdown
# Config

```yaml
port: 80
```
````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_145

A code block opens with fewer backticks than the `fence` of its block requires ([TLMD_145][sokf:tramlines:tramlines-md-spec#tlmd_145]).

### Message

```text
<file>:<line>: error[TLMD_145]: the fence has <count> backticks; at least <fence> are needed (schema <id>, <place of the rule>)
```

### Cause

The block of kind `code` sets `fence`, the minimum number of backticks in its fences. The fence of the code block in the document is shorter. A longer fence lets the block contain lines of three backticks, such as a Markdown example with its own code block.

### Fix

Open and close the code block in the document with at least as many backticks as `fence` sets. If a shorter fence is enough, lower `fence` in the schema, or remove it to use the default of three.

### Example

This schema expects Markdown in a fence of four backticks, and the document uses three:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - { kind: code, lang: markdown, fence: 4, example: '# Title' }
```

**`note.md`**

````markdown
# Config

```markdown
# Title
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_145]: the fence has 3 backticks; at least 4 are needed (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Use four backticks:

**`note.md`**

`````markdown
# Config

````markdown
# Title
````
`````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_44

The header row of a table differs from the `columns` that its block declares ([TLMD_44][sokf:tramlines:tramlines-md-spec#tlmd_44]).

### Message

```text
<file>:<line>: error[TLMD_44]: the table header is `<column> | <column>`; expected `<column> | <column>` (schema <id>, <place of the rule>)
```

### Cause

The block of kind `table` declares its header under `columns`. The header row of the table in the document differs in its columns, their names, or their order. With `columns-open: true`, the header can contain more columns after the declared ones, but it must start with them.

### Fix

Change the header row of the table in the document to the declared columns. If the table needs other columns, change `columns` in the schema, or set `columns-open: true` to accept more columns at the end.

### Example

This schema declares the columns `Item` and `Price`, and the document names the second column `Cost`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Prices
    level: 1
    blocks:
      - { kind: table, columns: [Item, Price], example: Milk | 1.20 }
```

**`note.md`**

```markdown
# Prices

| Item | Cost |
| ---- | ---- |
| Milk | 1.20 |
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_44]: the table header is `Item | Cost`; expected `Item | Price` (schema note, section "Prices" block 1)
1 error, 0 warnings
[exit 1]
```

Rename the column:

**`note.md`**

```markdown
# Prices

| Item | Price |
| ---- | ----- |
| Milk | 1.20  |
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_45

A block contains HTML, and its rule doesn't accept HTML ([TLMD_45][sokf:tramlines:tramlines-md-spec#tlmd_45]).

### Message

```text
<file>:<line>: error[TLMD_45]: inline HTML is not accepted here (schema <id>, <place of the rule>)
```

### Cause

A unit of the block contains an HTML tag, and the block doesn't set `inline-html: true`. The check doesn't count HTML comments or the anchors of keys as HTML. The error points at the first line of the block with HTML.

### Fix

Replace the HTML in the document with Markdown, or remove it. If blocks of this kind can contain HTML, set `inline-html: true` on the block in the schema.

### Example

This schema declares a prose block, and the document makes a word bold with HTML:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - { kind: prose, example: Buy milk. }
```

**`note.md`**

```markdown
# Note

Buy <b>milk</b>.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_45]: inline HTML is not accepted here (schema note, section "Note" block 1)
1 error, 0 warnings
[exit 1]
```

Use Markdown for the bold word:

**`note.md`**

```markdown
# Note

Buy **milk**.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_163

The path of an image doesn't match the `source-pattern` of its block ([TLMD_163][sokf:tramlines:tramlines-md-spec#tlmd_163]).

### Message

```text
<file>:<line>: error[TLMD_163]: the image `<path>` does not match `<pattern>`; a target of this kind reads <example> (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_163]: the image `<reference>` names a document; this rule takes what matches `<pattern>`, a file or an asset by its path; a target of this kind reads <example> (schema <id>, <place of the rule>)
```

### Cause

The block of kind `image` declares `source-pattern`, a regular expression, and the path of an image in the document doesn't match it. The engine matches the path as written. Under a host, the engine matches a path in the host's scheme, and the target of a region, by what the host resolves each one to ([TLMD_46][sokf:tramlines:tramlines-md-spec#tlmd_46]). When the host reports that an image or the target of a region names a document, the engine reports it whatever the pattern, because a pattern describes a path and a document has none ([TLMD_200][sokf:tramlines:tramlines-md-spec#tlmd_200]). The second message is for that case. `tramlines validate` runs without a host, so it reports only the first message, for an image.

### Fix

Change the path of the image in the document to one that the pattern accepts. If the block can contain images from other places, widen `source-pattern` in the schema. If the image or the region names a document, point it at a file or an asset instead.

### Example

This schema accepts SVG images from the `images` folder, and the document shows a PNG file from another folder:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Diagram
    level: 1
    blocks:
      - kind: image
        source-pattern: '^images/.+\.svg$'
        example: '![Flow](images/flow.svg)'
```

**`note.md`**

```markdown
# Diagram

![Flow](flow.png)
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_163]: the image `flow.png` does not match `^images/.+\.svg$`; a target of this kind reads ![Flow](images/flow.svg) (schema note, section "Diagram" block 1)
1 error, 0 warnings
[exit 1]
```

Point the image at an SVG file in `images`:

**`note.md`**

```markdown
# Diagram

![Flow](images/flow.svg)
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_47

A block of the schema declares `rules` beside `each` or `sequence` ([TLMD_47][sokf:tramlines:tramlines-md-spec#tlmd_47]).

### Message

```text
<file>:<line>: error[TLMD_47]: `rules` beside `each` or `sequence`: a block takes one of the three
```

### Cause

The error is on the schema. A block applies rules to its units in one of three ways: `each` applies one rule to every unit, `sequence` applies one rule to each position, and `rules` applies a list of rules as a grammar. The block declares `rules` and one of the others.

### Fix

In the schema, keep one of the three keys on the block. To keep the rule of `each` beside other rules, move it into `rules`.

### Example

This schema declares `each` and `rules` on one block, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        each: { template: 'STEP ${}', example: STEP Open the lid. }
        rules:
          - { template: 'NOTE ${}', min: 0, example: NOTE Use both hands. }
```

**`note.md`**

```markdown
# Steps

- STEP Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_47]: `rules` beside `each` or `sequence`: a block takes one of the three
1 error, 0 warnings
[exit 1]
```

Move the rule of `each` into `rules`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Steps
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - { template: 'STEP ${}', example: STEP Open the lid. }
          - { template: 'NOTE ${}', min: 0, example: NOTE Use both hands. }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_172

A blockquote contains a heading ([TLMD_172][sokf:tramlines:tramlines-md-spec#tlmd_172]).

### Message

```text
<file>:<line>: error[TLMD_172]: heading `<heading>` is inside a blockquote; a heading of the document stands outside every quote (schema <id>, <place of the rule>)
```

### Cause

A blockquote reads only its paragraphs. A heading of the document stands outside every quote, so a heading inside a quote starts no section. A heading inside a quote or a list within the quote counts too. A quote that holds only a heading reports this error and no missing paragraph. A heading inside a list item is a block of the item. [TLMD_162][sokf:errors-blocks#tlmd_162] reports such a heading where the rule of the item declares no `blocks`.

### Fix

Move the heading out of the quote, or write it as a paragraph of the quote.

### Example

This schema declares a quote and then a list:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Notes
    level: 1
    blocks:
      - kind: blockquote
        example: Keep the lid closed.
      - kind: bullet-list
        example: Open the lid.
```

This document puts a heading inside the quote:

**`note.md`**

```markdown
# Notes

> Keep the lid closed.
>
> ## Why

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_172]: heading `## Why` is inside a blockquote; a heading of the document stands outside every quote (schema note, section "Notes" block 1)
1 error, 0 warnings
[exit 1]
```

Write the heading as a paragraph of the quote:

**`note.md`**

```markdown
# Notes

> Keep the lid closed.
>
> Why: the steam escapes.

- Open the lid.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-blocks#tlmd_162]: /docs/reference/errors-blocks.md#tlmd_162
[sokf:tramlines:tramlines-md-spec#tlmd_145]: /knowledge/spec/tramlines-md-spec.md#tlmd_145
[sokf:tramlines:tramlines-md-spec#tlmd_153]: /knowledge/spec/tramlines-md-spec.md#tlmd_153
[sokf:tramlines:tramlines-md-spec#tlmd_163]: /knowledge/spec/tramlines-md-spec.md#tlmd_163
[sokf:tramlines:tramlines-md-spec#tlmd_172]: /knowledge/spec/tramlines-md-spec.md#tlmd_172
[sokf:tramlines:tramlines-md-spec#tlmd_200]: /knowledge/spec/tramlines-md-spec.md#tlmd_200
[sokf:tramlines:tramlines-md-spec#tlmd_42]: /knowledge/spec/tramlines-md-spec.md#tlmd_42
[sokf:tramlines:tramlines-md-spec#tlmd_43]: /knowledge/spec/tramlines-md-spec.md#tlmd_43
[sokf:tramlines:tramlines-md-spec#tlmd_44]: /knowledge/spec/tramlines-md-spec.md#tlmd_44
[sokf:tramlines:tramlines-md-spec#tlmd_45]: /knowledge/spec/tramlines-md-spec.md#tlmd_45
[sokf:tramlines:tramlines-md-spec#tlmd_46]: /knowledge/spec/tramlines-md-spec.md#tlmd_46
[sokf:tramlines:tramlines-md-spec#tlmd_47]: /knowledge/spec/tramlines-md-spec.md#tlmd_47
