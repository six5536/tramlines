---
type: Explanation
id: explanation-scaffolds
title: How a schema derives its scaffold
description: Why the engine builds a starter document from every schema, what the starter contains, and why a starter that fails the check is an error in its schema.
order: 45
---

# How a schema derives its scaffold

A scaffold is the document that a new document starts from. The engine builds scaffolds from the rules of a schema, one for each combination of lifecycle states, and checks each one against the schema. `tramlines new` prints the scaffold of the states that you name. A host program, such as an editor plugin, hands it to the writer of a new document. `tramlines validate` checks the scaffolds of each schema before it checks a document. A schema author therefore also sees a scaffold through the errors that it causes.

## Where the text of a scaffold comes from

The structure of a scaffold comes from the schema: its frontmatter keys, its sections, and its blocks. Its words come from the schema too, and from nowhere else:

- The `text` that a rule fixes, such as a heading that is the same in every document
- The `example` of each rule that requires text
- The `example` of the format that a rule uses, when the rule has no `example` of its own
- The `id` that the `template` of `id` produces for the states of the scaffold

The engine writes no text of its own. Filler such as "Write the summary here" would pass the check, tell the writer nothing, and be copied into every new document. So when a required part has no `text` and no `example`, the engine can't derive the scaffold, and the command reports the error in the schema:

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

A rule's example appears in two places: in every error that the rule reports, as a line of the kind that the rule expects, and in every new document. [Write examples and guidance][sokf:how-to-write-examples-and-guidance] shows how to write one.

## Guidance and markers

A scaffold contains three kinds of marker. Each marker is an HTML comment, or a YAML comment in the frontmatter:

- A `guidance` line before a part holds the `guidance` of the rule for that part: what no example can show. The first line of every scaffold is a guidance line that tells the writer to replace each part with their own text. The writer reads each guidance line and then deletes it.
- An `only` marker before a part names the states that the part applies in, from the `only` of its rule. A host keeps or drops each part according to the states of the new document.
- An `optional` region wraps a part that the schema doesn't require. The writer keeps its content and deletes both markers, or deletes the region.

The markers belong only in the scaffold. In a document that a schema checks, a `guidance` or `only` marker is an error, and an `optional` region left in place is a warning. [Keys, markers, and scaffolds][sokf:errors-keys-markers-scaffolds] lists these errors.

## Why the scaffold is checked

Checking a scaffold tests the schema against its own rules. When a scaffold fails, every new document starts with an error before its writer types a word. So the engine checks each scaffold against the schema, in the states of that scaffold. It also checks that each `only` marker agrees with the rule that the marker gates. It reports any failure as an error in the schema, with the key of the criterion that the scaffold breaks.

In this schema, each example passes its own rule, and `note.md` passes too. The scaffold also writes the optional list, so the scaffold is longer than the `line-limit`:

**`note.yaml`**

```yaml
line-limit: 3
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        each: { pattern: '^[A-Z].*\.$', example: 'Buy milk.' }
      - kind: bullet-list
        optional: true
        example: Eggs
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:1: error[TLMD_164]: the scaffold fails: [TL_21] the document has 5 lines; the limit is 3; split it (schema note)
1 error, 0 warnings
[exit 1]
```

The engine checks each example against its own rule first. When an example fails its rule, the scaffold line that holds the example fails too. The example check reports that fault, so the scaffold check doesn't report it again. Raise the limit, and the scaffold passes:

**`note.yaml`**

```yaml
line-limit: 5
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: prose
        each: { pattern: '^[A-Z].*\.$', example: 'Buy milk.' }
      - kind: bullet-list
        optional: true
        example: Eggs
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## Optional parts

An optional section or block is in the scaffold too, inside an `optional` region, because the schema author still wants the writer to see it. Each optional section has a region of its own, so the writer keeps one section and deletes the next. Two regions inside one section, with only blank lines between them, become one, so the writer keeps or deletes two optional blocks of a section together. When a part inside a region has no text to write, the scaffold leaves it out, and a warning names the rule that has no `example`.

## A fence with its own text

The engine builds a code block whose rule declares `lines` from the line rules, one line for each rule. A diagram built this way repeats one line many times. Instead, give the rule an `example` that holds the whole block. The scaffold copies the example, and the line rules read their keys from the copy.

## See also

- [TLMD_114][sokf:tramlines:tramlines-md-spec#tlmd_114]: one scaffold for each combination of states, checked against the schema.
- [TLMD_115][sokf:tramlines:tramlines-md-spec#tlmd_115] and [TLMD_116][sokf:tramlines:tramlines-md-spec#tlmd_116]: the engine writes no text itself.
- [TLMD_122][sokf:tramlines:tramlines-md-spec#tlmd_122]: markers that agree with their rules.
- [TLMD_123][sokf:tramlines:tramlines-md-spec#tlmd_123] to [TLMD_125][sokf:tramlines:tramlines-md-spec#tlmd_125], and [TLMD_192][sokf:tramlines:tramlines-md-spec#tlmd_192]: optional regions.
- [TLMD_126][sokf:tramlines:tramlines-md-spec#tlmd_126]: a fence with its own text.
- [Embed the engine in a Rust program][sokf:how-to-embed-the-engine]

<!-- sokf:links -->

[sokf:errors-keys-markers-scaffolds]: /docs/reference/errors-keys-markers-scaffolds.md
[sokf:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:tramlines:tramlines-md-spec#tlmd_114]: /knowledge/spec/tramlines-md-spec.md#tlmd_114
[sokf:tramlines:tramlines-md-spec#tlmd_115]: /knowledge/spec/tramlines-md-spec.md#tlmd_115
[sokf:tramlines:tramlines-md-spec#tlmd_116]: /knowledge/spec/tramlines-md-spec.md#tlmd_116
[sokf:tramlines:tramlines-md-spec#tlmd_122]: /knowledge/spec/tramlines-md-spec.md#tlmd_122
[sokf:tramlines:tramlines-md-spec#tlmd_123]: /knowledge/spec/tramlines-md-spec.md#tlmd_123
[sokf:tramlines:tramlines-md-spec#tlmd_125]: /knowledge/spec/tramlines-md-spec.md#tlmd_125
[sokf:tramlines:tramlines-md-spec#tlmd_126]: /knowledge/spec/tramlines-md-spec.md#tlmd_126
[sokf:tramlines:tramlines-md-spec#tlmd_192]: /knowledge/spec/tramlines-md-spec.md#tlmd_192
