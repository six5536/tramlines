---
type: Explanation
id: explanation-frontmatter-authority
title: Frontmatter as the authority over the body
description: Why the frontmatter of a document is the authority over the text of its body, how a schema binds body text to frontmatter values, and how the lifecycle states in the frontmatter determine which rules apply.
order: 30
---

# Frontmatter as the authority over the body

When a heading and a frontmatter value disagree, the frontmatter is right. A schema can bind a heading or a paragraph to a frontmatter value. The check then requires the body to agree with the frontmatter, never the other way round. The frontmatter also holds the lifecycle state of a document, and that state decides which rules apply to the document.

## Why the frontmatter is the authority

Frontmatter is a YAML map. Each value has a name, and a program reads it without parsing prose. Tools that list, index, or move documents read the frontmatter, so its copy of a fact is the one that those tools see. When the same fact appears twice, once in the frontmatter and once in the body, one copy must be the authority. Otherwise the two drift apart, and nobody knows which to trust.

With the frontmatter as the authority, every disagreement has one fix: change the body to match. A program that repairs documents, such as a host that embeds tramlines, can make that fix itself without guessing. The same authority applies to a new document. tramlines derives its body from its frontmatter, so the two start out equal.

## Binding body text with `from`

A schema binds a part of the body to a frontmatter key with `{ from: <key> }`:

- `heading: { from: title }` on a section requires the heading to equal the value of `title`.
- `text: { from: description }` on a unit rule requires a paragraph, an item, or a line to equal the value of `description`.

In this schema, the heading of a recipe is its `title`, and its first paragraph is its `description`:

**`recipe.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    example: Pancakes
  description:
    example: Thin pancakes for four people.
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        max: 1
        each: { text: { from: description } }
```

The body of this document doesn't match its frontmatter:

**`pancakes.md`**

```markdown
---
title: Pancakes
description: Thin pancakes for four people.
---

# Crepes

Thin pancakes for two people.
```

```console
$ tramlines validate --schema recipe.yaml pancakes.md
pancakes.md:6: error[TLMD_165]: heading `Crepes` must equal the frontmatter `title`: `Pancakes` (schema recipe, section "<title>")
pancakes.md:8: error[TLMD_165]: `Thin pancakes for two people.` must equal the frontmatter `description`: `Thin pancakes for four people.` (schema recipe, section "<title>" block 1)
2 errors, 0 warnings
[exit 1]
```

Each error names the frontmatter key and its value. The value is the text to write. `tramlines validate` reports the disagreement and leaves the file as it is. With `--fix`, it rewrites the heading and the paragraph to match the frontmatter values, as a host that applies the engine's repairs does.

A `{ from: … }` binding is the only way to tie body text to frontmatter. A literal `text`, such as `text: RESPONSIBILITIES`, sets a label that's the same in every document.

## Lifecycle states live in the frontmatter

A schema can declare a lifecycle. A lifecycle has one or more dimensions, and each dimension is a list of states:

```yaml
lifecycle:
  - { key: state, states: [open, closed] }
```

Each dimension is a frontmatter key that every document must contain, with one of its states as the value. You don't declare it again under `frontmatter`. The state of a document is the value of each of these keys. A rule that sets `only` applies only in the states that it lists. The frontmatter therefore decides which sections and blocks the body must contain, as well as the text of a heading.

This schema requires an `Outcome` section when a meeting note is closed:

**`meeting.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: state, states: [open, closed] }
frontmatter:
  title:
    example: Release planning
sections:
  - heading: { from: title }
    level: 1
    blocks: []
  - heading: Agenda
    level: 2
    blocks:
      - kind: bullet-list
        example: Pick the release date.
  - heading: Outcome
    level: 2
    only: { state: closed }
    blocks:
      - kind: prose
        example: The release ships on 3 March.
```

The same body passes while the note is open and fails when it's closed:

**`planning.md`**

```markdown
---
title: Release planning
state: closed
---

# Release planning

## Agenda

- Pick the release date.
```

