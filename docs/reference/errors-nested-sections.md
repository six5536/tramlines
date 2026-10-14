---
type: ErrorReference
id: errors-nested-sections
title: 'Errors: sections inside sections'
description: The errors that tramlines reports for a section that declares a parent section, and how to fix each one.
order: 45
---

# Errors: sections inside sections

These errors come from a section that declares `parent`, and from its place in the schema and in the document. Section 4 of the Markdown specification defines them.

## TLMD_31

A heading of a section that declares `parent` doesn't lie inside a heading of its parent ([TLMD_31][sokf:tramlines:tramlines-md-spec#tlmd_31]).

### Message

```text
<file>:<line>: error[TLMD_31]: heading `<heading>` lies outside every `<parent heading>` section; it belongs inside one (schema <id>, <place of the rule>)
```

### Cause

The section names its parent with `parent`, and the nearest preceding heading at the parent's level or a smaller level isn't a heading of the parent.

### Fix

Move the heading in the document under a heading of its parent, or add a heading of the parent before it.

### Example

In this schema, `Lessons` lies inside `Outcome`:

**`bug.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Bug
    level: 1
    blocks: []
  - heading: Outcome
    level: 1
    optional: true
    rule-id: outcome
    blocks: []
  - heading: Lessons
    level: 2
    parent: outcome
    optional: true
    blocks: []
```

This document places `Lessons` under `Bug`:

**`bug.md`**

```markdown
# Bug

## Lessons
```

```console
$ tramlines validate --schema bug.yaml bug.md
bug.md:3: error[TLMD_31]: heading `## Lessons` lies outside every `# Outcome` section; it belongs inside one (schema bug, section "Lessons")
1 error, 0 warnings
[exit 1]
```

Add the `Outcome` heading before `Lessons`:

**`bug.md`**

```markdown
# Bug

# Outcome

## Lessons
```

```console
$ tramlines validate --schema bug.yaml bug.md
0 errors, 0 warnings
```

## TLMD_166

A heading matches a section whose `parent` doesn't apply in the lifecycle state of the document ([TLMD_166][sokf:tramlines:tramlines-md-spec#tlmd_166]).

### Message

```text
<file>:<line>: error[TLMD_166]: heading `<heading>` lies inside `<rule-id>`, which does not apply in this document's states (schema <id>, <place of the rule>)
```

### Cause

A section declares `parent`, and the parent section sets `only` to states that exclude the document's state. The parent section doesn't apply, so the document can't contain a heading for the child section inside the parent.

### Fix

Remove the heading from the document, or move the document to a state in which the parent applies and add the parent's heading. If the section belongs in every state, change the `only` of its parent in the schema.

### Example

In this schema, `Outcome` applies only to a closed bug, and `Lessons` lies inside it:

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
    rule-id: outcome
    blocks: []
  - heading: Lessons
    level: 2
    parent: outcome
    optional: true
    blocks: []
```

This bug is open and contains a `Lessons` heading:

**`bug.md`**

```markdown
---
state: open
---

# Bug

## Lessons
```

```console
$ tramlines validate --schema bug.yaml bug.md
bug.md:7: error[TLMD_166]: heading `## Lessons` lies inside `outcome`, which does not apply in this document's states (schema bug, section "Lessons")
1 error, 0 warnings
[exit 1]
```

Close the bug and add its outcome:

**`bug.md`**

```markdown
---
state: closed
---

# Bug

# Outcome

## Lessons
```

```console
$ tramlines validate --schema bug.yaml bug.md
0 errors, 0 warnings
```

## TLMD_34

The `parent` of a section names no section of the schema ([TLMD_34][sokf:tramlines:tramlines-md-spec#tlmd_34]).

### Message

```text
<file>:<line>: error[TLMD_34]: `parent` names no rule `<rule-id>`
<file>:<line>: error[TLMD_34]: `parent` names `<rule-id>`, which is not a section
```

### Cause

The `parent` of a section in the schema names a `rule-id` that no rule declares, or the `rule-id` of a block instead of a section.

### Fix

Set `parent` to the `rule-id` of the section that contains this one. If that section has no `rule-id`, give it one.

### Example

In this schema, `parent` names `steps`, but the `rule-id` of the section is `step`:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 2
    parent: steps
    blocks: []
```

**`guide.md`**

```markdown
# Step 1

## Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.yaml:8: error[TLMD_34]: `parent` names no rule `steps`
1 error, 0 warnings
[exit 1]
```

Name the `rule-id` of the parent:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 2
    parent: step
    blocks: []
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

## TLMD_156

The `parent` of a section names a section whose level isn't smaller than its own ([TLMD_156][sokf:tramlines:tramlines-md-spec#tlmd_156]).

### Message

```text
<file>:<line>: error[TLMD_156]: `parent` `<rule-id>` is level <parent level>; a section lies inside one of a smaller level, and this one is level <level>
```

### Cause

A section lies inside its parent only when the parent's heading has a smaller level, as `##` lies inside `#`. The schema gives the section a `level` equal to or smaller than the `level` of the section that its `parent` names.

### Fix

Give the section a `level` greater than the level of its parent, or set `parent` to a section of a smaller level.

### Example

In this schema, `Check` and its parent `Step` are both level 1:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 1
    parent: step
    blocks: []
