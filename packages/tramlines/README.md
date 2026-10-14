# @six5536/tramlines

tramlines validates documents against a schema: a YAML file that defines what a
document of one kind must contain: its frontmatter, headings, blocks, and the shape
of each line. Each rule includes an example, so a failure shows the writer what
to write instead. Markdown is the first format. tramlines ships as prebuilt
binaries for Linux, macOS, and Windows. Each release publishes them to
npm.

## Install

```sh
npm install -g @six5536/tramlines
```

This package is a thin launcher. It declares a prebuilt binary for each
supported platform as an `optionalDependency`, and npm installs only the one
matching your machine; a small JS shim then runs it.

**Supported platforms:** `x64` and `arm64` builds for Linux and macOS, and an
`x64` build for Windows. On any other platform, or to build from source, use the Rust toolchain
instead:

```sh
cargo install tramlines
```

## Usage

```sh
tramlines validate --schema list.yaml groceries.md          # check documents against one schema
tramlines validate --schema list.yaml --formats formats.yaml *.md   # with shared line formats
tramlines validate --schema list.yaml --format md notes.txt  # name the format of a document
tramlines validate --schema list.yaml --fix *.md             # repair headings and anchors first
tramlines validate --schema list.yaml --github *.md          # annotations for GitHub Actions
tramlines new --schema issue.yaml --state state=open         # print the scaffold of a new document
```

Each finding is one line, `path:line: severity[KEY]: message`, then a count.
`--json` prints the report as one JSON object instead, and `--warnings` lists
the warnings that the count line counts.
`KEY` names the rule of the specification that the finding breaks.
Exit codes: `0` no error, `1` errors found, `2` a usage error or a file that
cannot be read.

## Documentation

The [tramlines documentation](https://six5536.github.io/tramlines/) has a
tutorial, a guide for each part of a schema, the reference of every schema
key and every error, and the specifications.

## License

Elastic License 2.0. See <https://github.com/six5536/tramlines/blob/main/LICENSE>.
