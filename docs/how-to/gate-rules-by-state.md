---
type: HowTo
id: how-to-gate-rules-by-state
title: Apply a rule in some states only
description: Declare the states that a document moves through, and gate a section, a block, or a frontmatter key to some of them with `only`.
order: 30
---

# Apply a rule in some states only

When a document changes as its work progresses, declare its states in a `lifecycle`. An issue, for example, gains an outcome when it closes. Then gate each rule that applies only in some states with `only`. At the end, one schema describes the document in every state, and the check requires the parts that each state needs.

In this guide, you write a schema for an issue, which is open or closed, and a bug or a feature. Only a bug includes a reproduction, and only a closed issue includes an outcome and a date.

## Before you begin

- A schema with sections and frontmatter, as in [Your first schema][sokf:tutorial-first-schema].

## Steps

1. Declare each dimension of the lifecycle under `lifecycle`, in order, with its frontmatter `key` and its `states`. Each dimension is a required frontmatter key whose value is one of its states. Don't declare the key again under `frontmatter`. A dimension can take `guidance`, a note that explains the states to a writer.

2. Set `only` on each rule that applies only in some states. `only` maps a dimension to a state, or to a list of states. A rule with `only` applies when the state of the document is in the set for every dimension that `only` names. Any rule can take `only`: a section, a block, a frontmatter key, or any other rule of a schema.

**`issue.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: state
    states: [open, closed]
    guidance: Close the issue when its work is merged.
  - { key: kind, states: [bug, feature] }
frontmatter:
  title:
    example: The parser rejects a tab
  closed-on:
    pattern: '^\d{4}-\d{2}-\d{2}$'
    example: '2026-09-30'
    only: { state: closed }
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        example: A list item indented with a tab is read as a code block.
  - heading: Reproduction
    level: 2
    only: { kind: bug }
    blocks:
      - kind: numbered-list
        example: Indent a list item with a tab.
  - heading: Outcome
    level: 2
    only: { state: closed, kind: [bug, feature] }
    blocks:
      - kind: prose
        example: The parser now reads a tab as four spaces.
```

3. Check an open bug. This open bug includes a reproduction but lacks an outcome and a date:

**`tab.md`**

```markdown
---
title: The parser rejects a tab
state: open
kind: bug
---

# The parser rejects a tab

A list item indented with a tab is read as a code block.

## Reproduction

1. Indent a list item with a tab.
2. Run the check.
```

```console
$ tramlines validate --schema issue.yaml tab.md
0 errors, 0 warnings
```

4. Change the state, and check again. After the bug is closed, the check requires the outcome and the date:

**`tab.md`**

```markdown
---
title: The parser rejects a tab
state: closed
kind: bug
---

# The parser rejects a tab

A list item indented with a tab is read as a code block.

## Reproduction

1. Indent a list item with a tab.
2. Run the check.
```

```console
$ tramlines validate --schema issue.yaml tab.md
tab.md: error[TLMD_18]: section `## Outcome` is missing (schema issue, section "Outcome")
tab.md:2: error[TL_54]: frontmatter key `closed-on` is missing (schema issue, key `closed-on`)
2 errors, 0 warnings
[exit 1]
```

The check reports a part that the state of the document doesn't allow as undeclared. A feature can't include a reproduction:

**`tabs-as-spaces.md`**

```markdown
---
title: Read a tab as four spaces
state: open
kind: feature
---

# Read a tab as four spaces

A list item indented with a tab is read as a code block.

## Reproduction

1. Indent a list item with a tab.
```

```console
$ tramlines validate --schema issue.yaml tabs-as-spaces.md
tabs-as-spaces.md:11: error[TLMD_19]: heading `## Reproduction` is not declared (schema issue)
1 error, 0 warnings
[exit 1]
```

5. Add real documents of the type under `examples`, each as one `document`. The command checks each example at the states of its own frontmatter, as it checks a document, and never copies it. An example can't contain markers.

**`issue.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: state
    states: [open, closed]
    guidance: Close the issue when its work is merged.
  - { key: kind, states: [bug, feature] }
frontmatter:
  title:
    example: The parser rejects a tab
  closed-on:
    pattern: '^\d{4}-\d{2}-\d{2}$'
    example: '2026-09-30'
    only: { state: closed }
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        example: A list item indented with a tab is read as a code block.
  - heading: Reproduction
    level: 2
    only: { kind: bug }
    blocks:
      - kind: numbered-list
        example: Indent a list item with a tab.
  - heading: Outcome
    level: 2
    only: { state: closed, kind: [bug, feature] }
    blocks:
      - kind: prose
        example: The parser now reads a tab as four spaces.
