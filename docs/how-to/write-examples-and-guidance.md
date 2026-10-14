---
type: HowTo
id: how-to-write-examples-and-guidance
title: Write examples and guidance
description: Give each rule an example that shows the text it requires, and guidance for what no example can show, so that the error for every failure shows the writer what to write.
order: 100
---

# Write examples and guidance

When you write a rule, give it an `example`, a real line of the kind that the rule requires, and add `guidance` for what no example can show. At the end, the error for every failure of the rule shows a line that passes, and a new document starts from text that a writer can imitate.

The example of a rule is the only part of a schema that shows the text that the rule requires. The command checks the example against its rule and prints it in the error for every failure of the rule. The scaffold, the document that a new document starts from, copies the example. A schema never describes the same text twice, so two wordings can't drift apart.

## Before you begin

- A schema with rules that have shapes, as in [Write the shape of a line][sokf:how-to-write-line-shapes].

## Steps

1. Write each `example` as a line that a real document of the kind would contain, about a real subject. Never write a placeholder such as `<decision>`, an instruction such as `Describe the decision.`, or filler such as `One item.` A writer, a person or an agent, imitates the examples in a new document. An example written as a prompt therefore comes back word for word.

2. Give an `example` to every rule with a `template` or a `pattern`, unless the rule binds a format that has one. A rule with only `text`, a count, or a `select-pattern` needs none, because its error is clear without one.

   The command checks every example against its rule and reports each one that fails as an error in the schema. This example lacks the full stop that its template requires:

**`minutes.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Decisions
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: 'Decided: ${}.'
          example: 'Decided: the release moves to 3 November'
```

**`minutes.md`**

```markdown
## Decisions

- Decided: the release moves to 3 November.
```

```console
$ tramlines validate --schema minutes.yaml minutes.md
minutes.yaml:8: error[TL_64]: the example `Decided: the release moves to 3 November` fails at column 41: expected `.`, found the end (schema minutes, section "Decisions" block 1)
1 error, 0 warnings
[exit 1]
```

3. Add `guidance` for what an example can't show, such as a choice, a limit, or a reason. Guidance also covers what a writer must avoid where no rule enforces it. Phrase it as what to do instead: a schema contains no wrong text for a writer to imitate. The schema, a section, a preamble, a block, a frontmatter key, and a lifecycle dimension each accept guidance. The scaffold places it before the part that it guides, and the command never prints it in an error.

4. To name a rule in its failures, give it a `rule-id`. Without one, the error names the rule by its place in the schema, its key, or its pattern.

**`meeting.yaml`**

```yaml
line-limit: 0
guidance: One meeting, with what it decided and who attended.
sections:
  - heading: Decisions
    level: 2
    guidance: Record what the meeting settled, not what it discussed.
    blocks:
      - kind: bullet-list
        rule-id: decisions
        each:
          template: 'Decided: ${}.'
          example: 'Decided: the release moves to 3 November.'
  - heading: Attendees
    level: 2
    blocks:
      - kind: bullet-list
        each:
          pattern: '^[A-Z]\w+ [A-Z]\w+$'
          example: Ana Silva
```

5. Check a document, and read each failure. An error starts with the key of the criterion that the line breaks, such as `TL_74`, and ends with the rule that the line fails. The error for a shape quotes the text, then either the column where a template stops matching or the pattern that the text fails, then the example. The command doesn't print the guidance of the section:

**`planning.md`**

```markdown
## Decisions

- Decided: the release moves to 3 November.
- We talked about the budget

## Attendees

- Ana Silva
- sam
```

```console
$ tramlines validate --schema meeting.yaml planning.md
planning.md:4: error[TL_74]: `We talked about the budget` fails at column 1: expected `Decided: `, found `We talked about the bud…`; an item of this kind reads Decided: the release moves to 3 November. (schema meeting, rule `decisions`)
planning.md:9: error[TL_74]: `sam` does not match `^[A-Z]\w+ [A-Z]\w+$`; an item of this kind reads Ana Silva (schema meeting, section "Attendees" block 1)
2 errors, 0 warnings
[exit 1]
```

6. To bring in a rule that existing documents don't follow yet, set `severity: warn` on it. Its failures are warnings, which don't change the exit code. Set it on the rule that fails: here, the unit rule under `each`. A `severity` on a block or a section reaches every rule inside it. On a section, it also covers the faults of the heading of the section, such as a heading that is missing, repeated, or out of place. For this reason, set it as close to the failing rule as you can. After the documents comply, set the severity to `error`, or remove the key.

**`meeting.yaml`**

```yaml
line-limit: 0
guidance: One meeting, with what it decided and who attended.
sections:
  - heading: Decisions
    level: 2
    guidance: Record what the meeting settled, not what it discussed.
    blocks:
      - kind: bullet-list
        rule-id: decisions
        each:
          template: 'Decided: ${}.'
          example: 'Decided: the release moves to 3 November.'
  - heading: Attendees
    level: 2
    blocks:
      - kind: bullet-list
        each:
          severity: warn
          pattern: '^[A-Z]\w+ [A-Z]\w+$'
          example: Ana Silva
```

