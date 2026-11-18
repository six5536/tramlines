---
type: ErrorReference
id: errors-limits
title: "Errors: limits"
description: "The errors that tramlines reports for the form of a limit in a schema, and how to fix each one."
order: 13
---

# Errors: limits

These errors come from the form of a `max` in a schema, which section 3 of the core specification defines. [Schema types and keys][sokf:errors-schema-types-keys-lifecycle] lists the error for a `min` above the count of a `max`, and [Blocks][sokf:errors-blocks], [Units][sokf:errors-units], and [Frontmatter][sokf:errors-frontmatter] list the errors of a document that holds more than a hard limit allows.

## TL_137

A `max` is a number, where tramlines reads a map of a limit and its `when-at-limit` text ([TL_137][sokf:tramlines:tramlines-spec#tl_137]).

### Message

```text
<file>:<line>: error[TL_137]: `max` is a map of a limit and its `when-at-limit` text, not a number: write `max: { hard: <n>, when-at-limit: "…" }` to keep the check, or `soft` for a limit that is never checked
```

### Cause

Every `max`, of a block, of a rule under `rules` or `lines`, and of a frontmatter list, is a map. It holds one of `soft` and `hard`, with the count, and `when-at-limit`, the text that tells the writer what to do with the content that doesn't fit within the limit. A bare number tells the writer how many units fit, but not where the rest goes, so the writer squeezes the content into the count.

### Fix

Replace the number with the map that the message shows. Write `hard` where the structure needs the count, such as one list in a section, and the engine checks it. Write `soft` for the usual length, which the engine never checks. In `when-at-limit`, tell the writer, in the language of the schema, what to do with more.

### Example

This schema limits the summary with a bare number:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        max: 1
        example: The kettle needs descaling.
```

**`note.md`**

```markdown
# Note

The kettle needs descaling.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:7: error[TL_137]: `max` is a map of a limit and its `when-at-limit` text, not a number: write `max: { hard: 1, when-at-limit: "…" }` to keep the check, or `soft` for a limit that is never checked
1 error, 0 warnings
[exit 1]
```

Write the usual length as a soft limit, with the text for the writer:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        max:
          soft: 1
          when-at-limit: 'Put the details in the list that follows.'
        example: The kettle needs descaling.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TL_138

The `when-at-limit` of a `max` is empty ([TL_138][sokf:tramlines:tramlines-spec#tl_138]).

### Message

```text
<file>:<line>: error[TL_138]: `when-at-limit` of `max` is empty: it holds the text that tells the writer what to do at the limit
```

### Cause

The scaffold writes `when-at-limit` into the guidance line of the part, a host shows it with the part's guidance, and an error on a hard limit quotes it. An empty text tells the writer nothing about the content that doesn't fit within the limit.

### Fix

Write the text that tells the writer what to do with more, such as where the rest of the content goes.

### Example

This schema gives its limit an empty text:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        max:
          hard: 3
          when-at-limit: ""
        example: Descale the kettle.
```

**`note.md`**

```markdown
# Note

- Descale the kettle.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:9: error[TL_138]: `when-at-limit` of `max` is empty: it holds the text that tells the writer what to do at the limit
1 error, 0 warnings
[exit 1]
```

Write the text:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        max:
          hard: 3
          when-at-limit: 'Put further tasks in the next note.'
        example: Descale the kettle.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-blocks]: /docs/reference/errors-blocks.md
[sokf:errors-frontmatter]: /docs/reference/errors-frontmatter.md
[sokf:errors-schema-types-keys-lifecycle]: /docs/reference/errors-schema-types-keys-lifecycle.md
[sokf:errors-units]: /docs/reference/errors-units.md
[sokf:tramlines:tramlines-spec#tl_137]: /knowledge/spec/tramlines-spec.md#tl_137
[sokf:tramlines:tramlines-spec#tl_138]: /knowledge/spec/tramlines-spec.md#tl_138