```

**`guide.md`**

```markdown
# Step 1

## Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.yaml:8: error[TLMD_156]: `parent` `step` is level 1; a section lies inside one of a smaller level, and this one is level 1
1 error, 0 warnings
[exit 1]
```

Make `Check` level 2:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 2
    parent: step
    blocks: []
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

## TLMD_157

The schema doesn't list a section directly after its parent, with only the parent's other subsections between them ([TLMD_157][sokf:tramlines:tramlines-md-spec#tlmd_157]).

### Message

```text
<file>:<line>: error[TLMD_157]: section `<section>` is not listed after its `parent` `<rule-id>` with only its subsections between
```

### Cause

The schema lists the sections in document order, so a section inside a parent comes after the parent. The schema lists the section before its parent, or lists another section at the parent's level or a smaller level between them.

### Fix

Move the section in the schema so that it follows its parent, with only the parent's other subsections between them.

### Example

This schema lists `Check` before its parent:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Check
    level: 2
    parent: step
    blocks: []
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
```

**`guide.md`**

```markdown
# Step 1

## Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.yaml:3: error[TLMD_157]: section `Check` is not listed after its `parent` `step` with only its subsections between
1 error, 0 warnings
[exit 1]
```

List `Check` after `Step`:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "Step ${:int}", example: Step 1 }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: Check
    level: 2
    parent: step
    blocks: []
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

## TLMD_35

The heading of a section that sets `number-from-parent` doesn't extend the number of its parent's heading ([TLMD_35][sokf:tramlines:tramlines-md-spec#tlmd_35]).

### Message

```text
<file>:<line>: error[TLMD_35]: heading `<heading>` must be numbered `<number>.<n>`, under `<parent heading>` (schema <id>, <place of the rule>)
```

### Cause

The section sets `number-from-parent: true`, so its heading must begin with the number of its parent's heading, a dot, and a number. The heading in the document begins with another number, or with none.

### Fix

Number the heading in the document as the message shows: the number of the parent heading, a dot, and a number.

### Example

In this schema, each `Check` extends the number of its step:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "${:int} Step", example: 1 Step }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: { template: "${:int}.${:int} Check", example: 1.1 Check }
    level: 2
    parent: step
    number-from-parent: true
    blocks: []
```

This document numbers the second `Check` from the first step:

**`guide.md`**

```markdown
# 1 Step

## 1.1 Check

# 2 Step

## 1.1 Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.md:7: error[TLMD_35]: heading `## 1.1 Check` must be numbered `2.<n>`, under `# 2 Step` (schema guide, section "${:int}.${:int} Check")
1 error, 0 warnings
[exit 1]
```

Number the second `Check` from its own step:

**`guide.md`**

```markdown
# 1 Step

## 1.1 Check

# 2 Step

## 2.1 Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

## TLMD_36

The heading of the parent of a section that sets `number-from-parent` has no number ([TLMD_36][sokf:tramlines:tramlines-md-spec#tlmd_36]).

### Message

```text
<file>:<line>: error[TLMD_36]: heading `<parent heading>` carries no number for `<heading>` to extend (schema <id>, <place of the rule>)
```

### Cause

The section sets `number-from-parent: true`, and the heading of its parent in the document doesn't begin with a number. The error reports the line of the parent heading.

### Fix

Begin the parent heading in the document with a number, such as `1` or `2.3`. If the parent heading carries no number in documents of this kind, remove `number-from-parent` from the section in the schema.

### Example

In this schema, the heading of a step can begin with any text:

**`guide.yaml`**

```yaml
line-limit: 0
sections:
  - heading: { template: "${} Step", example: 1 Step }
    level: 1
    repeat: true
    rule-id: step
    blocks: []
  - heading: { template: "${:int}.${:int} Check", example: 1.1 Check }
    level: 2
    parent: step
    number-from-parent: true
    blocks: []
```

This document begins the heading of its step with a word instead of a number:

**`guide.md`**

```markdown
# First Step

## 1.1 Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
guide.md:1: error[TLMD_36]: heading `# First Step` carries no number for `## 1.1 Check` to extend (schema guide, section "${:int}.${:int} Check")
1 error, 0 warnings
[exit 1]
```

Number the step:

**`guide.md`**

```markdown
# 1 Step

## 1.1 Check
```

```console
$ tramlines validate --schema guide.yaml guide.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_156]: /knowledge/spec/tramlines-md-spec.md#tlmd_156
[sokf:tramlines:tramlines-md-spec#tlmd_157]: /knowledge/spec/tramlines-md-spec.md#tlmd_157
[sokf:tramlines:tramlines-md-spec#tlmd_166]: /knowledge/spec/tramlines-md-spec.md#tlmd_166
[sokf:tramlines:tramlines-md-spec#tlmd_31]: /knowledge/spec/tramlines-md-spec.md#tlmd_31
[sokf:tramlines:tramlines-md-spec#tlmd_34]: /knowledge/spec/tramlines-md-spec.md#tlmd_34
[sokf:tramlines:tramlines-md-spec#tlmd_35]: /knowledge/spec/tramlines-md-spec.md#tlmd_35
[sokf:tramlines:tramlines-md-spec#tlmd_36]: /knowledge/spec/tramlines-md-spec.md#tlmd_36
