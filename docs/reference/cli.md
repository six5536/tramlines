---
type: Reference
id: reference-cli
title: The command
description: The subcommands of `tramlines` and their options, the forms of the report, and the exit codes.
order: 10
---

# The command

This page lists the subcommands of `tramlines`, then the options of `tramlines validate`, how it picks the format of a document, the forms of its report, and its exit codes. It ends with `tramlines new`, `tramlines info`, `tramlines completions`, and the JSON Schema of a schema.

## Commands

```console
$ tramlines --help
Checks documents against a tramlines schema

Usage: tramlines <COMMAND>

Commands:
  validate     Check a schema, then each document against it, repairing with --fix
  new          Print the scaffold of a new document from a schema
  info         Print the schema language, or the formats and types a schema sees
  completions  Print a shell completion script to stdout
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Exit codes: 0 no error, 1 errors found, 2 usage error or a file that cannot be read.
Docs: https://six5536.github.io/tramlines/
```

## tramlines validate

`tramlines validate` checks the schema first, then each document against the schema. With no document, it checks the schema alone: its keys, its examples, and the scaffold of each combination of states.

```console
$ tramlines validate --help
Check a schema, then each document against it, repairing with --fix

Usage: tramlines validate [OPTIONS] --schema <FILE> [DOC]...

Arguments:
  [DOC]...  The documents to check; `-` reads one from standard input. Without a document, the command checks the schema alone

Options:
      --schema <FILE>   The schema file, in YAML
      --formats <FILE>  A file of line formats and types that the schema can use
      --format <NAME>   Read every document in this format, whatever its extension: `md`
      --fix             Repair the headings and anchors that the schema derives, then check
      --json            One JSON object on stdout instead of text
      --query <EXPR>    A JMESPath expression applied to the JSON object. Requires --json
      --github          GitHub workflow commands on stdout instead of text
      --warnings        List each warning. Default: their count
  -h, --help            Print help
```