examples:
  - document: |
      ---
      title: Read a tab as four spaces
      state: closed
      kind: feature
      closed-on: '2026-09-30'
      ---

      # Read a tab as four spaces

      A list item indented with a tab is read as a code block.

      ## Outcome

      The parser now reads a tab as four spaces.
```

**`tab.md`**

```markdown
---
title: The parser rejects a tab
state: closed
kind: bug
closed-on: '2026-09-30'
---

# The parser rejects a tab

A list item indented with a tab is read as a code block.

## Reproduction

1. Indent a list item with a tab.
2. Run the check.

## Outcome

The parser now reads a tab as four spaces.
```

```console
$ tramlines validate --schema issue.yaml tab.md
0 errors, 0 warnings
```

6. Start a new document with `tramlines new`. For each combination of states, the engine derives one scaffold, the document that a new document starts from. The command `tramlines validate` checks each scaffold. The command `tramlines new` prints the scaffold of the states that `--state` names. For each dimension that `--state` doesn't name, it takes the first state. The scaffold contains the parts that those states need, and a `guidance` line before each part that has guidance. Read each `guidance` line, act on it, and then delete it:

```console
$ tramlines new --schema issue.yaml --state state=closed --state kind=feature
---
title: The parser rejects a tab
closed-on: 2026-09-30
# tramlines:guidance Close the issue when its work is merged.
state: closed
kind: feature
---

<!-- tramlines:guidance The writer replaces each of the following examples with their own text. -->

# The parser rejects a tab

A list item indented with a tab is read as a code block.

## Outcome

The parser now reads a tab as four spaces.
```

## Troubleshooting

- `TL_62`: a lifecycle key is missing. Add the key. See [TL_62][sokf:errors-frontmatter#tl_62].
- `TL_108`: the value of a lifecycle key isn't one of its states. Set the key to one of the states that the message lists. See [TL_108][sokf:errors-frontmatter#tl_108].
- `TL_20`: `only` names a dimension or a state that the lifecycle doesn't declare. Fix the name. See [TL_20][sokf:errors-lifecycle-and-ids#tl_20].
- `TL_111`: a rule declares `only`, and the schema has no lifecycle. Declare the lifecycle. See [TL_111][sokf:errors-lifecycle-and-ids#tl_111].
- `TL_95`: the lifecycle declares one dimension twice. Remove one. See [TL_95][sokf:errors-lifecycle-and-ids#tl_95].
- `TL_112`: `frontmatter` declares the key of a lifecycle dimension. Remove it from `frontmatter`. See [TL_112][sokf:errors-lifecycle-and-ids#tl_112].
- `TL_69`: an entry of `examples` fails the schema at its own states. Fix the example. See [TL_69][sokf:errors-examples#tl_69].

## See also

- [Schema keys: `lifecycle-dimension`][sokf:reference-schema-keys#lifecycle-dimension]
- [Schema keys: `example`][sokf:reference-schema-keys#example]
- [Constrain frontmatter and ids][sokf:how-to-constrain-frontmatter], to put a state in the id of a document
- [Strict by default][sokf:explanation-strict-by-default]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [Section 6 of the core specification][sokf:tramlines:tramlines-spec#6-frontmatter-and-lifecycle]

<!-- sokf:links -->

[sokf:errors-examples#tl_69]: /docs/reference/errors-examples.md#tl_69
[sokf:errors-frontmatter#tl_108]: /docs/reference/errors-frontmatter.md#tl_108
[sokf:errors-frontmatter#tl_62]: /docs/reference/errors-frontmatter.md#tl_62
[sokf:errors-lifecycle-and-ids#tl_111]: /docs/reference/errors-lifecycle-and-ids.md#tl_111
[sokf:errors-lifecycle-and-ids#tl_112]: /docs/reference/errors-lifecycle-and-ids.md#tl_112
[sokf:errors-lifecycle-and-ids#tl_20]: /docs/reference/errors-lifecycle-and-ids.md#tl_20
[sokf:errors-lifecycle-and-ids#tl_95]: /docs/reference/errors-lifecycle-and-ids.md#tl_95
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-constrain-frontmatter]: /docs/how-to/constrain-frontmatter.md
[sokf:reference-schema-keys#example]: /docs/reference/schema-keys.md#example
[sokf:reference-schema-keys#lifecycle-dimension]: /docs/reference/schema-keys.md#lifecycle-dimension
[sokf:tramlines:tramlines-spec#6-frontmatter-and-lifecycle]: /knowledge/spec/tramlines-spec.md#6-frontmatter-and-lifecycle
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
