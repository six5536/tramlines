---
type: HowTo
id: how-to-describe-blocks
title: Describe blocks and their units
description: Declare the paragraphs, lists, tables, code blocks, and images of a section, the number of units in each, and the shape of each unit.
order: 70
---

# Describe blocks and their units

When the body of a section must contain certain paragraphs, lists, tables, or code, declare them under the `blocks` of the section. At the end, a body with a missing block, an extra block, or a unit of the wrong shape fails. The error names the line of the document where the problem is.

## Before you begin

- A schema with sections. [Describe sections and headings][sokf:how-to-describe-sections] shows how to write one.
- Templates and patterns, as in [Write the shape of a line][sokf:how-to-write-line-shapes].

## Steps

1. List the blocks of the body in order, each with a `kind`: `prose`, `bullet-list`, `numbered-list`, `table`, `code`, `blockquote`, `image`, `html`, `thematic-break`, `references` (link reference definitions), or `group`. tramlines reads the body as CommonMark blocks. Consecutive paragraphs form one `prose` block, and a paragraph of only images forms an `image` block. tramlines skips HTML comments, footnote definitions, and generated blocks when it matches blocks.

   If you leave out `blocks`, the body must be one prose block. Write `blocks: []` for an empty body.

2. Bound the units of each block. A unit is a paragraph of a prose block or a blockquote, an item of a list, a row of a table, or one repetition of a group. By default, a block needs at least one unit and has no upper limit. Set `min` or `max` to change these bounds.

3. Set `optional: true` on a block that a body can leave out. Set `repeat: true` on a block that can occur more than once in a row. To accept the blocks of a body in any order, set `blocks-ordered: false` on the section.

4. Give the units a shape:

   - `each` applies one rule to every unit.
   - `sequence` applies one rule to each position, in order, and so sets the number of units. Set `optional` or `repeat` on an entry to let its position be absent or recur.
   - `text` sets the whole text of a unit. `{ from: description }` requires the unit to equal the frontmatter value, as `heading: { from: title }` does for a heading.

   A block takes only one of `each`, `sequence`, or `rules`. [Select and count units and lines][sokf:how-to-select-and-count-lines] describes `rules`.

5. Describe what an item contains besides its own text. tramlines matches the text of an item without its nested blocks. To allow a nested list or code block, describe it under `blocks` on the rule of the item. Without `blocks` on the rule, an item can contain only its text.

6. Add the keys of each kind:

   - A `code` block needs `lang`: a tag, a list of tags, or `any`. A fence without a tag always fails. `fence` sets the minimum number of backticks. The default is three.
   - A `table` needs `columns`, the cells of its header row in order. `columns-open: true` accepts more columns after them. A unit of a table is a row, and a template matches the row as its cells separated by ` | `.
   - An `image` block takes `source-pattern`, a regular expression that each image path must match.
   - Any block takes `inline-html: true`. Without it, HTML other than a comment fails.

   A key that doesn't apply to the kind of its block is an error in the schema. This schema describes the notes of a meeting:

**`meeting.yaml`**

```yaml
line-limit: 80
frontmatter:
  title:
    example: Weekly sync
  description:
    example: The team agreed the release date.
types:
  date: '\d{4}-\d{2}-\d{2}'
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        each: { text: { from: description } }
      - kind: bullet-list
        sequence:
          - { template: "Date: ${:date}", example: "Date: 2026-09-30" }
          - { template: "Chair: ${}", example: "Chair: Ana" }
  - heading: Attendees
    level: 2
    blocks:
      - kind: table
        columns: [Name, Role]
        each: { template: "${} | ${}", example: "Ana | Chair" }
  - heading: Decisions
    level: 2
    blocks:
      - kind: numbered-list
        max: 5
        each:
          template: "${}."
          example: "Ship version 2 on Friday."
          blocks:
            - kind: blockquote
              optional: true
              example: Ana asked for one more day of testing.
      - kind: code
        lang: [text, console]
        optional: true
        example: npm run release
  - heading: Photos
    level: 2
    optional: true
    blocks:
      - kind: image
        source-pattern: '^images/.+\.png$'
        example: "![Whiteboard](images/board.png)"
```

