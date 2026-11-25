---
type: Reference
id: reference-schema-keys
title: Schema keys
description: Every key that a markdown schema can use, by the part of the schema that it belongs to.
order: 20
generator: schema-keys
---

# Schema keys

This page lists every key that a markdown schema can use. Each section is one part of a schema, starting with the schema itself, and lists its keys in the order of the grammar. A test writes this page from the grammar, so change the grammar and not this page.

## `schema`

What a schema can state about the documents that it governs. A schema is strict by default: whatever it declares is required, whatever it doesn't declare is rejected, and each requirement is exclusive. You relax a rule only with a key that you write on purpose. A key that ends in `-open` opens a set that is otherwise rejected, and `optional` and `repeat` loosen a count. Regular expressions use the Rust dialect, with no lookaround or backreferences, and aren't anchored. The shape of one unit, such as a line or a frontmatter value, is exactly one of these: `text`, the text itself; `template`, the text with typed holes `${name:type}` where it varies, matched whole; or `pattern`, a regular expression. Every shape reads the unit's text squashed, with each run of whitespace read as one space. Each rule has an `example`, which is text of the kind that the rule requires. The engine checks the example against the rule, shows it with every failure that the rule raises, and writes it into the scaffold. The scaffold is the document that the schema derives and that a new document starts from. A rule that declares a template or a pattern must have an example. For what an example can't show, use `guidance`. The engine writes guidance into the scaffold before the part that it governs, and no failure prints it. In a markdown document, a unit is a heading, a paragraph, an item, a row, or a line of a fence. The body is the document's sections, in heading order, and the preamble before the first heading.

A `schema` has at least one of `preamble`, `sections`.

