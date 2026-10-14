---
type: HowTo
id: how-to-repair-and-start-documents
title: Repair a document and start a new one
description: Start a document from the scaffold of its schema with `tramlines new`, and let `tramlines validate --fix` rewrite the headings and anchors that the schema derives.
order: 115
---

# Repair a document and start a new one

When a schema derives part of a document, such as a heading that repeats the title or the anchor of a key, let the command write that part. At the end, you have a document that started from its scaffold. You write only its content, and `tramlines validate --fix` keeps the derived parts in step with it.

## Before you begin

- A schema with frontmatter and sections, as in [Your first schema][sokf:tutorial-first-schema].
- An understanding of keys and their anchors, from [Capture keys and anchor them][sokf:how-to-capture-keys].

## Steps

1. Write the schema. This one describes a decision: an id built from a slug, a title that the level-1 heading repeats, and a list of consequences, each with a key.

**`decision.yaml`**

```yaml
line-limit: 0
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
        max: 1
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

2. Print the scaffold of a new document. The option `--slug` fills the `slug` hole of the id. The title takes the words of the slug unless you set `--title`. The heading repeats the title, as the schema requires. Save the output as the new document:

```console
$ tramlines new --schema decision.yaml --slug cart-lifetime
---
id: decision-cart-lifetime
title: Cart lifetime
---

<!-- tramlines:guidance The writer replaces each of the following examples with their own text. -->

# Cart lifetime

The cart keeps its items for seven days, and then it empties.

## Consequences

- C-1: A returning customer finds the cart as they left it.
```

3. Write the content. Replace each example with your own text, and delete the `guidance` line. In this document, the writer also changed the title, and added a second consequence:

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
- `TLMD_158`: a `guidance` line is left in the document. Write the part that it guides, and then delete the line. See [TLMD_158][sokf:errors-keys-markers-scaffolds#tlmd_158].

## See also

- [The command][sokf:reference-cli]
- [Apply a rule in some states only][sokf:how-to-gate-rules-by-state]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [Why the frontmatter is the authority][sokf:explanation-frontmatter-authority]

<!-- sokf:links -->

[sokf:errors-keys-markers-scaffolds#tlmd_158]: /docs/reference/errors-keys-markers-scaffolds.md#tlmd_158
[sokf:explanation-frontmatter-authority]: /docs/explanation/frontmatter-authority.md
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
