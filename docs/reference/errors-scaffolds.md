---
type: ErrorReference
id: errors-scaffolds
title: 'Errors: scaffolds'
description: The errors that tramlines reports for the scaffold that a new document starts from, when a schema can't derive it or the scaffold fails the schema.
order: 71
---

# Errors: scaffolds

These errors come from the scaffold of a schema: a part that the scaffold has no text for, a key that it can't write, two blocks that read as one, a marker that disagrees with its rule, and a scaffold that fails its own schema. Section 9 of the Markdown specification defines these errors.

## TLMD_116

tramlines can't derive a scaffold from the schema, because a part that the scaffold needs has no text in the schema ([TLMD_116][sokf:tramlines:tramlines-md-spec#tlmd_116]).

### Message

```text
<file>:<line>: error[TLMD_116]: the schema derives no scaffold: <reason>
```

### Cause

Before it checks the documents, the command derives a scaffold from the schema. The scaffold contains only text that the schema specifies in a `text` or an `example`. A required block, unit rule, heading, or frontmatter key declares neither, so the scaffold contains no text for it. The error is on the schema file, at the line of that rule. When an example lacks the key that the scaffold writes into it, the error is [TLMD_154](#tlmd_154). When text comes from a frontmatter key that the scaffold doesn't write, the error is [TLMD_155](#tlmd_155).

### Fix

In the schema, give the rule that the error names an `example`, or a `text` that sets its exact words.

### Example

This schema requires a paragraph but declares no example for it:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
```

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_116]: the schema derives no scaffold: a `prose` block has no `example`, and the scaffold writes no text of its own; give the block an `example`
1 error, 0 warnings
[exit 1]
```

Add an example:

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

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_154

tramlines can't derive a scaffold from the schema, because the example of a rule doesn't hold the key that the scaffold writes into it ([TLMD_154][sokf:tramlines:tramlines-md-spec#tlmd_154]).

### Message

```text
<file>:<line>: error[TLMD_154]: the schema derives no scaffold: a rule demands one unit per key of `<rule-id>`, and its example `<example>` does not carry `<key>` where `<hole>` reads it
<file>:<line>: error[TLMD_154]: the schema derives no scaffold: a section demands one heading per key of `<rule-id>`, and its example `<example>` does not carry `<key>` where `<hole>` reads it
<file>:<line>: error[TLMD_154]: the schema derives no scaffold: a key deriving from <source> cannot be written: `<key>` does not read as `<derived key>` where `<hole>` takes it
```

### Cause

The scaffold writes one unit or one heading for each key of the rule that `for-each-key-of` names. With `key-derives-from`, a derived key starts with the key that it derives from. To write each one, the scaffold puts the key into the example of the rule, at the hole that holds the key. The key doesn't fit that hole. The capturing rule allows keys that this rule's hole doesn't read. A key with a space, for example, doesn't fit a hole of type `word`. The error is on the schema file, at the line of the rule whose example is written.

### Fix

In the schema, give the two holes types that read the same keys. Make the example of the capturing rule a key that both holes read.

### Example

The tools of this schema are any text, but the pairing reads a `word`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tools
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: tools
        each: { template: '${name}', key: name, example: claw hammer }
      - kind: numbered-list
        each:
          template: '${name:word}: ${}'
          key: name
          for-each-key-of: tools
          example: 'hammer: drives nails'
```

**`note.md`**

```markdown
# Tools

- <a id="hammer"></a>hammer

1. hammer: drives nails
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TLMD_154]: the schema derives no scaffold: a rule demands one unit per key of rule `tools`, and its example `hammer: drives nails` does not carry `claw hammer` where `name` reads it
1 error, 0 warnings
[exit 1]
```

Read each tool as a `word`, with an example that is one:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Tools
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: tools
        each: { template: '${name:word}', key: name, example: hammer }
      - kind: numbered-list
        each:
          template: '${name:word}: ${}'
          key: name
          for-each-key-of: tools
          example: 'hammer: drives nails'
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_155

tramlines can't derive a scaffold from the schema, because a heading or a unit takes its text from a frontmatter key that the scaffold doesn't write ([TLMD_155][sokf:tramlines:tramlines-md-spec#tlmd_155]).

### Message

```text
<file>:<line>: error[TLMD_155]: the schema derives no scaffold: a section takes its heading from `<key>`, which no frontmatter writes
<file>:<line>: error[TLMD_155]: the schema derives no scaffold: a unit rule takes its text from `<key>`, which no frontmatter writes
```

### Cause

A heading declared as `{ from: <key> }`, or a unit rule with `text: { from: <key> }`, repeats the value of a frontmatter key. The scaffold writes a frontmatter key from its `const`, its template, its `example`, or the first value of its `enum`. The scaffold leaves the key out in three cases: the frontmatter doesn't declare it, its `only` excludes the states of the scaffold, or it's optional and declares none of them. The heading or the unit then has no text. The error is on the schema file, at the line of the section or the rule.

### Fix

In the schema, declare the frontmatter key, give it an `example`, and gate it no narrower than the rule that reads it.

### Example

The heading of this schema repeats the `title`, which declares no example:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    optional: true
sections:
  - heading: { from: title }
    level: 1
    blocks: []
```

**`note.md`**

```markdown
---
title: Groceries
---

# Groceries
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_155]: the schema derives no scaffold: a section takes its heading from `title`, which no frontmatter writes
1 error, 0 warnings
[exit 1]
```

Add an example to the key:

**`note.yaml`**

```yaml
line-limit: 0
frontmatter:
  title:
    optional: true
    example: Groceries
sections:
  - heading: { from: title }
    level: 1
    blocks: []
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_122

The `only` markers of a scaffold don't agree with the rules that match its parts when the command checks the scaffold against the schema ([TLMD_122][sokf:tramlines:tramlines-md-spec#tlmd_122]).

### Message

```text
<file>:<line>: error[TLMD_122]: <scaffold>: the `tramlines:only` marker before `<part>` gates nothing: the part it marks is not matched by a gated rule
<file>:<line>: error[TLMD_122]: <scaffold>: the `tramlines:only` marker before `<part>` disagrees with its rule, whose `only` is `<states>`
<file>:<line>: error[TLMD_122]: <scaffold>: the part `<part>` is matched by a rule whose `only` is `<states>` and carries no `tramlines:only` marker
```

### Cause

The scaffold marks each part that a rule with `only` writes. When the command checks the scaffold, each marked part must match a rule with the same `only`. Two rules that can match the same part break this agreement, because the check gives the part to the first rule that matches it. In the following example, a repeating section without `only` takes every `Details` heading, so the `Details` section with `only` never matches.

`<scaffold>` is `the scaffold`, or ``the scaffold derived for `<states>` `` when the schema has a lifecycle, and `<part>` is a line of that scaffold. A marker of a frontmatter key reads `on` instead of `before`, because it sits on the key's own line. When the command can tell which rule wrote the part, the error is on the line of that rule in the schema. Otherwise, the error is on the first line of the schema.

### Fix

In the schema, make sure that no other rule can match a part that a gated rule writes. For example, give that part a different heading.

### Example

The second `Details` section of this schema applies only to closed notes:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: state, states: [open, closed] }
sections:
  - { heading: Details, level: 1, optional: true, repeat: true, blocks: [] }
  - { heading: Details, level: 1, optional: true, only: { state: closed }, blocks: [] }
```

**`note.md`**

```markdown
---
state: open
---
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:6: error[TLMD_122]: the scaffold derived for `state=closed`: the `tramlines:only` marker before `# Details` gates nothing: the part it marks is not matched by a gated rule
1 error, 0 warnings
[exit 1]
```

Give the gated section a different heading:

**`note.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: state, states: [open, closed] }
sections:
  - { heading: Details, level: 1, optional: true, repeat: true, blocks: [] }
  - { heading: Outcome, level: 1, optional: true, only: { state: closed }, blocks: [] }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_124

A part that the schema doesn't require has no text, so the scaffold leaves it out ([TLMD_124][sokf:tramlines:tramlines-md-spec#tlmd_124]).

### Message

```text
<file>:<line>: warning[TLMD_124]: an optional part is left out of the scaffold: <reason>
```

### Cause

The scaffold writes each optional part inside an `optional` region, from the `text` or the `example` of its rule. An optional rule declares neither, so the scaffold leaves the part out and doesn't show it to the writer. `<reason>` can also be any fault that refuses a required part, such as a key that the scaffold can't write ([TLMD_154](#tlmd_154), [TLMD_155](#tlmd_155)) or two blocks that read as one ([TLMD_149](#tlmd_149)). The warning is on the schema file, at the line of that rule. It's a warning, so the command still checks the documents and exits with 0 when they pass.

### Fix

In the schema, give the optional rule an `example`, or fix the fault that the reason names.

### Example

This schema allows a list but declares no example for it:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
      - kind: bullet-list
        optional: true
```

**`note.md`**

```markdown
# Note

Buy milk.
```

```console
$ tramlines validate --warnings --schema note.yaml note.md
note.yaml:8: warning[TLMD_124]: an optional part is left out of the scaffold: a `bullet-list` block has no `example`, and the scaffold writes no text of its own; give the block an `example`
0 errors, 1 warning
```

Add an example:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
      - kind: bullet-list
        optional: true
        example: Milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_164

A scaffold derived from the schema fails the schema ([TLMD_164][sokf:tramlines:tramlines-md-spec#tlmd_164]).

### Message

```text
<file>:<line>: error[TLMD_164]: <scaffold> fails: [<key>] <message>
<file>:<line>: error[TLMD_164]: <scaffold> fails at `<text>`: [<key>] <message>
```

### Cause

Before it checks the documents, the command derives a scaffold for each combination of states and checks it against the schema. The scaffold contains every part of the schema, including each optional part that it has text for, and a rule in the schema rejects it. In the following example, the `line-limit` is too small for a document with the optional list.

`<scaffold>` is `the scaffold`, or ``the scaffold derived for `<states>` `` when the schema has a lifecycle. `<key>` and `<message>` are those of the error in the scaffold. When that error has a line, `<text>` is that line of the scaffold. The error is on the line of the rule in the schema that the scaffold fails, or on the first line of the schema where no rule is to blame. The error quotes the key of the error in the scaffold. To find the rule, look up that key.

### Fix

In the schema, change the rule that the quoted error names, so that a document with every allowed part can pass it.

### Example

This schema allows a list and limits a document to three lines. This example uses the `note.md` of [TLMD_124](#tlmd_124), which has three lines.

**`note.yaml`**

```yaml
line-limit: 3
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
      - kind: bullet-list
        optional: true
        example: Milk
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:1: error[TLMD_164]: the scaffold fails: [TL_21] the document has 5 lines; the limit is 3; split it (schema note)
1 error, 0 warnings
[exit 1]
```

Raise the limit:

**`note.yaml`**

```yaml
line-limit: 10
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        example: Buy milk.
      - kind: bullet-list
        optional: true
        example: Milk
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_149

tramlines can't derive a scaffold from the schema, because two consecutive blocks of the scaffold read as one block, and the first takes all of it ([TLMD_149][sokf:tramlines:tramlines-md-spec#tlmd_149]).

### Message

```text
<file>:<line>: error[TLMD_149]: the schema derives no scaffold: a `<kind>` block whose rule admits more units than the scaffold writes is followed by another `<kind>` block, which a document reads as one block, so the first takes all of it; give the first block a `max` of the units it holds
```

### Cause

A document reads paragraphs in a row as one prose block, and two lists of one kind as one list. When two such blocks follow each other, tramlines divides the run between their rules, and each rule takes as many units as it allows. A rule takes the whole run when two things hold: its `max` is absent or larger than the number of units that the scaffold writes for it, and its units state no shape, format, or prohibited pattern. Nothing is then left for the block after it. This happens between two blocks of a section, and between two repetitions of a `group`. The error is on the schema file, at the line of the first block.

### Fix

In the schema, give the first block a `max` equal to the number of units that it holds, or a shape that the next block's text fails. Or put a block of another kind between the two.

### Example

This schema repeats a paragraph at least twice, but the paragraph has no `max`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: group
        min: 2
        blocks:
          - kind: prose
            example: Buy milk.
```

**`note.md`**

```markdown
# Note

Buy milk.

Buy bread.
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:3: error[TLMD_146]: the group block needs at least 2 repetitions; it has 1 (schema note, section "Note" block 1)
note.yaml:9: error[TLMD_149]: the schema derives no scaffold: a `prose` block whose rule admits more units than the scaffold writes is followed by another `prose` block, which a document reads as one block, so the first takes all of it; give the first block a `max` of the units it holds
2 errors, 0 warnings
[exit 1]
```

Give the paragraph a `max` of 1, so that each repetition takes one paragraph:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: group
        min: 2
        blocks:
          - kind: prose
            max: 1
            example: Buy milk.
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_116]: /knowledge/spec/tramlines-md-spec.md#tlmd_116
[sokf:tramlines:tramlines-md-spec#tlmd_122]: /knowledge/spec/tramlines-md-spec.md#tlmd_122
[sokf:tramlines:tramlines-md-spec#tlmd_124]: /knowledge/spec/tramlines-md-spec.md#tlmd_124
[sokf:tramlines:tramlines-md-spec#tlmd_149]: /knowledge/spec/tramlines-md-spec.md#tlmd_149
[sokf:tramlines:tramlines-md-spec#tlmd_154]: /knowledge/spec/tramlines-md-spec.md#tlmd_154
[sokf:tramlines:tramlines-md-spec#tlmd_155]: /knowledge/spec/tramlines-md-spec.md#tlmd_155
[sokf:tramlines:tramlines-md-spec#tlmd_164]: /knowledge/spec/tramlines-md-spec.md#tlmd_164