- `line-limit` (an integer, required). The most lines that a document can have. A longer document is an error, and you split it. A value of zero or less sets no limit.
- `guidance` (a string). What the schema states that no example can show. The engine opens the body of the scaffold with it. No failure prints it.
- `language` (a BCP 47 language tag). The language of the schema's text, as a BCP 47 tag, such as `en` or `de`. The engine gives the tag to the host, which words each limit and each label in that language. The engine writes no words of its own: where the host gives none, it writes the limit in the syntax of the schema, and a label as its key.
- `lifecycle` (a list of [`lifecycle-dimension`](#lifecycle-dimension)). The states that the documents of the schema move through, with one entry per dimension, in order. Each dimension is a required frontmatter enum. Don't repeat it under `frontmatter`. A dimension is what a condition can name. The engine derives one scaffold for each combination of states.
- `frontmatter` (a map of [`frontmatter-constraint`](#frontmatter-constraint)). The frontmatter keys, with one entry per key. A declared key is required unless it sets `optional`. The engine never rejects the keys that the host reserves. Declare a reserved key to require it or to narrow its value. An undeclared key that is neither reserved nor a lifecycle key is an error unless `frontmatter-open` is set.
- `frontmatter-open` (`true` or `false`). Set to `true` to accept undeclared frontmatter keys. When absent, an undeclared key that isn't reserved or a lifecycle key is an error.
- `prohibited-patterns` (a list of [`prohibition`](#prohibition)). Patterns that no body line may match, each with the message that states its rule. Fenced blocks, code spans, and the interiors of regions are excluded: the patterns read each code span as a run of backticks of the same length, so they match no word inside it. A unit rule and a format take the same key, which then applies to the text that they govern instead.
- `formats` (a map of [`format`](#format)). Line formats local to the schema, in the shape of the shared registry. The nearest layer wins: an entry here replaces an entry of the same name in a farther layer entirely.
- `types` (a map of strings or lists of strings). Hole types local to the schema, in the shape of the shared registry. Each entry has a slug as its name and, as its value, either a regular expression with no anchor and no capture group or a list of literal choices. The nearest layer wins by name, and the engine compiles every format that the schema sees against the result. The built-in types are reserved: `text` (`.+`), `word`, `int`, `slug`, and `seq`.
- `examples` (a list of [`example`](#example)). Complete documents that satisfy this schema. The engine checks each one as it checks any other document, at the states that its own frontmatter gives, so a marker in an example is an error. Examples show what a document of the type looks like, and the engine never copies them.
- `preamble` ([`preamble`](#preamble)). The content between the frontmatter and the first heading. Content there is an error when no preamble is declared. A document with no headings is all preamble.
- `sections` (a list of [`section`](#section)). The document's headings, in document order. A heading that no entry matches is an error unless an open region admits it. A heading more than one level deeper than the heading before it is an error. The engine reports an ordered schema whose entries skip a level on the schema.
- `sections-ordered` (`true` or `false`). Set to `false` to accept the declared sections in any order. When absent, the sections must follow the listed order, and a heading whose section the schema lists before the section of the heading before it is an error that names the heading.
- `sections-prohibited` (a list of [`prohibited-heading`](#prohibited-heading)). Headings that are banned inside open regions.
- `labels` (a list of strings). The keys of the labels that a unit can carry, each a slug, such as `[human, agent]`. A label is its words in square brackets followed by one space, such as `[human] `. It stands straight after the key of a unit that begins with the key that its rule captures, and otherwise at the start of the unit, after the box of a task item. The host gives the words of each label in the language of the schema, and a label whose host gives none has its key as its words. A block or a unit rule that declares `label` needs this key.

## `lifecycle-dimension`

One ordered dimension of a lifecycle.

- `key` (a slug, required). The frontmatter key that contains the state.
- `states` (a list of strings, required). The states, in lifecycle order. The key's value must be one of them.
- `guidance` (a string). What the states mean. The engine writes it into the scaffold as a YAML comment line before the key.

## `frontmatter-constraint`

What a schema can state about one frontmatter key. The key is required unless it sets `optional`.

A `frontmatter-constraint` also accepts the keys of [`rule-common`](#rule-common).

- `const` (a string). The value must equal this.
- `template` (a string). The value written with typed holes where it varies, matched whole. On `id`, the template can contain only the holes that the engine fills in a new document. A `${:seq}` hole is unique within the kind, and the template can contain at most one. A `${:slug}` hole is one or more slug words. A `${<key>}` hole has no type and is named for a lifecycle key whose states are slugs. It contains the document's state for that key. Replacing each hole with `x` must yield a slug, and only `seq-width` can sit beside the template.
- `pattern` (a regular expression). A regular expression that the value must match, for what a template can't express.
- `enum` (a list of strings). The allowed values. The value must be one of them.
- `format` (a string). The format that the value must have. `url` requires an absolute URL. `slug` requires lowercase letters and digits joined by single hyphens. Values: `url`, `slug`.
- `seq-width` (an integer). On `id`, the number of digits of the `seq` hole. When absent, the width is three. Requires `template` beside it.
- `list` (`true` or `false`). Set to `true` when the value is a list. Then `const`, `pattern`, `enum`, and `format` apply to every element, and `min` and a hard `max` bound the number of elements. A scalar where a list is declared, or a list where none is declared, is an error.
- `min` (an integer). The minimum number of elements. When absent, there is no lower bound. Requires `list` beside it.
- `max` ([`limit`](#limit)). The limit on the number of elements, and what the writer does with more. When absent, there is no upper bound. Requires `list` beside it.
- `optional` (`true` or `false`). Set to `true` to let the key be absent. When absent, the key is required.
- `only` (a map of lifecycle dimensions to states, or a list of such maps). The condition in which the key applies, as on every rule that takes `only`.
- `example` (a string). The key's value, or one element of it when `list` is declared.
- `guidance` (a string). What an example can't show about this key. The engine writes it into the scaffold as a YAML comment line before the key.

## `prohibition`

One banned pattern and the message that states its rule. A finding gives the message and the span that the pattern matched, and a scaffold writes the message after the guidance of the part that the pattern governs.

- `pattern` (a regular expression, required). The regular expression that no governed text may match.
- `message` (a string, required). One sentence that states the rule as a fact about the part, in the voice of the document, such as "An interface's summary contains no modal verb."

## `format`

One named line grammar, in a schema's `formats` or in a registry. A format has either one shape and its example, or `variants`, and each variant has a shape and an example. The engine shows an example beside every failure of its shape. An entry replaces the entry of the same name in a farther layer entirely, unless it declares `patch`.

A `format` has exactly one of `text`, `template`, `pattern`, `variants`.

- `patch` (`true` or `false`). Set to `true` when this entry states only how it differs from the entry of the same name in a farther layer. The engine merges the two by JSON Merge Patch (RFC 7386). A key stated here replaces the farther one, an omitted key is inherited, a key set to null is dropped, and a list replaces the farther list entirely. The keys below apply to the merged entry, so a patch contains only what it changes. The engine reports a patch that shadows no farther entry on the schema. When absent, the entry replaces the farther entry entirely.
- `text` (a string). The text that every bound line must equal.
- `template` (a string). The line written with typed holes where it varies, matched whole.
- `pattern` (a regular expression). A regular expression that every bound line must match, for what a template can't express.
- `key` (a string). The hole or named group, present in every shape of the format, whose capture is the line's key.
- `variants` (a map of [`format-variant`](#format-variant)). The named shapes of the format, in the order in which a line tries them. A line conforms when one of them matches. When none matches, the failure names the variant that the line follows furthest, the place where the line fails, and that variant's example. The example of each variant must match only its own variant.
- `prohibited-patterns` (a list of [`prohibition`](#prohibition)). Patterns that no bound line may match, checked after the shape, each with its message. Code spans are excluded, as they are from the schema's patterns: the patterns read each one as a run of backticks of the same length, so they match no word inside it. A line of a fence or of an HTML block holds no code span, so the patterns read all of it.
- `example` (a string). One conforming line, for a format with one shape. In a format with variants, each variant has its own example.

## `example`

One complete document that satisfies the schema at its own lifecycle states. The engine checks it as it checks any other document, and reports a marker in it on the schema.

- `document` (a string, required). The complete document.

## `preamble`

The content before the first heading. A preamble has no heading, level, or repeat, only blocks.

A `preamble` also accepts the keys of [`rule-common`](#rule-common).

- `blocks` (a list of [`block`](#block)). The preamble's blocks, as for a section's `blocks`.
- `blocks-ordered` (`true` or `false`). As for a section's `blocks-ordered`.
- `guidance` (a string). What an example can't show about the preamble. The engine writes it into the scaffold before the preamble.

## `section`

One heading of the document. A section is required unless it sets `optional`, and occurs once unless it sets `repeat`. Its body runs to the next heading of any level. Nesting is by level, never by containment, except that a section that declares `parent` lies inside its parent. When an exact heading and a varying heading both match, the exact heading wins.

A `section` also accepts the keys of [`rule-common`](#rule-common).

- `heading` (a string, `{ from: <frontmatter key> }`, or [`heading`](#heading), required). The heading. It is the exact text, `{ from: <key> }` for the frontmatter value, or a `heading` rule for a heading that varies or captures a key. A string contains no hole. Write a heading that varies as a template, and give the template an example.
- `renamed-from` (a string, or a list of them). Former heading text, beside an exact heading. The engine reports a heading that has an old text, and its repair rewrites the heading.
- `rule-id` (a slug). The name by which other rules reference this section. It must be unique within the schema.
- `level` (an integer, required). The heading depth. Values: `1`, `2`, `3`, `4`, `5`, `6`.
- `optional` (`true`, `false`, a map of lifecycle dimensions to states, or a list of such maps). Set to `true` to let the section be absent. Set to a condition, as for `only`, to let the section be absent where the document's state matches the condition and require it elsewhere. When absent, the section is required.
- `repeat` (`true` or `false`). Set to `true` to let the heading occur more than once. When absent, it occurs once.
- `headings-open` (`true` or `false`). Set to `true` to accept undeclared headings of this level or deeper after this section, up to the next declared heading or the next heading of a higher level. Only document-scope rules apply to their bodies, and `sections-prohibited` still applies.
- `subheadings-open` (`true` or `false`). Set to `true` to act as `headings-open` does, but only for undeclared headings deeper than this level, so a heading of this level or higher ends the region.
- `for-each-key-of` (the name of a set of keys, or a list of them). Requires one occurrence of this section for each key filed under the referenced name, each key once. The name is the `rule-id` of a section or a block, or a `key-of`, and never the `rule-id` of a unit rule, which holds no keys. A list of names pairs with the keys of any of them, and a key that two of them hold needs one occurrence. The engine reads the key of each occurrence by the heading's `key`, which the heading must declare. The referenced name must hold a key. A missing occurrence and an orphan are each an error that names the key. Under a `parent`, the keys are those captured inside the same occurrence of the parent.
- `parent` (the `rule-id` of a section). The section that this one lies inside. The parent must have a smaller `level` and be listed before this section, with only its own subsections between them. Each occurrence must follow an occurrence of the parent, before the next heading of the parent's level or higher. `optional` and `repeat` count within each occurrence of the parent. A body still runs to the next heading.
- `number-from-parent` (`true` or `false`). Set to `true` when the heading must begin with the number of its parent's heading, a dot, and a number, such as `4.2.1` under `4.2`. A number is the text before the first space: digits separated by dots. A parent heading with no number is an error. Requires `parent` beside it.
- `blocks` (a list of [`block`](#block)). The body, as an ordered list of blocks. When absent, the body is one prose block of one or more paragraphs. `[]` means an empty body. The body must be exactly these blocks, in order unless `blocks-ordered` is `false`.
- `blocks-ordered` (`true` or `false`). Set to `false` to accept the declared blocks in any order. When absent, the blocks must appear in the declared order.
- `only` (a map of lifecycle dimensions to states, or a list of such maps). The condition in which the rule applies: a map of lifecycle dimensions to states, or a list of such maps. A map matches when the document's state is in the set for every dimension that it names, and a list matches when any of its maps does. When absent, the rule always applies. The engine reports on the schema a dimension or state that the lifecycle doesn't declare, an empty list, a dimension with no state, or `only` without a lifecycle.
- `guidance` (a string). What an example can't show about this section. The engine writes it into the scaffold before the section's heading.
- `examples` (a list of strings). Further occurrences of a section that declares `repeat: true`. Each is the whole markdown text of one occurrence: a heading of the section's level, its body, and any sections inside it. The scaffold writes the occurrence that it derives from the rules, then each text, in order and unchanged. A section that declares `for-each-key-of` occurs once for each key of another rule: the scaffold writes the occurrence of the first key from the rules, the occurrence of each later key from the next text, and the occurrences that remain after the last text from the rules. The engine checks each text as part of the scaffold.

## `prohibited-heading`

One heading that is banned inside open regions.

A `prohibited-heading` also accepts the keys of [`rule-common`](#rule-common).

- `heading` (a string, required). The exact heading text.
- `only` (a map of lifecycle dimensions to states, or a list of such maps). As for a section's `only`.

## `limit`

How many units, lines, or elements a part holds at most, and what the writer does with more. A `soft` limit is the usual length: the engine never checks it, and caps nothing by it, such as the number of units that a scaffold writes. A `hard` limit is checked, and a part that holds more is an error. The engine writes `when-at-limit` into the part's guidance line in the scaffold, after the part's own guidance and the words that the host gives for the limit. Where the host gives none, the engine writes the limit in the syntax of the schema, such as `` `hard: 9`, `when-at-limit`: ``. The engine also returns `when-at-limit` with the limit to a host that asks for the part's guidance. A finding on a part that holds more than a hard limit quotes `when-at-limit` after the engine's own words.

A `limit` has exactly one of `soft`, `hard`.

- `soft` (an integer). The number that the part usually holds at most. The engine never checks it.
- `hard` (an integer). The number that the part holds at most. A part that holds more is an error.
- `when-at-limit` (a string, required). What the writer does with the content that doesn't fit within the limit, such as "Put the detail in the sections that follow." or "Put each further list in a section of its own." Without such a text, the writer compresses the content to fit. An empty text is an error.

## `format-variant`

One named shape of a variant format.

A `format-variant` has exactly one of `text`, `template`, `pattern`.

- `text` (a string). The text that a line of this variant equals.
- `template` (a string). The line written with typed holes where it varies, matched whole.
- `pattern` (a regular expression). A regular expression that a line of this variant matches.
- `example` (a string, required). One conforming line that matches only this variant.

## `block`

One part of a body, read from the document's CommonMark blocks. Consecutive paragraphs form one prose block, and a paragraph of images only forms an image block. A block has a kind and units: the paragraphs of prose or a blockquote, the items of a list, the rows of a table, or the repetitions of a group. `each` applies one unit rule to every unit. `sequence` applies one rule per position and fixes the count. Otherwise, `min` and a hard `max` bound the count. Generated blocks, region interiors, footnote definitions, and HTML comments are exempt from matching. Every rule reads the text with its generated anchors removed. The engine reports on the schema a key that doesn't apply to the block's kind.

A `block` also accepts the keys of [`rule-common`](#rule-common).

- `kind` (a string, required). The kind of block. A `prose` block is paragraphs. A `bullet-list` or `numbered-list` block is items. A `table` block is a GFM table. A `code` block is a tagged fence. A `blockquote` block is quoted paragraphs. An `image` block is a paragraph of images only. An `html` block is an HTML block. A `thematic-break` block is a horizontal rule. A `references` block is link reference definitions at the top level of the body, outside the generated block. A `group` block is a repeating sequence of blocks. Values: `prose`, `bullet-list`, `numbered-list`, `table`, `code`, `blockquote`, `image`, `html`, `thematic-break`, `references`, `group`.
- `optional` (`true`, `false`, a map of lifecycle dimensions to states, or a list of such maps). Set to `true` to let the block be absent. Set to a condition, as for a section's `only`, to let the block be absent where the document's state matches the condition and require it elsewhere. When absent, the block is required.
- `repeat` (`true` or `false`). Set to `true` to let the block occur more than once in a row. When absent, it occurs once.
- `min` (an integer). The minimum number of units. When absent, the minimum is one.
- `max` ([`limit`](#limit)). The limit on the number of units, and what the writer does with more. When absent, there is no upper bound.
- `each` ([`unit`](#unit)). A rule that every unit must satisfy.
- `sequence` (a list of [`unit`](#unit)). One rule per unit, by position. The count is the length of the sequence unless an entry is optional or repeats. A unit must satisfy both `each` and the rule for its position.
- `rules` (a list of [`unit`](#unit)). The grammar of the block's units, an alternative to `each` and `sequence`. An entry with a shape and no `select-pattern` is a production. A production takes the units that its shape matches, and each unit goes to the first such entry in order. An entry with a `select-pattern` takes the units that the guard selects and checks them. If it states only a guard and a count, it counts those units and takes nothing. An entry with neither takes every unit. `min` and a hard `max` bound what each entry took or counted. A unit that no entry takes is an error unless `units-open` is set. The engine reports it against the production that it follows furthest.
- `units-open` (`true` or `false`). Set to `true` to accept a unit that no entry of `rules` takes. When absent, such a unit is an error. Requires `rules` beside it.
- `lang` (a string, or a list of them). The tag or tags that the fence must have, or `any`. A code block requires this key. An untagged fence is always an error. Applies to a block of kind `code`.
- `fence` (an integer). The least number of backticks of the fence that a scaffold writes, which no check reads, so that a block whose content holds a fence of three gets a longer one. When absent, the least is three. Applies to a block of kind `code`.
- `columns` (a list of strings). The header row, in order. A table block requires this key. A differing header is an error. Applies to a block of kind `table`.
- `columns-open` (`true` or `false`). Set to `true` to accept further columns after the declared ones. Applies to a block of kind `table`.
- `example-columns` (a list of strings). The header row of the scaffold of a table block that declares `columns: []` and `columns-open: true`, whose columns vary from document to document. The scaffold writes it before the rows of the example, with each `|` in a name escaped, and each row that the scaffold writes has one cell for each of its columns. The list names at least one column. Applies to a block of kind `table`.
- `source-pattern` (a regular expression). A regular expression that each image path must match. The engine matches an image in the host's reference scheme by what the host reports that it shows, and any other image as written. Applies to a block of kind `image`.
- `inline-html` (`true` or `false`). Set to `true` to accept inline HTML in this block's units. When absent, any HTML other than a comment is an error.
- `label` (a string). Whether each unit of the block carries one of the schema's `labels`: `required` for a label on every unit, or `optional` for a label where the writer gives one. A unit rule of the block that declares no `label` takes this one. When absent, a unit carries a label only where its unit rule declares `label`. Values: `required`, `optional`. Applies to a block of kind `prose`, `blockquote`, `bullet-list`, `numbered-list`, `table`.
- `blocks` (a list of [`block`](#block)). The blocks of one repetition of the group. Applies to a block of kind `group`.
- `rule-id` (a slug). The name by which other rules reference this block. Its keys are the keys that its units capture, except the keys of a unit rule that declares `key-of`.
- `only` (a map of lifecycle dimensions to states, or a list of such maps). As for a section's `only`.
- `example` (a string, or a list of them). The text of one of this block's units, or a list of such texts. The engine uses it for every unit whose rule has no text or example of its own. A list gives those units in order, as the list of a unit rule does.
- `guidance` (a string). What an example can't show about this block. The engine writes it into the scaffold before the block.

## `heading`

A section's heading as a unit rule. It defines the heading's shape, the key that it captures, and an example of the heading. A repeated section is cited by its key.

A `heading` has exactly one of `text`, `template`, `pattern`.

- `text` (a string, or `{ from: <frontmatter key> }`). The exact heading text, or `{ from: <key> }` for the frontmatter value.
- `template` (a string). The heading written with typed holes where it varies, matched whole.
- `pattern` (a regular expression). A regular expression that the heading must match, for what a template can't express. Write `^` and `$` to bind the ends.
- `key` (a string). The hole of the template, or the named group of the pattern, whose capture is the section's key.
- `unique` (a string). The scope within which the key is unique. When absent, the scope is `document`. Values: `document`. Requires `key` beside it.
- `key-derives-from` ([`key-derivation`](#key-derivation)). Where the key's prefix must come from. The key must start with a prefix that the derivation allows, followed by the separator. Requires `key` beside it.
- `example` (a string). A heading of the kind that the template or the pattern requires.

## `unit`

A rule on one unit of a block: a paragraph, an item, a row, a line of a fence, or a repetition of a group. The engine matches a paragraph with its lines joined by one space, and a line of a fence without its indent. It reads a paragraph, an item, a row, and a heading in one fixed spelling of their markup, as it reads the shape: each emphasis as `_x_`, each strong emphasis as `**x**`, each code span in its shortest backticks, and each needless escape as the plain character. So a pattern names markup in that spelling. A line of a fence reads as written. It matches an item's own text. Anything more that the item contains, such as a nested list or a fence, must match the rule's `blocks`. A rule has at most one shape: `text`, `template`, `pattern`, or the `format` that it binds.

A `unit` also accepts the keys of [`rule-common`](#rule-common).

- `text` (a string, or `{ from: <frontmatter key> }`). The text that the unit must equal: a string, or `{ from: <key> }` for the frontmatter value. It applies to a paragraph, an item, or a line.
- `template` (a string). The unit's text written with typed holes where it varies, matched whole.
- `pattern` (a regular expression). A regular expression that the unit's text must match, for what a template can't express. Write `^` and `$` to bind the ends.
- `format` (the name of a format). The name of a line format that the unit's text must match.
- `key` (a string). The hole of the template, the named group of the pattern, or the hole or group of every shape of the format, whose capture is the unit's key. The key must be unique within its scope by its slug, and the slug names the key's generated anchor. Don't declare `key` beside a format that captures a key.
- `key-derives-from` ([`key-derivation`](#key-derivation)). Where the key's prefix must come from. The key must start with a prefix that the derivation allows, followed by the separator.
- `key-optional` (`true` or `false`). Set to `true` to treat a unit that contains no key as plain rather than as an error. The key is the rule's own `key` or the one that its `format` declares. The engine reports a rule that has neither. When absent, a unit with no key is an error.
- `key-repeats` (`true` or `false`). Set to `true` to let the rule capture one key on several units, and a key that another rule captures, as a list that names the owner of each item names one owner many times. Each capture by this rule refers to the key, which a capture by a rule without `key-repeats` defines where there is one. When absent, a second capture of a key is an error.
- `unique` (a string). The scope within which the key is unique. When absent, the scope is `document`. The engine reports a clash on both documents, and names the key and the other document. Values: `document`.
- `rule-id` (a slug). The name of this rule. The rule's findings name the rule by it, and so do the words that a host writes before the `when-at-limit` of its limit. The name holds no keys, and no other rule references it, so several unit rules can share it. A rule under `rules` or `lines` that declares `max` and binds no format must declare one. Name it after what the rule counts, such as `wait` for a rule that selects the items that begin with `wait`.
- `key-of` (a slug). The name of the set that the keys of this rule belong to, which a `for-each-key-of` or a `key-derives-from` references. Several rules can name one set, and a set that shares its name with the `rule-id` of a section or a block is the set of that section or block. When absent, the keys are filed under the `rule-id` of the enclosing block, else of its group, else of its section. The rule must capture a key, by `key` or by its format.
- `for-each-key-of` (the name of a set of keys, or a list of them). Requires one unit that captures each key filed under the referenced name, each key once, as a section's `for-each-key-of` pairs headings with keys. The name is the `rule-id` of a section or a block, or a `key-of`. A list of names pairs with the keys of any of them, each key once. This rule must capture a key, by `key` or by its format, and the referenced name must hold one. A missing key and an orphan are each an error that names the key. When the section declares a `parent`, the keys are those captured inside the same occurrence of the parent. Otherwise, they are the keys of the document. The engine reports a missing key even where the block that would contain it is absent.
- `prohibited-patterns` (a list of [`prohibition`](#prohibition)). Patterns that the unit's text must not match, each with its message. Code spans are excluded, as they are from the schema's patterns: the patterns read each one as a run of backticks of the same length, so they match no word inside it. A line of a fence or of an HTML block holds no code span, so the patterns read all of it.
- `label` (a string). Whether each unit that this rule governs carries one of the schema's `labels`, as for a block's `label`, in place of the block's. When absent, the rule takes the `label` of its block, and a position of a `sequence` takes the `label` of the block's `each` before it. A line of a fence carries no label. Values: `required`, `optional`. Applies to a block of kind `prose`, `blockquote`, `bullet-list`, `numbered-list`, `table`.
- `optional` (`true` or `false`). In a sequence, set to `true` to let this position be absent. When absent, the position is required.
- `repeat` (`true` or `false`). In a sequence, set to `true` to let this position occur more than once. When absent, it occurs at most once.
- `blocks` (a list of [`block`](#block)). The blocks that a list item contains beyond its own text. Each block is required in every item unless it sets `optional`. When absent, an item contains nothing beyond its text.
- `lines` (a list of [`unit`](#unit)). The grammar of the content lines of a code or `html` block, as `rules` is the grammar of a block's units. Each line, without its indent, is a unit of its own. Blank lines don't count. A line that no rule takes is an error unless `lines-open` is set. A rule here can't contain `lines` or `blocks`. A key captured from a line is filed under the line rule's `key-of`, or, where the rule declares none, under the enclosing block's `rule-id`. A rule that contains `lines` can also state an `example`. That example is the whole fence: the scaffold copies it, and these rules read its keys out of it.
- `lines-open` (`true` or `false`). Set to `true` to accept a line that no rule of `lines` takes. When absent, such a line is an error. Requires `lines` beside it.
- `select-pattern` (a regular expression). Under `rules` or `lines`, a regular expression that selects the units or lines that this rule governs by their text, for a rule that picks them by something other than its shape. A rule that states only a guard and a count counts what it matches and takes nothing.
- `min` (an integer). Under `rules` or `lines`, the fewest units or lines that the rule may take or count. A rule that declares `select-pattern` must state `min`, because it has no default. Write `min: 0` to govern such units without requiring one. When absent on a rule without a guard, the minimum is one.
- `max` ([`limit`](#limit)). Under `rules` or `lines`, the limit on the units or lines that the rule may take or count, and what the writer does with more. When absent, there is no upper bound. A rule with a `max` and no `format` must declare a `rule-id`.
- `example` (a string, or a list of them). The text of a unit that this rule governs, or a list of such texts. A rule that binds a format and states no example of its own uses the format's example. The engine checks each text of a list against the rule, and a failure quotes the first. The scaffold writes one unit for each text of a list, in order, and copies of the last text while `min` demands more, but never more units than the hard `max` of the rule or of the block. A rule under `rules` writes its list even with `min: 0`. In a `sequence`, a position that declares `repeat` writes the whole list, and any other position writes the first text. In a `code` block, each text is one fence, so the block declares `repeat: true`.

## `key-derivation`

Where the prefixes of a `key-derives-from` come from, and the separator that ends them.

A `key-derivation` has exactly one of `rule`, `from`.

- `rule` (the name of a set of keys). The name of the set of keys that are the allowed prefixes. In a markdown schema, it is the `rule-id` of a section or a block, or a `key-of`.
- `from` (a string). The frontmatter key whose value is the only allowed prefix.
- `separator` (a string, required). The literal text that ends the prefix. A key derives from an allowed prefix that it starts with, followed by the separator, and from the longest where more than one starts it, so a prefix can contain the separator.

## `rule-common`

Keys that every rule map with `common: true` accepts.

- `severity` (a string). How the engine reports a failure of the rule: `error` reports an error, and `warn` reports a warning. A rule inside this one, such as a block's unit rule, takes this severity unless it declares its own. When absent, the rule takes the severity of the rule it sits in, and a failure of a rule in no other is an error. Values: `error`, `warn`.
