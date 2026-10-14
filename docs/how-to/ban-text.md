---
type: HowTo
id: how-to-ban-text
title: Ban text in a document
description: Ban text from every line of a document, from one kind of line, or from every line of a format, with `prohibited-patterns`.
order: 50
---

# Ban text in a document

When some text must never appear, list a regular expression for it under `prohibited-patterns`, with a message that states the rule. Such text might be a `TODO` left in a finished document, or a vague word in a changelog. Each line that matches the expression then fails, and its error quotes the matched text and gives the message. The scaffold of a new document shows the message too, so that its writer reads the rule before writing.

You can set the key at three levels. A ban on the schema applies to every line of the body. A ban on a rule applies to the text of each unit that the rule checks. A ban on a format applies to every line bound to the format.

## Before you begin

- A schema with a list, as in [Your first schema][sokf:tutorial-first-schema].
- To ban text in a format, a format that you defined, as in [Share line formats across schemas][sokf:how-to-share-line-formats].

## Steps

1. To ban text anywhere in the body, list it under `prohibited-patterns` on the schema. Each entry has a `pattern` and a `message`. The `pattern` is an unanchored regular expression in the Rust dialect. The `message` is one sentence that states the rule as a fact about the document. Use `\b` to match whole words, and `(?i)` to ignore case.

2. To ban text in one kind of unit, put `prohibited-patterns` on the rule that checks it.

3. To ban text in every line of a format, put `prohibited-patterns` on the format. The command checks the shape of a line first, then the bans of its rule and of its format.

   This changelog schema bans `TODO` everywhere, the word `various` in a list item, and `...` in every line of the format `change`:

**`changelog.yaml`**

```yaml
line-limit: 0
prohibited-patterns:
  - pattern: '\bTODO\b'
    message: 'A finished changelog contains no TODO.'
types:
  change: [Added, Changed, Fixed, Removed]
formats:
  change:
    template: '${:change}: ${}'
    example: 'Fixed: the parser reads a tab as four spaces.'
    prohibited-patterns: [{pattern: '\.\.\.', message: 'A change ends with a full stop, not with an ellipsis.'}]
sections:
  - heading: Changelog
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: change
          prohibited-patterns: [{pattern: '(?i)\bvarious\b', message: 'An item names its change rather than calling it various.'}]
      - kind: code
        lang: text
        optional: true
        example: tramlines validate --schema list.yaml groceries.md
```

4. Check a document. The command applies no ban inside fenced blocks or code spans, so text that you quote as code passes:

**`changelog.md`**

````markdown
# Changelog

- Added: the `--formats` option. TODO document it
- Fixed: various bugs.
- Changed: the `TODO` marker in a code span is fine...

```text
TODO in a fence is fine
```
````

```console
$ tramlines validate --schema changelog.yaml changelog.md
changelog.md:3: error[TL_75]: `TODO` is refused: A finished changelog contains no TODO. (schema changelog)
changelog.md:4: error[TL_75]: `various` is refused: An item names its change rather than calling it various. An item of this kind reads Fixed: the parser reads a tab as four spaces. (schema changelog, section "Changelog" block 1)
changelog.md:5: error[TL_75]: `...` is refused: A change ends with a full stop, not with an ellipsis. An item of this kind reads Fixed: the parser reads a tab as four spaces. (schema changelog, section "Changelog" block 1)
3 errors, 0 warnings
[exit 1]
```

Each finding quotes the span that the pattern matched, and gives the message of the ban, not its pattern. When the ban is on a rule or a format, the finding also shows the example of that rule or format, so that you see a line that passes. The scaffold of the schema writes the message of each ban after the guidance of the part that the ban governs. It writes the message of a ban on the schema in its opening line.

## Troubleshooting

- `TL_75`: a line of the document matches a banned pattern. Rewrite the span that the message quotes. If the span is a name, not prose, quote it as code instead. See [TL_75][sokf:errors-lines#tl_75].
- `TL_118`: a pattern isn't a valid regular expression in the Rust dialect. For example, the dialect has no lookarounds and no backreferences. See [TL_118][sokf:errors-schema#tl_118].

## See also

- [Schema keys: `schema`][sokf:reference-schema-keys#schema]
- [Schema keys: `unit`][sokf:reference-schema-keys#unit]
- [Schema keys: `format`][sokf:reference-schema-keys#format]
- [Section 8 of the core specification][sokf:tramlines:tramlines-spec#8-findings]

<!-- sokf:links -->

[sokf:errors-lines#tl_75]: /docs/reference/errors-lines.md#tl_75
[sokf:errors-schema#tl_118]: /docs/reference/errors-schema.md#tl_118
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:reference-schema-keys#format]: /docs/reference/schema-keys.md#format
[sokf:reference-schema-keys#schema]: /docs/reference/schema-keys.md#schema
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
[sokf:tramlines:tramlines-spec#8-findings]: /knowledge/spec/tramlines-spec.md#8-findings
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
