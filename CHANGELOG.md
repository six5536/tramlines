# Changelog

This file records the notable changes of each release of tramlines. git-cliff writes the notes of each release from the commits of the release, and the notes of 0.2.0, the first public release, describe all that tramlines does. While tramlines is pre-1.0, a minor version might contain breaking changes.

## [0.2.1] - 2026-10-15

This release changes nothing in tramlines. Its crates and packages are the first that the public repository publishes to crates.io and npm through trusted publishing, with no stored token.

## [0.2.0] - 2026-10-15

This release is the first public release of tramlines. It publishes the crates `tramlines-core`, `tramlines-md`, and `tramlines` to crates.io, and the npm packages `@six5536/tramlines` and `@six5536/tramlines-<platform>` to npm.

### Added

- `for-each-key-of` takes a list of rules and pairs with the union of their
  keys, each distinct key once (TLMD_197). A rule that declares
  `key-repeats: true` captures one key on several units, or a key that
  another rule captures, and none of its captures clashes (TLMD_198).
- `Host::check` adds the rules of a host to each check of a document. The
  check of a schema then holds each scaffold and each example to those rules,
  and checks the gates of the host against the `only` markers of each
  scaffold (TLMD_174).
- The `example` of a block or a unit rule can be a list of texts. The
  scaffold writes one unit for each text, in order, so that a new document
  shows several items rather than one, and the check of the schema holds
  each text to its rule and names the one that fails (TLMD_176 to TLMD_188).
- A section with `repeat: true` can list further occurrences of itself in
  `examples`. The scaffold writes each one, unchanged, after the occurrence
  that it derives from the rules (TLMD_189 to TLMD_191).
- A heading goes to a varying heading of its own level before one of another
  level, so that a bare template such as `${}` never takes the title
  (TLMD_175).
- `Body::nested_headings` gives each heading inside a list item or a quote,
  with its slug, for a host that resolves a link to it.
- `tramlines validate --schema <file> [--formats <file>] [--format <name>] [<doc>...]`
  checks documents against one schema. The format of a document comes from
  its extension (`.md`), or from `--format`, and `-` reads a document from
  standard input. The command also holds the schema to its own examples and
  scaffolds, and checks a schema alone when it names no document.
- `validate --fix` rewrites the headings, bound paragraphs and items, and
  anchors that a schema derives, then reports what's left.
- `validate --json`, with `--query` for a JMESPath projection, and
  `validate --github` for GitHub workflow commands. Warnings are counted, and
  listed with `--warnings`.
- `tramlines new` prints the scaffold of a schema at the states that
  `--state` names, with its id and title filled from `--slug`, `--id` and
  `--title`.
- `tramlines info grammar` and `tramlines info formats`, `tramlines completions`,
  and a man page for packagers.
- `tramlines-md.schema.json`, a JSON Schema of a markdown schema generated
  from the grammar, attached to each release and served by the documentation
  site, so that an editor completes and checks the keys of a schema.
- `tramlines-core`, the part every format shares: the core grammar, shapes,
  types and formats, frontmatter and lifecycle, and findings.
- `tramlines-md`, the markdown format: its grammar names the core grammar as
  its base and adds headings, sections, blocks, and units.
- The specifications `knowledge/spec/tramlines-spec.md` and
  `knowledge/spec/tramlines-md-spec.md`, each rule a keyed EARS criterion.
  The key of each finding names one fault with one fix, and from 1.0 a
  released key is never renumbered, reused or removed.
- The npm packages `@six5536/tramlines` and `@six5536/tramlines-<platform>`, and the crates on crates.io.

### Changed

- Each entry of `prohibited-patterns` is a map of a `pattern` and a required
  `message`, one sentence that states the rule. A finding gives the message
  and the matched span in place of the pattern, and a scaffold writes the
  message after the guidance of the part that the ban governs, so a writer
  reads every rule before writing. A guidance line is no text of the
  document, so a ban of the schema doesn't check it (TL_17, TL_75, TL_76,
  TLMD_195, TLMD_196).
- A group that opens with a paragraph takes the end of the prose before it,
  so a section can hold prose followed directly by a worked example or a
  planned change, with no comment to separate them (TLMD_78, TLMD_193,
  TLMD_194).
- Each optional section of a scaffold has an optional region of its own, so a
  writer keeps one optional section and deletes the next. Two optional blocks
  that meet inside one section still share a region (TLMD_125, TLMD_192).

### Fixed

- A guarded rule of a block's `rules` counts the units that earlier rules
  write into a scaffold and its guard selects, and reads its keys out of them,
  so it no longer writes its example a second time (TLMD_199).
- When the host reports that an image or the target of a region names a
  document, the check reports it whatever the `source-pattern` of its block.
  A region's target was matched as written, and such an image was skipped
  (TLMD_200).
