---
type: HowTo
id: how-to-repair-and-start-documents
title: Repair a document and start a new one
description: Start a document with `tramlines new`, which writes what the schema prescribes and prints each part's guidance and example, and let `tramlines validate --fix` rewrite the headings and anchors that the schema derives.
order: 115
---

# Repair a document and start a new one

When a schema derives part of a document, such as a heading that repeats the title or the anchor of a key, let the command write that part. At the end, you have a document that started from what its schema prescribes. You write only its content, and `tramlines validate --fix` keeps the derived parts in step with it.

## Before you begin

- A schema with frontmatter and sections, as in [Your first schema][sokf:tutorial-first-schema].
- An understanding of keys and their anchors, from [Capture keys and anchor them][sokf:how-to-capture-keys].

## Steps

1. Write the schema. This one describes a decision: an id built from a slug, a title that the level-1 heading repeats, and a list of consequences, each with a key. It declares its language, English.

**`decision.yaml`**

```yaml
line-limit: 0
language: en
types:
  consequence-id: 'C-\d+'
frontmatter:
  id:
    template: 'decision-${:slug}'
  title:
    example: Store the cart for a week
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max:
          soft: 1
          when-at-limit: 'Put each consequence in the section that follows.'
        example: The cart keeps its items for seven days, and then it empties.
  - heading: Consequences
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${id:consequence-id}: ${}'
          key: id
          example: 'C-1: A returning customer finds the cart as they left it.'
```

2. Print a new document. The option `--slug` fills the `slug` hole of the id. The title takes the words of the slug unless you set `--title`. The new document holds only what is true of it: the id, the title, the heading that repeats the title, and the heading that the schema fixes. It holds no example, so nothing of an example can stay in it by mistake. After it, on stderr, the command prints the guide of the document: each part, its guidance, and its example. The guidance of the paragraph states its limit in English, before its `when-at-limit`. Save the output on stdout as the new document, as `> cart.md` does:

```console
$ tramlines new --schema decision.yaml --slug cart-lifetime
---
id: decision-cart-lifetime
title: Cart lifetime
---

# Cart lifetime

## Consequences

The guide of the new document: each part, its guidance, and its example.

Section `# <title>`:

  Block 1 (prose):
    Usually up to 1 paragraph. Never leave out or merge content to stay within the count. When at the limit: Put each consequence in the section that follows.
    Example:
      The cart keeps its items for seven days, and then it empties.

Section `## Consequences`:

  Block 1 (bullet-list):
    Example:
      - C-1: A returning customer finds the cart as they left it.
```

3. Write the content of each part in your own words, as its guidance asks and its example shows. Until you do, the check reports each part that the document lacks, by its section and its block. In this document, the writer also changed the title, and added a second consequence:

**`cart.md`**

```markdown
---
id: decision-cart-lifetime
title: Keep the cart for a week
---

# Cart lifetime

A cart keeps its items for seven days after the last change, and then it empties.

## Consequences

- C-1: A returning customer finds the cart as they left it.
- C-2: The store keeps a week of carts for each customer.
```

4. Check the document. The heading no longer repeats the title, and neither key has its anchor:

```console
$ tramlines validate --schema decision.yaml cart.md
cart.md:6: error[TLMD_165]: heading `Cart lifetime` must equal the frontmatter `title`: `Keep the cart for a week` (schema decision, section "<title>")
cart.md:12: error[TLMD_109]: key `C-1` has no anchor; its repair writes `<a id="c-1"></a>` before it
cart.md:13: error[TLMD_109]: key `C-2` has no anchor; its repair writes `<a id="c-2"></a>` before it
3 errors, 0 warnings
[exit 1]
```

5. Repair the document with `--fix`. The command rewrites the heading from the title and writes each anchor, then reports what is left. A second run finds nothing to repair:

```console
$ tramlines validate --schema decision.yaml --fix cart.md
rewrote cart.md
0 errors, 0 warnings
$ tramlines validate --schema decision.yaml --fix cart.md
0 errors, 0 warnings
```

   To repair a document that an editor or a hook pipes in, name it `-` and its format with `--format`, as in `tramlines validate --schema decision.yaml --format md --fix -`. The command prints the repaired document on stdout and the report on stderr.

## Troubleshooting

- `the id template has a seq hole`: the number of a new document depends on the documents around it, which the command doesn't read. Name the whole id with `--id`.
- `is no state of`: a `--state` names a state that its dimension doesn't declare. Use one of the states that the message lists.
- `TLMD_152` or `TLMD_18` on a new document: the document lacks a block or a section that the schema requires. Write it, as the guide that `tramlines new` printed shows.
- `TL_143`: a required frontmatter key has no value. Write its value, as the example of the guide shows. See [TL_143][sokf:errors-frontmatter#tl_143].
- `--fix` reports a paragraph that must read a fixed text, and leaves it: the section holds another error that can move a block to another position, such as an item that breaks the pattern of its list, so the command can't be sure which text the paragraph needs. Correct that error, and run `--fix` again. See [TL_74][sokf:errors-lines#tl_74].
- `TLMD_158`: a `guidance` line is left in the document, as a document started from the scaffold of an older version of the command holds them. Write the part that it guides, and then delete the line. See [TLMD_158][sokf:errors-keys-markers-scaffolds#tlmd_158].

## See also

- [The command][sokf:reference-cli]
- [Apply a rule in some states only][sokf:how-to-gate-rules-by-state]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [Why the frontmatter is the authority][sokf:explanation-frontmatter-authority]

<!-- sokf:links -->

[sokf:errors-frontmatter#tl_143]: /docs/reference/errors-frontmatter.md#tl_143
[sokf:errors-keys-markers-scaffolds#tlmd_158]: /docs/reference/errors-keys-markers-scaffolds.md#tlmd_158
[sokf:errors-lines#tl_74]: /docs/reference/errors-lines.md#tl_74
[sokf:explanation-frontmatter-authority]: /docs/explanation/frontmatter-authority.md
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
