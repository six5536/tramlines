# tramlines

[![License: Elastic 2.0](https://img.shields.io/badge/License-Elastic_2.0-blue.svg)](LICENSE)

tramlines validates Markdown files against a schema that you write in YAML. A schema is a set of rules. The rules describe which frontmatter fields, headings, lists, and paragraphs a file must contain, and the form of each line. Each rule includes an example line. When a line is wrong, the error shows that example, so you can see what to write.

## Features

- Checks frontmatter, headings, lists, paragraphs, and individual lines
- Accepts line rules written as sample lines with named blanks, or as regular expressions
- Lets schemas share line formats through a formats file
- Applies some rules only in certain document states, such as `draft` or `final`
- Shows an example of a correct line in each error message
- Checks the schema itself, including its example lines
- Checks the HTML anchors of the IDs in headings and lines
- Repairs the headings and anchors that a schema derives, and writes a starter document from a schema
- Prints its report as text, as JSON, or as GitHub annotations
- Includes a Rust library that does all of this, and accepts extra schema keys from other tools

## Installation

### Prerequisites

- Node.js 18 or later, or a Rust toolchain

### Install

With npm:

```bash
npm install -g @six5536/tramlines
```

With Cargo, from crates.io:

```bash
cargo install tramlines
```

## Usage

The following schema describes a shopping list: a title in the frontmatter, a level-1 heading with the same text as the title, and a bullet list. Each item of the list consists of a word, a space, `x`, and a number.

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

The following file follows the schema:

**`groceries.md`**

```markdown
---
title: Groceries
---

# Groceries

- Milk x2
- Bread x1
```

```console
$ tramlines validate --schema list.yaml groceries.md
0 errors, 0 warnings
```

In the following file, the heading differs from the title, and the last item has no count:

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
$ tramlines validate --schema list.yaml shopping.md
shopping.md:5: error[TLMD_165]: heading `Shopping` must equal the frontmatter `title`: `Groceries` (schema list, section "<title>")
shopping.md:8: error[TL_74]: `Bread` fails at column 6: expected ` x`, found the end; an item of this kind reads Milk x2 (schema list, section "<title>" block 1)
2 errors, 0 warnings
[exit 1]
```

Each error line takes the form `path:line: severity[KEY]: message`. `KEY` names the rule of the tramlines specification that the file breaks. The documentation explains each key. The end of the message names the rule in your schema that the line breaks. The error for the list item also shows the example line of that rule, `Milk x2`.

The command exits with one of three codes:

- 0 when it finds no errors
- 1 when it finds errors
- 2 when the command line is invalid or the command can't read a file

## Documentation

The [tramlines documentation](https://six5536.github.io/tramlines/) includes tutorials, a guide for each part of a schema, a reference for every schema key and every error, and the specifications.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

Elastic License 2.0. See [LICENSE](LICENSE).