7. Check a document:

**`sync.md`**

````markdown
---
title: Weekly sync
description: The team agreed the release date.
---

# Weekly sync

The team agreed the release date.

- Date: 2026-09-30
- Chair: Ana

## Attendees

| Name | Role  |
| ---- | ----- |
| Ana  | Chair |
| Ben  | Notes |

## Decisions

1. Ship version 2 on Friday.

   > Ana asked for one more day of testing.

2. Move the retro to Monday.

```text
npm run release
```

## Photos

![Whiteboard](images/board.png)
````

```console
$ tramlines validate --schema meeting.yaml sync.md
0 errors, 0 warnings
```

8. Read the errors for blocks. In this document, the second line of the sequence is missing, a paragraph stands before the table, the table has a different header row, a decision contains HTML, the code block has a tag that the schema doesn't list, and the photo is in the wrong folder:

**`sync.md`**

````markdown
---
title: Weekly sync
description: The team agreed the release date.
---

# Weekly sync

The team agreed the release date.

- Date: 2026-09-30

## Attendees

Everyone came.

| Name | Team |
| ---- | ---- |
| Ana  | Web  |

## Decisions

1. Move the retro to <b>Monday</b>.

```sh
npm run release
```

## Photos

![Whiteboard](board.png)
````

```console
$ tramlines validate --schema meeting.yaml sync.md
sync.md:10: error[TLMD_147]: position 2 of the sequence is missing (schema meeting, section "<title>" block 2)
sync.md:14: error[TLMD_39]: unexpected prose block (schema meeting, section "Attendees")
sync.md:16: error[TLMD_44]: the table header is `Name | Team`; expected `Name | Role` (schema meeting, section "Attendees" block 1)
sync.md:22: error[TLMD_45]: inline HTML is not accepted here (schema meeting, section "Decisions" block 1)
sync.md:24: error[TLMD_43]: the code block is tagged `sh`; expected `text` or `console` (schema meeting, section "Decisions" block 2)
sync.md:30: error[TLMD_163]: the image `board.png` does not match `^images/.+\.png$`; a target of this kind reads ![Whiteboard](images/board.png) (schema meeting, section "Photos" block 1)
6 errors, 0 warnings
[exit 1]
```

A block that the section doesn't declare causes one error, and tramlines still checks the next block against its rule. Here the paragraph under `Attendees` fails, and tramlines validates the table as usual. Each error names a block by its position in the section. The name `"<title>"` stands for a heading taken from the frontmatter.

9. To repeat a run of blocks as one unit, wrap them in a `group` with its own `blocks`. A group always repeats, so `min` and `max` bound the number of repetitions. Here each repetition is a paragraph with a question, followed by a paragraph with its answer:

**`faq.yaml`**

```yaml
line-limit: 60
sections:
  - heading: FAQ
    level: 1
    blocks:
      - kind: group
        min: 2
        blocks:
          - kind: prose
            max: 1
            each: { template: "Q: ${}?", example: "Q: Can I bring a guest?" }
          - kind: prose
            max: 1
            example: Yes, tell the host a day before.
```

**`faq.md`**

```markdown
# FAQ

Q: Can I bring a guest?

Yes, tell the host a day before.
```

```console
$ tramlines validate --schema faq.yaml faq.md
faq.md:3: error[TLMD_146]: the group block needs at least 2 repetitions; it has 1 (schema faq, section "FAQ" block 1)
1 error, 0 warnings
[exit 1]
```

10. Add a second question and answer:

**`faq.md`**

```markdown
# FAQ

Q: Can I bring a guest?

Yes, tell the host a day before.

Q: Is there parking?

No, take the bus.
```

