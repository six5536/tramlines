---
type: ErrorReference
id: errors-lines-and-pairings
title: 'Errors: lines and pairings'
description: The errors that tramlines reports for the lines of a code block and for rules that pair one unit with each key of another rule, and how to fix each one.
order: 61
---

# Errors: lines and pairings

These errors come from two kinds of rule. The `lines` rules check each line of a code block. The pairing rules pair a unit with each key of another rule. Section 6 of the Markdown specification defines them, with the other rules for units and lines.

## TLMD_87

A line of a code block matches none of the `lines` rules of its block ([TLMD_87][sokf:tramlines:tramlines-md-spec#tlmd_87]).

### Message

```text
<file>:<line>: error[TLMD_87]: `<line>` matches no rule of the fence; declare it or set `lines-open` (schema <id>, <place of the rule>)
```

### Cause

The unit rule of a code block declares `lines`, a grammar for the lines of the block. None of its rules with a shape matches this line, in whole or in part. The engine skips blank lines and ignores the indent of each line. When a line partly matches a rule, the engine reports `TLMD_57` instead.

### Fix

Change the line in the document so that a rule matches it, or remove the line. If the block can contain lines of this kind, add a rule for them under `lines` in the schema, or set `lines-open: true` on the rule that declares `lines`.

### Example

This schema accepts lines of the form `name=value`, and the document contains a comment line:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - kind: code
        lang: text
        each:
          lines:
            - { template: '${:word}=${}', example: port=80 }
```

**`note.md`**

````markdown
# Config

```text
port=80
# the default
```
````

```console
$ tramlines validate --schema note.yaml note.md
note.md:5: error[TLMD_87]: `# the default` matches no rule of the fence; declare it or set `lines-open` (schema note, section "Config" block 1)
1 error, 0 warnings
[exit 1]
```

Remove the comment:

**`note.md`**

````markdown
# Config

```text
port=80
```
````

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_90

A unit rule declares `lines` under a block that isn't a code block or an HTML block, or a line rule declares `lines` of its own ([TLMD_90][sokf:tramlines:tramlines-md-spec#tlmd_90]).

### Message

```text
<file>:<line>: error[TLMD_90]: `lines` applies to a `code` or `html` block, not a `<kind>` block
<file>:<line>: error[TLMD_90]: `lines` does not nest
```

### Cause

The error is on the schema. Only a code block or an HTML block contains raw lines for `lines` to check. The first message reports that a unit rule under a block of another kind declares `lines`. The second reports that a rule under `lines` declares `lines` again.

### Fix

In the schema, describe the units of the block with a shape, such as `template` or `pattern`, in place of `lines`. If the block contains raw lines, make it a `code` block with a `lang`. Under `lines`, remove the inner `lines`, and give the line rule a shape.

### Example

This schema declares `lines` on the paragraphs of a blockquote, so the run reports the error on the schema:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - kind: blockquote
        each:
          lines:
            - { template: '${:word}=${}', example: port=80 }
```

**`note.md`**

```markdown
# Config

> port=80
```

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:8: error[TLMD_90]: `lines` applies to a `code` or `html` block, not a `blockquote` block
1 error, 0 warnings
[exit 1]
```

Give each paragraph a template in place of `lines`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Config
    level: 1
    blocks:
      - kind: blockquote
        each: { template: '${:word}=${}', example: port=80 }
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_95

A rule pairs a unit or a section with each key that another rule captures, and one of those keys has no unit or section ([TLMD_95][sokf:tramlines:tramlines-md-spec#tlmd_95]).

### Message

```text
<file>:<line>: error[TLMD_95]: no <unit> names key `<key>` of rule `<rule-id>` (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_95]: no `<section>` section for key `<key>` (schema <id>, <place of the rule>)
```

### Cause

A unit rule or a section declares `for-each-key-of`, which names a rule by its `rule-id`. The engine expects one unit or section for each key that the named rule captures, and the document contains none for this key. The error is on the line that captures the key. Where the section declares a `parent`, the engine checks each occurrence of the parent on its own, and ends the message with ``under `<parent heading>` ``.

### Fix

Add to the document a unit or a section that names the key, or remove the key from the rule that captures it.

### Example

This schema pairs each item of the numbered list with a tool of the bullet list, and the document describes `hammer` but not `saw`:

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

**`note.md`**

```markdown
# Tools

- <a id="hammer"></a>hammer
- <a id="saw"></a>saw

1. hammer: drives nails
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:4: error[TLMD_95]: no item names key `saw` of rule `tools` (schema note, section "Tools")
1 error, 0 warnings
[exit 1]
```

Describe `saw`:

**`note.md`**

```markdown
# Tools

- <a id="hammer"></a>hammer
- <a id="saw"></a>saw

1. hammer: drives nails
2. saw: cuts wood
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_96

A unit or a section names a key that its paired rule doesn't capture ([TLMD_96][sokf:tramlines:tramlines-md-spec#tlmd_96]).

### Message

```text
<file>:<line>: error[TLMD_96]: key `<key>` is not one rule `<rule-id>` captured (schema <id>, <place of the rule>)
<file>:<line>: error[TLMD_96]: section names key `<key>`, which rule `<rule-id>` did not capture (schema <id>, <place of the rule>)
```

### Cause

A unit rule or a section declares `for-each-key-of`, which names a rule by its `rule-id`. The unit or section captures a key that the named rule doesn't capture in the same scope. Where the section declares a `parent`, the scope is one occurrence of the parent, and the message ends with ``under `<parent heading>` ``.

### Fix

Change the key in the document to one that the named rule captures, add the key to that rule, or remove the unit or section.

### Example

The schema is the same as in the `TLMD_95` example. This document describes `drill`, which isn't in the list of tools. A paired item refers to a key, so `drill` needs no anchor:

**`note.md`**

```markdown
# Tools

- <a id="hammer"></a>hammer

1. hammer: drives nails
2. drill: makes holes
```

```console
$ tramlines validate --schema note.yaml note.md
note.md:6: error[TLMD_96]: key `drill` is not one rule `tools` captured (schema note, section "Tools")
1 error, 0 warnings
[exit 1]
```

Remove `drill`:

**`note.md`**

```markdown
# Tools

- <a id="hammer"></a>hammer

1. hammer: drives nails
```

```console
$ tramlines validate --schema note.yaml note.md
0 errors, 0 warnings
```

## TLMD_98

A section or a unit rule declares `for-each-key-of`, but it captures no key of its own ([TLMD_98][sokf:tramlines:tramlines-md-spec#tlmd_98]).

### Message

```text
<file>:<line>: error[TLMD_98]: `for-each-key-of` on a rule that captures no key: declare `key`, or bind a `format` that declares one
<file>:<line>: error[TLMD_98]: `for-each-key-of` pairs headings by their key: the heading declares `key`
```

### Cause

The error is on the schema. A pairing matches each key that the pairing rule captures with a key of the named rule. The pairing rule must therefore capture a key of its own. Here, the unit rule or the heading with `for-each-key-of` declares no `key`, and binds no `format` that declares one.

### Fix

In the schema, add `key` to the pairing unit rule or heading, with the name of the hole that holds the key. Or bind a `format` that declares a key.

### Example

This schema pairs the numbered list with the tools, but the rule of the numbered list declares no `key`, so the run reports the error on the schema:

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
          for-each-key-of: tools
          example: 'hammer: drives nails'
```

The document is the fixed document from the `TLMD_96` example:

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TLMD_98]: `for-each-key-of` on a rule that captures no key: declare `key`, or bind a `format` that declares one
1 error, 0 warnings
[exit 1]
```

Add `key`:

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

## TLMD_167

The `for-each-key-of` of a section or a unit rule names no rule, or a rule that captures no key ([TLMD_167][sokf:tramlines:tramlines-md-spec#tlmd_167]).

### Message

```text
<file>:<line>: error[TLMD_167]: `for-each-key-of` names no rule `<rule-id>`
<file>:<line>: error[TLMD_167]: `for-each-key-of` names rule `<rule-id>`, which captures no key
```

### Cause

The error is on the schema. A pairing matches its keys with the keys of the rule that `for-each-key-of` names by its `rule-id`. No rule of the schema declares that `rule-id`, or the rule that declares it captures no key.

### Fix

In the schema, set `for-each-key-of` to the `rule-id` of a rule that captures a key, or add `key` to the rule that it names.

### Example

This schema pairs the numbered list with `tool`, but the bullet list declares the `rule-id` `tools`:

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
          for-each-key-of: tool
          example: 'hammer: drives nails'
```

The document is the fixed document from the `TLMD_96` example:

```console
$ tramlines validate --schema note.yaml note.md
note.yaml:11: error[TLMD_167]: `for-each-key-of` names no rule `tool`
1 error, 0 warnings
[exit 1]
```

Name the rule by its `rule-id`:

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

<!-- sokf:links -->

[sokf:tramlines:tramlines-md-spec#tlmd_167]: /knowledge/spec/tramlines-md-spec.md#tlmd_167
[sokf:tramlines:tramlines-md-spec#tlmd_87]: /knowledge/spec/tramlines-md-spec.md#tlmd_87
[sokf:tramlines:tramlines-md-spec#tlmd_90]: /knowledge/spec/tramlines-md-spec.md#tlmd_90
[sokf:tramlines:tramlines-md-spec#tlmd_95]: /knowledge/spec/tramlines-md-spec.md#tlmd_95
[sokf:tramlines:tramlines-md-spec#tlmd_96]: /knowledge/spec/tramlines-md-spec.md#tlmd_96
[sokf:tramlines:tramlines-md-spec#tlmd_98]: /knowledge/spec/tramlines-md-spec.md#tlmd_98
