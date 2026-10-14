---
type: ErrorReference
id: errors-command
title: 'Errors: the command'
description: The errors that stop `tramlines validate` before it checks a document, and how to fix each one.
order: 80
---

# Errors: the command

These errors stop `tramlines validate` with exit code 2 before it checks any document. They report a `--formats` file that the command can't use, a document whose format it can't determine, and a `--format` that it doesn't know. Section 10 of the core specification defines them.

## TL_87

The file that `--formats` names doesn't parse as YAML, or has no `registry` key ([TL_87][sokf:tramlines:tramlines-spec#tl_87]).

### Message

```text
error[TL_87]: <file>: <reason>
```

### Cause

The command reads the `--formats` file as YAML, with a `registry` map of formats and an optional `types` map. The command stops when the file doesn't parse or has no `registry` key. A fault inside the registry or the types also stops the command, such as a format with no `example` or a type named `slug`. Each such fault prints the key that it has in the `formats` of a schema, with the file and the line ([TL_124][sokf:tramlines:tramlines-spec#tl_124]). A YAML anchor, alias, or tag in the file stops the command under [TL_14][sokf:errors-schema#tl_14], as it does in a schema. The command prints each error to stderr and omits the count of errors and warnings.

### Fix

Change the `--formats` file so that it parses as YAML, and put the formats under a `registry` key. For a fault inside the registry, look up the key that the error prints.

### Example

This schema binds a format `item`:

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks:
      - kind: bullet-list
        each:
          format: item
```

**`note.md`**

```markdown
# Note

- milk
```

This formats file defines `item`, but not under a `registry` key:

**`formats.yaml`**

```yaml
item:
  template: '${:slug}'
  example: milk
```

```console
$ tramlines validate --schema note.yaml --formats formats.yaml note.md
error[TL_87]: formats.yaml: no registry
[exit 2]
```

Put the format under `registry`:

**`formats.yaml`**

```yaml
registry:
  item:
    template: '${:slug}'
    example: milk
```

```console
$ tramlines validate --schema note.yaml --formats formats.yaml note.md
0 errors, 0 warnings
```

A fault inside the registry prints its own key. This format has no `example`, which [TL_40][sokf:errors-formats-and-examples#tl_40] describes:

**`formats.yaml`**

```yaml
registry:
  item:
    template: '${:slug}'
```

```console
$ tramlines validate --schema note.yaml --formats formats.yaml note.md
error[TL_40]: formats.yaml:3: format `item`: a format with one shape carries an `example`
[exit 2]
```

Add the `example` back:

**`formats.yaml`**

```yaml
registry:
  item:
    template: '${:slug}'
    example: milk
```

```console
$ tramlines validate --schema note.yaml --formats formats.yaml note.md
0 errors, 0 warnings
```

## TL_90

The extension of a document matches no format, and the command line includes no `--format` ([TL_90][sokf:tramlines:tramlines-spec#tl_90]).

### Message

```text
error[TL_90]: <file>: no format matches the extension `<extension>`; name one with `--format` (<formats>)
```

### Cause

The command picks the format of each document by its file extension: `.md` and `.markdown` select the `md` format. The command checks the extension of every document before it reads any. One document with another extension, or with none, stops the whole run.

### Fix

Name the format of the documents with `--format`, or give the document a filename with an extension that a format matches.

### Example

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
```

This document is markdown, but its filename ends in `.txt`:

**`note.txt`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml note.txt
error[TL_90]: note.txt: no format matches the extension `txt`; name one with `--format` (`md`)
[exit 2]
```

Name the format:

```console
$ tramlines validate --schema note.yaml --format md note.txt
0 errors, 0 warnings
```

## TL_91

`--format` names a format that the command doesn't know ([TL_91][sokf:tramlines:tramlines-spec#tl_91]).

### Message

```text
error[TL_91]: no format `<name>`; the formats are <formats>
```

### Cause

`--format` takes the name of a format, such as `md`, not the name of a language or an extension.

### Fix

Pass `--format` one of the formats that the message lists.

### Example

**`note.yaml`**

```yaml
line-limit: 0
sections:
  - heading: Note
    level: 1
    blocks: []
```

**`note.txt`**

```markdown
# Note
```

```console
$ tramlines validate --schema note.yaml --format markdown note.txt
error[TL_91]: no format `markdown`; the formats are `md`
[exit 2]
```

Use the name of the format:

```console
$ tramlines validate --schema note.yaml --format md note.txt
0 errors, 0 warnings
```

<!-- sokf:links -->

[sokf:errors-formats-and-examples#tl_40]: /docs/reference/errors-formats-and-examples.md#tl_40
[sokf:errors-schema#tl_14]: /docs/reference/errors-schema.md#tl_14
[sokf:tramlines:tramlines-spec#tl_124]: /knowledge/spec/tramlines-spec.md#tl_124
[sokf:tramlines:tramlines-spec#tl_87]: /knowledge/spec/tramlines-spec.md#tl_87
[sokf:tramlines:tramlines-spec#tl_90]: /knowledge/spec/tramlines-spec.md#tl_90
[sokf:tramlines:tramlines-spec#tl_91]: /knowledge/spec/tramlines-spec.md#tl_91
