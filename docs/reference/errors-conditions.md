---
type: ErrorReference
id: errors-conditions
title: 'Errors: conditions'
description: The errors that tramlines reports for the condition of a rule, in `only` or in `optional`, and how to fix each one.
order: 14
---

# Errors: conditions

These errors come from the condition of a rule: the map of lifecycle states, or the list of such maps, that `only` and `optional` take. Section 3 of the core specification defines them.

## TL_20

The condition of a rule, in `only` or in `optional`, names a lifecycle dimension or a state that the schema doesn't declare ([TL_20][sokf:tramlines:tramlines-spec#tl_20]).

### Message

```text
<file>:<line>: error[TL_20]: `<key>` names a dimension the lifecycle does not declare: `<dimension>`
<file>:<line>: error[TL_20]: `<key>` names `<state>`, which is no state of `<dimension>`
```

### Cause

The key `only` restricts a rule to some states of a document, and the key `optional` of a section or a block can make the part optional in some states. Each key of a map of the condition is the `key` of a dimension under `lifecycle`, and each value is one of the `states` of that dimension.

### Fix

Change the schema. Correct the dimension or the state in the condition, or add it to the `lifecycle`.

### Example

This rule names a state `drafted`, which the lifecycle doesn't declare:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [drafted] }
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note

## Open questions
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TL_20]: `only` names `drafted`, which is no state of `status`
1 error, 0 warnings
[exit 1]
```

Name the state as the lifecycle does:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_111

A rule declares a condition, in `only` or in `optional`, and the schema declares no lifecycle ([TL_111][sokf:tramlines:tramlines-spec#tl_111]).

### Message

```text
<file>:<line>: error[TL_111]: `<key>` without a lifecycle
```

### Cause

A condition restricts a rule to some states of a document, and the `lifecycle` of a schema declares those states. A schema without a `lifecycle` declares no states for a condition to name.

### Fix

Change the schema. Declare the dimension that the condition names under `lifecycle`, or remove the condition from the rule.

### Example

This schema gates a section on `status`, but declares no lifecycle:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note

## Open questions
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TL_111]: `only` without a lifecycle
1 error, 0 warnings
[exit 1]
```

Declare the lifecycle:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: { status: [draft] }
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_136

A condition of a rule is an empty list, or one of its maps lists no state for a dimension that the lifecycle declares ([TL_136][sokf:tramlines:tramlines-spec#tl_136]).

### Message

```text
<file>:<line>: error[TL_136]: `<key>` is an empty list; give it a map of states, or a list of maps
<file>:<line>: error[TL_136]: `<key>` lists no state for `<dimension>`; list at least one
```

### Cause

A condition is a map of lifecycle dimensions to states, or a list of such maps, and a list matches when any of its maps matches. An empty list has no map to match, so it states no condition. A map that lists no state for a dimension matches no document, because the state of a document is never in an empty list.

### Fix

Change the schema. Write the maps of the condition in the list, list the states of each dimension in its map, or remove the condition from the rule.

### Example

This rule gives `only` an empty list:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: []
    blocks: []
```

**`note.md`**

```markdown
---
status: draft
---

# Note

## Open questions
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TL_136]: `only` is an empty list; give it a map of states, or a list of maps
1 error, 0 warnings
[exit 1]
```

Write the map of the states in which the rule applies:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - key: status
    states: [draft, done]
sections:
  - heading: Note
    level: 1
    blocks: []
  - heading: Open questions
    level: 2
    only: [{ status: [draft] }]
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-spec#tl_111]: /knowledge/spec/tramlines-spec.md#tl_111
[sokf:tramlines:tramlines-spec#tl_136]: /knowledge/spec/tramlines-spec.md#tl_136
[sokf:tramlines:tramlines-spec#tl_20]: /knowledge/spec/tramlines-spec.md#tl_20