```console
$ tramlines validate --schema faq.yaml faq.md
0 errors, 0 warnings
```

The two questions and two answers form four paragraphs in a row. tramlines divides them among the rules. When two rules of the same kind follow each other, the first takes as many units as its `max` allows and its shape accepts. The next rule takes the rest. [How blocks are matched to a schema][sokf:explanation-block-matching] explains how tramlines matches each block to a rule.

## Troubleshooting

- `TLMD_39`: the body contains a block that the section doesn't declare. See [Errors: blocks][sokf:errors-blocks].
- `TLMD_152`: the body lacks a block that the section declares. See [TLMD_152][sokf:errors-blocks#tlmd_152].
- `TLMD_69`: the body contains a block of another kind where the schema expects a declared block. See [TLMD_69][sokf:errors-blocks#tlmd_69].
- `TLMD_146`, `TLMD_160`: a block has fewer units than its `min`, or more than its `max`. See [TLMD_146][sokf:errors-blocks#tlmd_146] and [TLMD_160][sokf:errors-blocks#tlmd_160].
- `TLMD_147`, `TLMD_161`: a block has fewer units than its `sequence` has entries, or more. See [TLMD_147][sokf:errors-blocks#tlmd_147] and [TLMD_161][sokf:errors-blocks#tlmd_161].
- `TLMD_162`: an item contains a nested list or code block that its rule doesn't declare under `blocks`. See [TLMD_162][sokf:errors-blocks#tlmd_162].
- `TLMD_42`: a block declares a key that doesn't apply to its kind. See [TLMD_42][sokf:errors-block-kinds#tlmd_42].
- `TLMD_153`: a block lacks a key that its kind requires. See [TLMD_153][sokf:errors-block-kinds#tlmd_153].
- `TLMD_47`: a block declares `rules` beside `each` or `sequence`. Keep one. See [TLMD_47][sokf:errors-block-kinds#tlmd_47].

## See also

- [Schema keys: `block`][sokf:reference-schema-keys#block]
- [Schema keys: `unit`][sokf:reference-schema-keys#unit]
- [Errors: kinds of block][sokf:errors-block-kinds]
- [How blocks are matched to a schema][sokf:explanation-block-matching]
- [Select and count units and lines][sokf:how-to-select-and-count-lines]

<!-- sokf:links -->

[sokf:errors-block-kinds]: /docs/reference/errors-block-kinds.md
[sokf:errors-block-kinds#tlmd_153]: /docs/reference/errors-block-kinds.md#tlmd_153
[sokf:errors-block-kinds#tlmd_42]: /docs/reference/errors-block-kinds.md#tlmd_42
[sokf:errors-block-kinds#tlmd_47]: /docs/reference/errors-block-kinds.md#tlmd_47
[sokf:errors-blocks]: /docs/reference/errors-blocks.md
[sokf:errors-blocks#tlmd_146]: /docs/reference/errors-blocks.md#tlmd_146
[sokf:errors-blocks#tlmd_147]: /docs/reference/errors-blocks.md#tlmd_147
[sokf:errors-blocks#tlmd_152]: /docs/reference/errors-blocks.md#tlmd_152
[sokf:errors-blocks#tlmd_160]: /docs/reference/errors-blocks.md#tlmd_160
[sokf:errors-blocks#tlmd_161]: /docs/reference/errors-blocks.md#tlmd_161
[sokf:errors-blocks#tlmd_162]: /docs/reference/errors-blocks.md#tlmd_162
[sokf:errors-blocks#tlmd_69]: /docs/reference/errors-blocks.md#tlmd_69
[sokf:explanation-block-matching]: /docs/explanation/block-matching.md
[sokf:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:how-to-select-and-count-lines]: /docs/how-to/select-and-count-lines.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-schema-keys#block]: /docs/reference/schema-keys.md#block
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