**`planning.md`**

```markdown
## Decisions

- Decided: the release moves to 3 November.

## Attendees

- Ana Silva
- sam
```

```console
$ tramlines validate --warnings --schema meeting.yaml planning.md
planning.md:8: warning[TL_74]: `sam` does not match `^[A-Z]\w+ [A-Z]\w+$`; an item of this kind reads Ana Silva (schema meeting, section "Attendees" block 1)
0 errors, 1 warning
```

7. When you rename a heading, list its former text under `renamed-from` on the section. A document with the old heading fails with an error that names the new one, and `tramlines validate --fix` rewrites the heading.

**`meeting.yaml`**

```yaml
line-limit: 0
guidance: One meeting, with what it decided and who attended.
sections:
  - heading: Decisions
    level: 2
    renamed-from: Outcomes
    guidance: Record what the meeting settled, not what it discussed.
    blocks:
      - kind: bullet-list
        rule-id: decisions
        each:
          template: 'Decided: ${}.'
          example: 'Decided: the release moves to 3 November.'
  - heading: Attendees
    level: 2
    blocks:
      - kind: bullet-list
        each:
          severity: warn
          pattern: '^[A-Z]\w+ [A-Z]\w+$'
          example: Ana Silva
```

8. After you write the part that a guidance line guides, delete the line from the new document. A scaffold contains each piece of guidance as a `<!-- tramlines:guidance … -->` line before a part of the body, or a `# tramlines:guidance …` comment before a frontmatter key. A document that still contains one fails at that line:

**`review.md`**

```markdown
## Outcomes

<!-- tramlines:guidance Record what the meeting settled, not what it discussed. -->
- Decided: the venue is the town hall.

## Attendees

- Ana Silva
```

```console
$ tramlines validate --schema meeting.yaml review.md
review.md:1: error[TLMD_137]: heading `Outcomes` was renamed; it is now `Decisions` (schema meeting, section "Decisions")
review.md:3: error[TLMD_158]: a `tramlines:guidance` line is left here: write the part it guides, then delete the line
2 errors, 0 warnings
[exit 1]
```

## Troubleshooting

- `TL_64`: an example fails its own rule. Fix the example, or the shape if the example is right. See [TL_64][sokf:errors-examples#tl_64].
- `TL_65`: the example of a format fails the shape, the bans, or the key of the format. See [TL_65][sokf:errors-examples#tl_65].
- `TL_66`: a rule with a `template` or a `pattern` has no `example`. Add one. See [TL_66][sokf:errors-examples#tl_66].
- `TL_68`: the `guidance` of a rule repeats its `example`. Write what the example can't show, or remove the guidance. See [TL_68][sokf:errors-examples#tl_68].
- `TLMD_158`: a guidance line is left in a document. Write the part that it guides, then delete the line. See [TLMD_158][sokf:errors-keys-markers-scaffolds#tlmd_158].
- `TLMD_164`: the scaffold that the schema derives from its examples fails the schema, though each example passes its own rule. See [TLMD_164][sokf:errors-scaffolds#tlmd_164].
- `TLMD_137`: a heading still uses a former text, listed under `renamed-from`. Change it to the heading that the message names. See [TLMD_137][sokf:errors-keys-and-repairs#tlmd_137].

## See also

- [Schema keys][sokf:reference-schema-keys]
- [Errors][sokf:reference-errors]
- [Read an error and fix the document][sokf:how-to-read-a-finding]
- [Strict by default][sokf:explanation-strict-by-default]
- [How a schema derives its scaffold][sokf:explanation-scaffolds]
- [Section 7 of the core specification][sokf:tramlines:tramlines-spec#7-examples-and-guidance]

<!-- sokf:links -->

[sokf:errors-examples#tl_64]: /docs/reference/errors-examples.md#tl_64
[sokf:errors-examples#tl_65]: /docs/reference/errors-examples.md#tl_65
[sokf:errors-examples#tl_66]: /docs/reference/errors-examples.md#tl_66
[sokf:errors-examples#tl_68]: /docs/reference/errors-examples.md#tl_68
[sokf:errors-keys-and-repairs#tlmd_137]: /docs/reference/errors-keys-and-repairs.md#tlmd_137
[sokf:errors-keys-markers-scaffolds#tlmd_158]: /docs/reference/errors-keys-markers-scaffolds.md#tlmd_158
[sokf:errors-scaffolds#tlmd_164]: /docs/reference/errors-scaffolds.md#tlmd_164
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
[sokf:how-to-read-a-finding]: /docs/how-to/read-a-finding.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-errors]: /docs/reference/errors.md
[sokf:reference-schema-keys]: /docs/reference/schema-keys.md
[sokf:tramlines:tramlines-spec#7-examples-and-guidance]: /knowledge/spec/tramlines-spec.md#7-examples-and-guidance