```console
$ tramlines validate --schema meeting.yaml planning.md
planning.md: error[TLMD_18]: section `## Outcome` is missing (schema meeting, section "Outcome")
1 error, 0 warnings
[exit 1]
```

With `state: open`, the `Outcome` section doesn't apply:

**`planning.md`**

```markdown
---
title: Release planning
state: open
---

# Release planning

## Agenda

- Pick the release date.
```

```console
$ tramlines validate --schema meeting.yaml planning.md
0 errors, 0 warnings
```

To close a note, you change one frontmatter value. The check then reports what the body of the closed note must contain. An `id` template can include the state too, such as `note-${state}-${:seq}`. The check then requires the id to agree with the `state` key. [Apply a rule in some states only][sokf:how-to-gate-rules-by-state] shows how to write these rules.

## A document with no frontmatter

A document doesn't have to open with frontmatter. When it doesn't, the check reads it as if its frontmatter were an empty map. The rules stay the same: every key that the schema requires is missing, and every lifecycle key is missing.

**`notes.md`**

```markdown
# Release planning

## Agenda

- Pick the release date.
```

```console
$ tramlines validate --schema meeting.yaml notes.md
notes.md:1: error[TL_54]: frontmatter key `title` is missing (schema meeting, key `title`)
notes.md:1: error[TL_62]: lifecycle key `state` is missing (schema meeting)
2 errors, 0 warnings
[exit 1]
```

Each missing key causes one error: the key `title` and the lifecycle key `state`. When you add the frontmatter, both errors go away.

A schema that declares no frontmatter key and no lifecycle accepts a document with no frontmatter, because an empty map breaks none of its rules.

## See also

- [TL_94][sokf:tramlines:tramlines-spec#tl_94]: a document with no frontmatter is checked as if its frontmatter were an empty map.
- [TL_61][sokf:tramlines:tramlines-spec#tl_61], [TL_62][sokf:tramlines:tramlines-spec#tl_62], and [TL_108][sokf:tramlines:tramlines-spec#tl_108]: the state of a document is read from its frontmatter, and a missing or unknown state is reported.
- [TL_19][sokf:tramlines:tramlines-spec#tl_19]: a rule with `only` applies only in the listed states.
- [TL_59][sokf:tramlines:tramlines-spec#tl_59]: an id that includes a state must agree with the state key.
- [TLMD_136][sokf:tramlines:tramlines-md-spec#tlmd_136] and [TLMD_165][sokf:tramlines:tramlines-md-spec#tlmd_165]: a heading or unit bound to a frontmatter value is reported and repaired to that value.
- [Constrain frontmatter and ids][sokf:how-to-constrain-frontmatter]
- [Apply a rule in some states only][sokf:how-to-gate-rules-by-state]

<!-- sokf:links -->

[sokf:how-to-constrain-frontmatter]: /docs/how-to/constrain-frontmatter.md
[sokf:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:tramlines:tramlines-md-spec#tlmd_136]: /knowledge/spec/tramlines-md-spec.md#tlmd_136
[sokf:tramlines:tramlines-md-spec#tlmd_165]: /knowledge/spec/tramlines-md-spec.md#tlmd_165
[sokf:tramlines:tramlines-spec#tl_108]: /knowledge/spec/tramlines-spec.md#tl_108
[sokf:tramlines:tramlines-spec#tl_19]: /knowledge/spec/tramlines-spec.md#tl_19
[sokf:tramlines:tramlines-spec#tl_59]: /knowledge/spec/tramlines-spec.md#tl_59
[sokf:tramlines:tramlines-spec#tl_61]: /knowledge/spec/tramlines-spec.md#tl_61
[sokf:tramlines:tramlines-spec#tl_62]: /knowledge/spec/tramlines-spec.md#tl_62
[sokf:tramlines:tramlines-spec#tl_94]: /knowledge/spec/tramlines-spec.md#tl_94
