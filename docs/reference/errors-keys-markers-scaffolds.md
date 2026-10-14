---
type: ErrorReference
id: errors-keys-markers-scaffolds
title: 'Errors: markers'
description: The errors that tramlines reports for the markers of a scaffold that a document keeps.
order: 70
---

# Errors: markers

These errors come from a marker that a scaffold leaves in a document. Section 3 of the Markdown specification defines these errors. [A separate page][sokf:errors-scaffolds] lists the errors for the scaffolds that a schema derives.

## TLMD_10

A document still contains an `only` marker, which belongs only in a scaffold ([TLMD_10][sokf:tramlines:tramlines-md-spec#tlmd_10]).

### Message

```text
<file>:<line>: error[TLMD_10]: a `tramlines:only` marker belongs in a scaffold, not a governed document
```

### Cause

A scaffold marks each part that applies only in some lifecycle states with a `<!-- tramlines:only … -->` line, or with a `# tramlines:only …` comment after a frontmatter key. A document written from the scaffold still contains one of these markers outside a code block.

### Fix

Delete the `only` marker from the document: the whole line for an HTML comment, or only the `# tramlines:only …` comment after a frontmatter key. The schema already states when the part applies.

### Example

In this schema, `Outcome` applies only to a closed bug:

**`bug.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: state, states: [open, closed] }
sections:
  - heading: Bug
    level: 1
    blocks: []
  - heading: Outcome
    level: 1
    only: { state: closed }
    blocks: []
```

This document still contains the `only` line of its scaffold:

**`bug.md`**

```markdown
---
state: closed
---

# Bug

<!-- tramlines:only state=closed -->

# Outcome
```

```console
$ tramlines validate --schema bug.yaml bug.md
bug.md:7: error[TLMD_10]: a `tramlines:only` marker belongs in a scaffold, not a governed document
1 error, 0 warnings
[exit 1]
```

Delete the line:

**`bug.md`**

```markdown
---
state: closed
---

# Bug

# Outcome
```

```console
$ tramlines validate --schema bug.yaml bug.md
0 errors, 0 warnings
```

## TLMD_158

A document still contains a `guidance` line of its scaffold ([TLMD_158][sokf:tramlines:tramlines-md-spec#tlmd_158]).

### Message

```text
<file>:<line>: error[TLMD_158]: a `tramlines:guidance` line is left here: write the part it guides, then delete the line
```

### Cause

A scaffold explains each part with a `<!-- tramlines:guidance … -->` line before it, or a `# tramlines:guidance …` line before a frontmatter key. A document written from the scaffold still contains one of these lines outside a code block.

### Fix

In the document, write the part that the `guidance` line describes, then delete the line.

### Example

This schema requires a paragraph:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
```

This document still contains the `guidance` line of its scaffold:

**`note.md`**

```markdown
# Note

<!-- tramlines:guidance Say what to buy. -->

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_158]: a `tramlines:guidance` line is left here: write the part it guides, then delete the line
1 error, 0 warnings
[exit 1]
```

Delete the line:

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_11

A document still contains an `optional` region of its scaffold ([TLMD_11][sokf:tramlines:tramlines-md-spec#tlmd_11]).

### Message

```text
<file>:<line>: warning[TLMD_11]: a `tramlines:optional` region is left here: keep what it holds and drop both markers, or drop the region
```

### Cause

A scaffold puts each part that the schema doesn't require between `<!-- tramlines:optional -->` and `<!-- /tramlines:optional -->`. The document still contains the opening marker. The finding is a warning, so the command still exits with 0.

### Fix

In the document, decide whether to keep the part. To keep it, delete both markers. To leave it out, delete the whole region, markers included.

### Example

This example uses the `note.yaml` of [TLMD_158](#tlmd_158). This document still contains an `optional` region:

**`note.md`**

```markdown
# Note

<!-- tramlines:optional -->

Buy milk.

<!-- /tramlines:optional -->
```

```console
$ tramlines validate --warnings --schema note.yaml note.md
note.md:3: warning[TLMD_11]: a `tramlines:optional` region is left here: keep what it holds and drop both markers, or drop the region
0 errors, 1 warning
```

Keep the paragraph and delete both markers:

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_12

An `optional` marker has no partner: a region opens and nothing closes it, or it closes and nothing opened it ([TLMD_12][sokf:tramlines:tramlines-md-spec#tlmd_12]).

### Message

```text
<file>:<line>: error[TLMD_12]: a `tramlines:optional` region opens here and nothing closes it
<file>:<line>: error[TLMD_12]: a `tramlines:optional` region closes here and nothing opened it
```

### Cause

The document contains a `<!-- tramlines:optional -->` line with no `<!-- /tramlines:optional -->` line after it, or a closing line with no opening line before it. Regions don't nest. When a second opening line follows an opening line before any closing line, the first opening line has no partner either. A marker with no partner forms no region, so the command reports no [TLMD_11](#tlmd_11) for it.

### Fix

In the document, delete the marker that has no partner, along with any other `optional` marker that's left.

### Example

This example uses the `note.yaml` of [TLMD_158](#tlmd_158). This document contains the closing marker of a region but not its opening marker:

**`note.md`**

```markdown
# Note

Buy milk.

<!-- /tramlines:optional -->
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_12]: a `tramlines:optional` region closes here and nothing opened it
1 error, 0 warnings
[exit 1]
```

Delete the marker:

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-scaffolds]: /docs/reference/errors-scaffolds.md
[sokf:tramlines:tramlines-md-spec#tlmd_10]: /knowledge/spec/tramlines-md-spec.md#tlmd_10
[sokf:tramlines:tramlines-md-spec#tlmd_11]: /knowledge/spec/tramlines-md-spec.md#tlmd_11
[sokf:tramlines:tramlines-md-spec#tlmd_12]: /knowledge/spec/tramlines-md-spec.md#tlmd_12
[sokf:tramlines:tramlines-md-spec#tlmd_158]: /knowledge/spec/tramlines-md-spec.md#tlmd_158