- `--schema <FILE>`: the schema. The filename without its extension is the schema id, which each error message includes.
- `--formats <FILE>`: a file with a `registry` map of line formats and an optional `types` map. A format or type that the schema defines replaces one of the same name in this file.
- `--format <NAME>`: the format of every document. Without it, the command picks the format of each document from its extension.
- `<DOC>`: a document to check. `-` reads one document from standard input. It needs `--format`, and the report names the document `-`. The command reads standard input once, so `-` can appear only once.
- `--fix`: the command repairs the parts that the schema derives before it checks a document. These parts are a heading that repeats a frontmatter value, a heading listed in `renamed-from`, a paragraph or item bound to a frontmatter key, and the anchor of each key. The command writes a file only when the repairs change it. It keeps the file's line endings and its byte order mark. It reads every document before it writes any, so a file that can't be read stops the run with no file changed. With `-`, the command prints the repaired document on stdout and the report on stderr. It refuses `--json` and `--github` with `-`.
- `--warnings`: list each warning in the report. Without it, the count line counts the warnings, and no line lists them.
- `--json`, `--query <EXPR>`, `--github`: the form of the report, which [Report forms](#report-forms) describes.

## Formats

| Format | Extensions |
| --- | --- |
| `md` | `.md`, `.markdown` |

If a document's extension matches no format, the command stops with error [TL_90][sokf:tramlines:tramlines-spec#tl_90], unless `--format` names one. Standard input has no extension. The command reads a file that starts with a UTF-8 byte order mark as the text after the mark. If `--format` names an unknown format, the command stops with error [TL_91][sokf:tramlines:tramlines-spec#tl_91].

## Output

The command prints a `rewrote <path>` line for each file that `--fix` changed, one line for each error and listed warning, then a count:

```text
rewrote <path>
<path>:<line>: <severity>[<KEY>]: <message>
<n> errors, <m> warnings
```

- `<path>` is the document path as you gave it on the command line, or the schema path.
- `<line>` is the line number in the file. A finding with no line number has no `<line>:` part.
- `<severity>` is `error` or `warning`. A rule with `severity: warn` raises a warning instead of an error.
- `<KEY>` is the key of the criterion that the document or the schema breaks, such as `TL_54`. The error reference explains each one.
- When a schema rule raised the finding, `<message>` ends with the place of the rule in the schema, such as `(schema list, section "<title>" block 1)`.

The command sorts the findings by path, then by line number. When an error stops the command, the command prints one line on stderr for each fault, in the form `error[<KEY>]: <message>`. A fault that no criterion covers, such as a file that the command can't read, prints `error: <message>`. A fault inside the `--formats` file keeps the key that it has in the `formats` of a schema.

## Report forms

- `--json` prints one JSON object on stdout: `errors` and `warnings`, each a list of findings, and `fixes`, the files that `--fix` changed. A finding has `path`, `line` when it has a line, `key`, and `message`. A fix is `{ "op": "rewrote", "path": <path> }`. The object lists every warning, with or without `--warnings`.
- `--query <EXPR>` prints the result of a [JMESPath](https://jmespath.org/) expression over the JSON object, as one line of JSON. It requires `--json`. An expression that doesn't compile stops the command.
- `--github` prints each error and listed warning as a GitHub Actions workflow command: `::error file=<path>,line=<line>,title=<KEY>::<message>`. GitHub shows each command on the line of the file in a pull request. The `rewrote` lines and the count line follow as plain lines of the log. `--github` can't be combined with `--json`.

**`list.yaml`**

```yaml
line-limit: 50
frontmatter:
  title:
    example: Groceries
sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: bullet-list
        each: { template: "${item:word} x${n:int}", example: Milk x2 }
```

**`shopping.md`**

```markdown
---
title: Groceries
---

# Shopping

- Milk x2
- Bread
```

```console
$ tramlines validate --schema list.yaml shopping.md --github
::error file=shopping.md,line=5,title=TLMD_165::heading `Shopping` must equal the frontmatter `title`: `Groceries` (schema list, section "<title>")
::error file=shopping.md,line=8,title=TL_74::`Bread` fails at column 6: expected ` x`, found the end; an item of this kind reads Milk x2 (schema list, section "<title>" block 1)
2 errors, 0 warnings
[exit 1]
$ tramlines validate --schema list.yaml shopping.md --json
{"errors":[{"path":"shopping.md","line":5,"key":"TLMD_165","message":"heading `Shopping` must equal the frontmatter `title`: `Groceries` (schema list, section \"<title>\")"},{"path":"shopping.md","line":8,"key":"TL_74","message":"`Bread` fails at column 6: expected ` x`, found the end; an item of this kind reads Milk x2 (schema list, section \"<title>\" block 1)"}],"warnings":[],"fixes":[]}
[exit 1]
$ tramlines validate --schema list.yaml shopping.md --json --query 'errors[].key'
["TLMD_165","TL_74"]
[exit 1]
```

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | No errors. Warnings don't change the code. |
| `1` | At least one error. |
| `2` | The command line is wrong, a file is unreadable, or the command stopped on an error before it checked the documents. |

## tramlines new

`tramlines new` prints the scaffold of a schema: the document that a new document of the schema starts from. [Repair a document and start a new one][sokf:how-to-repair-and-start-documents] shows it at work.

```console
$ tramlines new --help
Print the scaffold of a new document from a schema

Usage: tramlines new [OPTIONS] --schema <FILE>

Options:
      --schema <FILE>      The schema file, in YAML
      --formats <FILE>     A file of line formats and types that the schema can use
      --slug <SLUG>        Fills the `slug` hole of the id template
      --id <ID>            The id, for a template with a `seq` hole or for no template
      --title <TEXT>       The title. Default: the words of the slug, or of the id
      --state <KEY=STATE>  A lifecycle key at a state, as KEY=STATE. Default: each dimension's first state
      --json               One object with the id and the document instead of the document
  -h, --help               Print help
```

- `--state <KEY>=<STATE>`: the state of one lifecycle dimension, repeated for each dimension that you name. A dimension that you don't name takes its first state. The scaffold contains the parts that the states need, and no `only` marker.
- `--slug <SLUG>`: fills the `slug` hole of the `id` template. A template with a `seq` hole needs the whole id from `--id`, because the number depends on the documents around the new one.
- `--id <ID>`: the id, which must match the `id` template when the schema has one.
- `--title <TEXT>`: the title. Without it, the title is the words of the slug, or of the id, when the schema declares `title`.
- `--json`: one JSON object, with the `id` and the `document`, instead of the document.

The command rewrites each heading, paragraph, and item that the schema binds to the id or the title. It stops with exit code 2 if the new document fails the schema.

## tramlines info

`tramlines info grammar` prints the schema language verbatim: the core grammar, then the markdown grammar, as two YAML documents.

`tramlines info formats` prints, as YAML, the line formats and types that a schema can use, after the layers resolve. Each entry has a `source`: `built-in`, the path of the formats file, or the id of the schema. With `--schema`, the command prints what that schema sees. Without it, the command prints the formats file with the built-in types.

## tramlines completions

`tramlines completions <SHELL>` prints a completion script for `bash`, `elvish`, `fish`, `powershell`, or `zsh`. For example, `tramlines completions bash > ~/.local/share/bash-completion/completions/tramlines` installs it for bash.

## The JSON Schema of a schema

[`tramlines-md.schema.json`](https://six5536.github.io/tramlines/tramlines-md.schema.json) is a JSON Schema of the YAML of a markdown schema. A test generates it from the grammar. An editor with YAML support reads it to complete and check the keys of a schema. To use it in an editor that runs the YAML language server, put the following line at the top of the schema:

```yaml
# yaml-language-server: $schema=https://six5536.github.io/tramlines/tramlines-md.schema.json
```

The JSON Schema checks the keys and the types of their values. The command checks the rest, such as the regular expressions, the examples, and the references between rules.

<!-- sokf:links -->

[sokf:how-to-repair-and-start-documents]: /docs/how-to/repair-and-start-documents.md
[sokf:tramlines:tramlines-spec#tl_90]: /knowledge/spec/tramlines-spec.md#tl_90
[sokf:tramlines:tramlines-spec#tl_91]: /knowledge/spec/tramlines-spec.md#tl_91
