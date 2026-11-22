---
type: TramlinesMdSpecification
id: tramlines-md-spec
title: tramlines-md markdown format specification
description: "The markdown format of tramlines: how the engine reads a markdown document, what its schema says about the body, and its keys and anchors, its scaffold, and its repairs."
version: "0.1"
status: draft
---

# tramlines-md markdown format specification

This specification defines tramlines-md: the markdown format of tramlines, what a schema specifies about the body of a markdown document, and how the engine checks it.

The [core specification][sokf:tramlines-spec] defines the grammar, the schema keys that every format shares, the shapes, the formats, the findings, and the command. This specification adds what only a markdown document has: headings and sections, blocks and their units, anchors, and markers in HTML comments. Its grammar, `tramlines-md.grammar.yaml`, names the core grammar as its base.

## About this specification

Each normative rule of this specification is an EARS criterion, keyed `TLMD_n`.

## 1. Terminology

- <a id="document"></a>`document` - A UTF-8 markdown text, with an optional YAML frontmatter block and a body.
- <a id="section"></a>`section` - One heading of a body and the blocks under it, to the next heading of any level.
- <a id="preamble"></a>`preamble` - The content of a body before its first heading.
- <a id="block"></a>`block` - One part of a body, of one kind: prose, a list, a table, a fence, a quote, images, HTML, a rule, link references, or a group.
- <a id="unit"></a>`unit` - One paragraph, item, row, line of a fence, or repetition of a group: the part of a block that one unit rule checks.
- <a id="line-rule"></a>`line rule` - A rule on one content line of a code or HTML block.
- <a id="position"></a>`position` - One entry of the blocks that a section declares, which claims the blocks of its kind.
- <a id="open-region"></a>`open region` - The headings after a section that declares `headings-open` or `subheadings-open`, up to the next declared heading.
- <a id="dialect"></a>`dialect` - How a host spells its markers: a namespace, and the regions, trailer, and scheme that it adds.
- <a id="marker"></a>`marker` - An HTML comment on a line of its own, or a YAML comment in the frontmatter, in the namespace of the dialect.
- <a id="region"></a>`region` - A block that the markers of the host open and close, whose interior the host generates.
- <a id="trailer"></a>`trailer` - The generated block of link reference definitions that ends a document.
- <a id="slug"></a>`slug` - The text in lower case, with letters, digits, hyphens, and underscores kept, other characters dropped, and spaces made hyphens.
- <a id="anchor"></a>`anchor` - The empty HTML link `<a id="<slug>"></a>` that marks where a key is described.
- <a id="pairing"></a>`pairing` - A section or a unit rule that declares `for-each-key-of`, with one occurrence for each key of another rule.
- <a id="scaffold"></a>`scaffold` - The document that a schema derives for one combination of states, which a new document starts from.
- <a id="label"></a>`label` - Words in square brackets that mark a unit, the words that the host gives for one of the labels that the schema declares, or its key.

## 2. Documents

A document opens with frontmatter when its first line is `---`. The body is CommonMark, with GitHub tables, footnotes, and task lists. The engine reads the body as blocks and units.

- <a id="tlmd_1"></a>TLMD_1 [human] [event]: WHEN the first line of a document is `---`, the engine SHALL read the lines up to the next `---` line as frontmatter and the lines after it as the body.
- <a id="tlmd_2"></a>TLMD_2 [human] [unwanted]: IF the frontmatter has no closing `---` line, THEN the engine SHALL read the frontmatter to the end of the text, with an empty body.
- <a id="tlmd_3"></a>TLMD_3 [human] [ubiquitous]: the engine SHALL read a text with CRLF line endings or with carriage returns alone as LF.
- <a id="tlmd_4"></a>TLMD_4 [human] [ubiquitous]: the engine SHALL read the body as CommonMark, with GitHub tables, footnotes, and task lists.
- <a id="tlmd_5"></a>TLMD_5 [human] [ubiquitous]: the engine SHALL read consecutive paragraphs as one prose block, and a paragraph of images only as an image block.
- <a id="tlmd_6"></a>TLMD_6 [human] [ubiquitous]: the engine SHALL read a paragraph with its lines joined by one space, and a line of a fence without its indent.
- <a id="tlmd_7"></a>TLMD_7 [human] [ubiquitous]: the engine SHALL match the item's own text, and check what the item contains beyond that text, a nested list or a fence, against the `blocks` of its rule.
- <a id="tlmd_162"></a>TLMD_162 [human] [unwanted]: IF an item contains a block after its first paragraph, such as a nested list, a fence, a blockquote, or another paragraph, or in place of it on the item's own line, and its rule declares no `blocks`, THEN the engine SHALL report a finding on the first line of that block, and hold no text of the item to its rule where the block stands in place of the text.
- <a id="tlmd_173"></a>TLMD_173 [agent] [ubiquitous]: the engine SHALL read a heading inside a list item or a blockquote as a block of that item or quote, of a kind that no schema declares, with its text read without the marker of the item or the quote that holds it, and only a heading outside both as a heading of the document.
- <a id="tlmd_172"></a>TLMD_172 [human] [unwanted]: IF a blockquote contains a heading, THEN the engine SHALL report a finding that names the heading, and count no paragraph missing from a quote that holds only headings.
- <a id="tlmd_8"></a>TLMD_8 [human] [ubiquitous]: the engine SHALL record the position of every code span and every code block of a body, at any depth of quotes, lists, and footnotes, and whether each code block is fenced.
- <a id="tlmd_9"></a>TLMD_9 [human] [invariant]: FOR ANY line, the engine SHALL produce a masked line with the length and the line number of the original, and change no byte outside a recorded span.

A body link that begins with `/` names a path from the root of the repository. The path of a link that is no URL is its text before the first `?` or `#`. A trailing `/` marks a path that names a directory, as it does in a shell, and the host, which reads the repository, checks that mark.

- <a id="tlmd_201"></a>TLMD_201 [human] [event]: WHEN the path of a `/`-rooted body link has a character other than `/`, ends with one or more `/` characters, and has no empty, `.`, or `..` segment before them, the engine SHALL classify the link as the repository path without those characters, marked as a path that names a directory.
- <a id="tlmd_202"></a>TLMD_202 [human] [event]: WHEN the path of a `/`-rooted body link does not end with `/` and has no empty, `.`, or `..` segment, the engine SHALL classify the link as a repository path that is not marked as naming a directory.
- <a id="tlmd_203"></a>TLMD_203 [human] [unwanted]: IF the path of a `/`-rooted body link has a character other than `/` and has an empty, `.`, or `..` segment before any trailing `/` characters, THEN the engine SHALL classify the link as an invalid repository path.
- <a id="tlmd_204"></a>TLMD_204 [human] [event]: WHEN a body link names a `/`-rooted path that ends with `/` and has a fragment, the engine SHALL strip the `/` characters before the fragment and keep the fragment.
- <a id="tlmd_214"></a>TLMD_214 [human] [event]: WHEN the path of a body link is made only of `/` characters, the engine SHALL classify the link as the root of the repository, marked as a path that names a directory.
- <a id="tlmd_215"></a>TLMD_215 [human] [event]: WHEN a body link that is no URL has a `?` before any `#`, the engine SHALL leave out of the link's path the query, the text from that `?` to the `#` or the end, and keep the fragment.

## 3. Dialect and markers

The engine has three markers of its own, each in the namespace `<ns>` of the dialect:

- `<!-- <ns>:only k=v,v2 -->` - The part after it belongs to those states. In the marker of a condition with more than one map, ` | ` separates the maps, as in `<!-- <ns>:only k=v | k2=v2 -->`.
- `<!-- <ns>:guidance … -->` - What an example cannot say, before the part that it guides. Before a frontmatter key, the marker is `# <ns>:guidance …`.
- `<!-- <ns>:optional -->` - With `<!-- /<ns>:optional -->`, a part that the schema does not demand.

A host adds to its dialect a region, a trailer, or a scheme.

- <a id="tlmd_10"></a>TLMD_10 [human] [unwanted]: IF a governed document contains an `only` marker, THEN the engine SHALL report a finding on the line of the marker.
- <a id="tlmd_158"></a>TLMD_158 [human] [unwanted]: IF a governed document contains a `guidance` marker, THEN the engine SHALL report a finding on the line of the marker.
- <a id="tlmd_11"></a>TLMD_11 [human] [unwanted]: IF a governed document contains an `optional` region, both of its markers, THEN the engine SHALL report a warning.
- <a id="tlmd_12"></a>TLMD_12 [human] [unwanted]: IF an `optional` marker has no partner, THEN the engine SHALL report a finding.
- <a id="tlmd_171"></a>TLMD_171 [human] [ubiquitous]: the engine SHALL count toward `line-limit` no marker line outside a fence, and no blank line that sets one apart, so that a scaffold is held to the length of the document that it leaves.
- <a id="tlmd_13"></a>TLMD_13 [human] [optional]: WHERE the dialect names a region, the engine SHALL read the lines from `<!-- <ns>:<region> target -->` to `<!-- /<ns>:<region> -->` as one block of the kind that the dialect names, and check its interior with no rule.
- <a id="tlmd_14"></a>TLMD_14 [human] [optional]: WHERE the dialect names a trailer, the engine SHALL read `<!-- <ns>:<trailer> -->` and the link reference definitions after it as a generated block that no rule reads.
- <a id="tlmd_15"></a>TLMD_15 [human] [optional]: WHERE the dialect names a scheme, the engine SHALL preserve the rest of a link label `<scheme>:<rest>` as written, for the host to resolve.
- <a id="tlmd_16"></a>TLMD_16 [human] [ubiquitous]: the engine SHALL read a marker inside a fenced code block as text.
- <a id="tlmd_17"></a>TLMD_17 [human] [ubiquitous]: the engine SHALL assign the default dialect the namespace `tramlines`, and no region, trailer, or scheme.

## 4. Sections and headings

A schema lists the headings of a document under `sections`, in document order. The body of a section runs to the next heading of any level. Sections nest by level, except that a section that declares `parent` is inside its parent.

- <a id="tlmd_18"></a>TLMD_18 [human] [unwanted]: IF a declared section that is not optional at the states of the document is missing from the document, or from an occurrence of its parent where it declares `parent`, THEN the engine SHALL report a finding that names the section.
- <a id="tlmd_229"></a>TLMD_229 [human] [optional]: WHERE a section or a block declares a condition as its `optional`, the engine SHALL treat the part as optional in a document whose state matches the condition, and as required in any other.
- <a id="tlmd_230"></a>TLMD_230 [human] [unwanted]: IF a document holds no state that the lifecycle declares for a dimension, THEN the engine SHALL treat each section and each block that some state of the dimension would admit as declared, and as required only where its `only` matches the states that the document holds and no state of the dimension would make it optional.
- <a id="tlmd_151"></a>TLMD_151 [human] [unwanted]: IF the heading of a declared section that does not declare `repeat` occurs more than once in the document, or in one occurrence of its parent where it declares `parent`, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_19"></a>TLMD_19 [human] [unwanted]: IF a heading matches no section and no open region admits it, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_20"></a>TLMD_20 [human] [unwanted]: IF a heading is more than one level deeper than the heading before it, matches no section or matches a section at the level that the section declares, and does not lie outside every occurrence of the parent of its section, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_21"></a>TLMD_21 [human] [unwanted]: IF the sections of a schema that orders them skip a level, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_22"></a>TLMD_22 [agent] [unwanted]: IF the heading of a declared section follows a heading whose section the schema lists after it, the heading does not repeat where its section does not declare `repeat`, and the schema does not set `sections-ordered: false`, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_144"></a>TLMD_144 [human] [unwanted]: IF a heading matches a section at another level than the section declares, THEN the engine SHALL report a finding that names the level.
- <a id="tlmd_23"></a>TLMD_23 [human] [ubiquitous]: the engine SHALL assign a heading that an exact heading and a varying heading both match to the exact one.
- <a id="tlmd_175"></a>TLMD_175 [human] [ubiquitous]: the engine SHALL match a heading against the varying headings of the sections of the heading's own level before it matches the heading against the varying heading of a section of another level.
- <a id="tlmd_24"></a>TLMD_24 [human] [optional]: WHERE a section declares `headings-open`, the engine SHALL accept undeclared headings of its level or deeper after it, up to the next declared heading, and check their bodies against only the rules of the document scope.
- <a id="tlmd_25"></a>TLMD_25 [human] [optional]: WHERE a section declares `subheadings-open`, the engine SHALL accept undeclared headings deeper than its level after it, up to the next declared heading.
- <a id="tlmd_26"></a>TLMD_26 [human] [unwanted]: IF a heading in an open region has the exact text of a prohibited heading, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_170"></a>TLMD_170 [human] [unwanted]: IF `sections-prohibited` names a heading that a declared section matches in some states that both admit, THEN the engine SHALL report a finding on the schema that names the heading.
- <a id="tlmd_27"></a>TLMD_27 [human] [ubiquitous]: the engine SHALL accept on the heading of a section the keys that a unit rule accepts for one line: a shape, `key`, `unique`, `key-derives-from`, and `example`.
- <a id="tlmd_28"></a>TLMD_28 [human] [ubiquitous]: the engine SHALL accept a heading key equal to a key that a rule named in the `for-each-key-of` of the section captures.
- <a id="tlmd_29"></a>TLMD_29 [human] [unwanted]: IF two sections or two units that one `for-each-key-of` pairs to a rule capture the same key in one scope, other than a capture in the body of a heading that repeats where its section does not, of a key that the body it repeats captures before it, THEN the engine SHALL report a finding on the second that names the key and the line of the first.
- <a id="tlmd_30"></a>TLMD_30 [human] [ubiquitous]: the engine SHALL check each pairing for a repeated key on its own, in each scope that the engine reads it in.
- <a id="tlmd_31"></a>TLMD_31 [human] [unwanted]: IF a heading of a section that declares `parent` follows no occurrence of the parent before the next heading of the level of the parent or higher, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_32"></a>TLMD_32 [human] [optional]: WHERE a section declares `parent`, the engine SHALL count its `optional` and its `repeat` within each occurrence of the parent.
- <a id="tlmd_166"></a>TLMD_166 [human] [unwanted]: IF a heading matches a section whose `parent` does not apply at the states of the document, THEN the engine SHALL report a finding that names the heading and the parent.
- <a id="tlmd_33"></a>TLMD_33 [human] [ubiquitous]: the engine SHALL accept a heading of a repeating section after the headings of the sections inside it.
- <a id="tlmd_34"></a>TLMD_34 [human] [unwanted]: IF a `parent` names no section, THEN the engine SHALL report a finding on the schema that names it.
- <a id="tlmd_156"></a>TLMD_156 [human] [unwanted]: IF a `parent` names a section of a level that is not smaller than the level of the section that declares it, THEN the engine SHALL report a finding on the schema that names both levels.
- <a id="tlmd_157"></a>TLMD_157 [human] [unwanted]: IF a section that declares `parent` is not listed after its parent with only the other sections inside that parent between, THEN the engine SHALL report a finding on the schema that names the section.
- <a id="tlmd_35"></a>TLMD_35 [human] [unwanted]: IF a section declares `number-from-parent` and its heading does not begin with the number of the heading of its parent, a dot and a number, THEN the engine SHALL report a finding that names the expected number.
- <a id="tlmd_36"></a>TLMD_36 [human] [unwanted]: IF the heading of the parent of a section that declares `number-from-parent` has no number, THEN the engine SHALL report a finding that names that heading.
- <a id="tlmd_37"></a>TLMD_37 [human] [optional]: WHERE a schema declares a `preamble`, the engine SHALL check the blocks before the first heading of a document against the `blocks` of the preamble.
- <a id="tlmd_38"></a>TLMD_38 [human] [unwanted]: IF a body has content before its first heading and the schema declares no `preamble`, THEN the engine SHALL report a finding.

## 5. Blocks

A section declares its body as an ordered list of blocks, each of one kind with rules for its units. A position claims the blocks of its kind. Where a position refuses a block, the engine offers the block to the positions after it.

- <a id="tlmd_39"></a>TLMD_39 [human] [unwanted]: IF no declared position claims a block of a body, in order unless the section sets `blocks-ordered: false`, THEN the engine SHALL report a finding that names the kind of the block.
- <a id="tlmd_152"></a>TLMD_152 [human] [unwanted]: IF a position that is not optional at the states of the document claims no block and no block of another kind stands where it expects one, THEN the engine SHALL report a finding that names the kind that it expects.
- <a id="tlmd_40"></a>TLMD_40 [human] [optional]: WHERE a section declares no `blocks`, the engine SHALL read its body as one prose block of one or more paragraphs.
- <a id="tlmd_41"></a>TLMD_41 [human] [ubiquitous]: the engine SHALL exempt generated blocks, region interiors, footnote definitions, and HTML comments from block matching.
- <a id="tlmd_42"></a>TLMD_42 [human] [unwanted]: IF a block declares a key that does not apply to its kind, or a unit rule declares a key that does not apply where the rule sits, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tlmd_153"></a>TLMD_153 [human] [unwanted]: IF a block lacks a key that its kind requires, `lang` for a `code` block, `columns` for a `table` block, or `blocks` for a `group` block, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tlmd_43"></a>TLMD_43 [human] [unwanted]: IF a fence has no tag, or a tag that the `lang` of its block does not list, THEN the engine SHALL report a finding.
- <a id="tlmd_145"></a>TLMD_145 [human] [unwanted]: IF a fence has fewer backticks than the `fence` of its block, THEN the engine SHALL report a finding that names both numbers.
- <a id="tlmd_44"></a>TLMD_44 [human] [unwanted]: IF the header row of a table differs from the declared `columns`, beyond further columns that `columns-open` accepts, THEN the engine SHALL report a finding.
- <a id="tlmd_207"></a>TLMD_207 [human] [unwanted]: IF a table block declares `example-columns` that names no column, or declares it beside a non-empty `columns` or without `columns-open: true`, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tlmd_45"></a>TLMD_45 [human] [unwanted]: IF a unit contains HTML other than a comment and its block does not set `inline-html`, THEN the engine SHALL report a finding.
- <a id="tlmd_46"></a>TLMD_46 [human] [ubiquitous]: the engine SHALL match the path of an image or the target of a region against a `source-pattern` by what the host reports that it shows where the path is in the scheme of the host, and as written otherwise.
- <a id="tlmd_163"></a>TLMD_163 [human] [unwanted]: IF the path of an image or the target of a region does not match the `source-pattern` of its block, THEN the engine SHALL report a finding that names the path and the pattern.
- <a id="tlmd_200"></a>TLMD_200 [human] [unwanted]: IF the host reports that an image or the target of a region names a document, THEN the engine SHALL report the finding of [TLMD_163](#tlmd_163) whatever the `source-pattern` of its block, with a message that says that it names a document.
- <a id="tlmd_47"></a>TLMD_47 [human] [unwanted]: IF a block declares `rules` beside `each` or beside `sequence`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_146"></a>TLMD_146 [agent] [unwanted]: IF a block has fewer units than its `min`, or a group fewer repetitions, THEN the engine SHALL report a finding that names the bound and the count.
- <a id="tlmd_160"></a>TLMD_160 [agent] [unwanted]: IF a block has more units than the count of a hard `max`, or a group more repetitions, THEN the engine SHALL report a finding that names the bound and the count, and quotes the `when-at-limit` of the `max`, as [TL_77][sokf:tramlines-spec#tl_77] states.
- <a id="tlmd_147"></a>TLMD_147 [human] [unwanted]: IF a position of a `sequence` that does not declare `optional` takes no unit, THEN the engine SHALL report a finding that names the position.
- <a id="tlmd_161"></a>TLMD_161 [human] [unwanted]: IF units of a block that declares `sequence` remain after its last position, THEN the engine SHALL report a finding on each of them.
- <a id="tlmd_48"></a>TLMD_48 [human] [ubiquitous]: the engine SHALL let consecutive declared prose rules take successive runs of units from one prose block.
- <a id="tlmd_49"></a>TLMD_49 [human] [ubiquitous]: the engine SHALL divide a run between these prose rules by the text of its units alone, and by no `min` or `max`.
- <a id="tlmd_50"></a>TLMD_50 [human] [unwanted]: IF units of a prose, bullet list, or numbered list block remain after the last position that takes a run of them, THEN the engine SHALL report a finding against the first unit that remains.
- <a id="tlmd_51"></a>TLMD_51 [human] [ubiquitous]: the engine SHALL divide a prose run only under ordered block matching.
- <a id="tlmd_52"></a>TLMD_52 [human] [ubiquitous]: the engine SHALL apply each entry of the `rules` of a block that declares a guard to the units that match that guard.
- <a id="tlmd_53"></a>TLMD_53 [human] [unwanted]: IF no entry of `rules` takes a unit, the unit matches no part of a rule with a shape, and the block declares no `units-open`, THEN the engine SHALL report a finding that names the unit.
- <a id="tlmd_54"></a>TLMD_54 [human] [ubiquitous]: the engine SHALL bound by the `min` and the hard `max` of an entry the units that the entry selected.
- <a id="tlmd_55"></a>TLMD_55 [human] [ubiquitous]: the engine SHALL assign a unit to the first rule with a shape and no guard whose shape the unit has.
- <a id="tlmd_56"></a>TLMD_56 [human] [ubiquitous]: the engine SHALL count, and not take, the units that a rule matches when that rule declares only a guard and a count.
- <a id="tlmd_57"></a>TLMD_57 [human] [unwanted]: IF no rule takes a unit or a line that partly matches a rule with a shape, and the block declares no `units-open` or its unit rule no `lines-open`, THEN the engine SHALL report the unit against the rule that it matches furthest, with the column where it stops.
- <a id="tlmd_58"></a>TLMD_58 [human] [unwanted]: IF an earlier rule with a shape and no guard takes the example of a later one, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_59"></a>TLMD_59 [human] [ubiquitous]: the engine SHALL claim a block for an optional, repeating, or grouped position only where that position reports no error against the block, other than one that reports more units than a hard `max` above zero allows, or a fault of a label that [TLMD_249](#tlmd_249) to [TLMD_251](#tlmd_251) report.
- <a id="tlmd_60"></a>TLMD_60 [human] [ubiquitous]: the engine SHALL claim a block for a required position that does not repeat, whatever that position reports against the block.
- <a id="tlmd_61"></a>TLMD_61 [human] [unwanted]: IF the engine cannot complete a group repetition, THEN the engine SHALL release every block that the repetition claimed, and where the group declares `optional`, has a `min` above 0, and holds no other repetition, give why the repetition failed as the reason that the group refused its blocks.
- <a id="tlmd_62"></a>TLMD_62 [human] [ubiquitous]: the engine SHALL apply one claim test whether a section fixes the block order or frees it.
- <a id="tlmd_63"></a>TLMD_63 [human] [ubiquitous]: the engine SHALL offer a block that a position released to the positions after it that declare its kind.
- <a id="tlmd_64"></a>TLMD_64 [human] [ubiquitous]: the engine SHALL discard the findings, captures, repairs, and gates of a position that it releases.
- <a id="tlmd_65"></a>TLMD_65 [human] [unwanted]: IF a position reports only warnings against a block, THEN the engine SHALL claim the block.
- <a id="tlmd_66"></a>TLMD_66 [human] [unwanted]: IF a position refuses a block of its own kind and then reports itself missing, THEN the engine SHALL report why the position refused the block.
- <a id="tlmd_67"></a>TLMD_67 [human] [unwanted]: IF no position claims a block that a position refused, THEN the engine SHALL report why the last position that tried the block refused it.
- <a id="tlmd_68"></a>TLMD_68 [human] [event]: WHEN a block of a kind that no later position declares appears before the block that a position declares, the engine SHALL move the position past it, as a block that no position claims.
- <a id="tlmd_169"></a>TLMD_169 [human] [unwanted]: IF the block of a position that is no group and is not optional at the states of the document stands after a block that a later position declares, and no later position declares the block of the position, THEN the engine SHALL report a finding that names the block, and check the block against its position.
- <a id="tlmd_209"></a>TLMD_209 [human] [unwanted]: IF a block fails by shape, as [TLMD_225](#tlmd_225) defines, the position that claims it whatever it reports, and has the shape, as [TLMD_224](#tlmd_224) defines, of an earlier optional position that is no group and claimed no block, THEN the engine SHALL report the finding of [TLMD_169](#tlmd_169) as an error at the block, and check the block against that earlier position.
- <a id="tlmd_210"></a>TLMD_210 [human] [event]: WHEN the engine reports a block under [TLMD_209](#tlmd_209), the engine SHALL offer the block after it to the position that the block failed.
- <a id="tlmd_211"></a>TLMD_211 [human] [invariant]: FOR ANY section that fixes the order of its blocks and any blocks of its body, the engine SHALL report an error against the blocks exactly where it reports one without [TLMD_209](#tlmd_209) and [TLMD_210](#tlmd_210).
- <a id="tlmd_224"></a>TLMD_224 [human] [ubiquitous]: the engine SHALL count a block as having the shape of a position for [TLMD_209](#tlmd_209) only where a unit rule of the position declares a template, a pattern, or a format, and the text of each unit of the block matches the template or the pattern, and a shape of the format, of one such unit rule.
- <a id="tlmd_225"></a>TLMD_225 [human] [ubiquitous]: the engine SHALL count a block as failing a position by shape for [TLMD_209](#tlmd_209) only where a unit rule of the position declares a template, a pattern, or a format, and the text of a unit of the block has the shape of no such unit rule, whatever the bans, the keys, and the counts of the position report.
- <a id="tlmd_226"></a>TLMD_226 [human] [event]: WHEN the block of [TLMD_209](#tlmd_209) is a run that the positions divide, the engine SHALL read as the block the longest run of units at the start of the run, as the run stood before the failed position took it, that each have the shape of the earlier position, and offer the rest of the run to the failed position.
- <a id="tlmd_227"></a>TLMD_227 [human] [event]: WHEN a position that declares `repeat` takes a block under [TLMD_209](#tlmd_209), the engine SHALL keep the position as one that claimed no block for the blocks after it.
- <a id="tlmd_69"></a>TLMD_69 [human] [unwanted]: IF a position meets a block of another kind and no block of its own kind follows, THEN the engine SHALL name the expected kind and the found kind in one finding.
- <a id="tlmd_70"></a>TLMD_70 [human] [unwanted]: IF a finding names the found kind, THEN the engine SHALL claim that block for the position that named it.
- <a id="tlmd_71"></a>TLMD_71 [agent] [unwanted]: IF a repetition of a group fails where the group needs it to reach its `min`, it is not the first repetition of a group that declares `optional`, and no later position takes its opening block with no error, where no later position takes the kind of its opening block, or where the group's own rule accepted the units that the repetition's opening position took, THEN the engine SHALL report why that repetition failed and claim its blocks for the group.
- <a id="tlmd_72"></a>TLMD_72 [human] [unwanted]: IF a block fits an inner block of a group after the first and no later position takes it, THEN the engine SHALL read it as a repetition that has no opening blocks and name the expected kind.
- <a id="tlmd_73"></a>TLMD_73 [human] [ubiquitous]: the engine SHALL claim for a group every block that the group took, whatever the group reports about them.
- <a id="tlmd_74"></a>TLMD_74 [human] [ubiquitous]: the engine SHALL assign to a position over a run the longest extent of units that it reports no error against, other than one that reports more units than a hard `max` above zero allows, or a fault of a label that [TLMD_249](#tlmd_249) to [TLMD_251](#tlmd_251) report.
- <a id="tlmd_75"></a>TLMD_75 [human] [unwanted]: IF no extent of a run satisfies a position that claims a block whatever it reports, THEN the engine SHALL assign to that position the whole run, less the units that [TLMD_193](#tlmd_193) leaves to a later group.
- <a id="tlmd_76"></a>TLMD_76 [human] [ubiquitous]: the engine SHALL choose an extent only for a prose, bullet list, or numbered list block.
- <a id="tlmd_77"></a>TLMD_77 [human] [ubiquitous]: the engine SHALL choose an extent only where a section fixes the block order.
- <a id="tlmd_78"></a>TLMD_78 [human] [ubiquitous]: the engine SHALL choose an extent only where a section declares two or more positions of the kind of the block, counting a group that opens with a position of that kind as one more position, whatever closes the group, and counting the next repetition of a group that opens and closes with a position of that kind as one more position, whatever the group's `max`.
- <a id="tlmd_193"></a>TLMD_193 [human] [optional]: WHERE a later group that opens with a position of the kind of a run fits the block after the run only after its opening, the engine SHALL leave to that group the units at the end of the run that have the shape of the group's opening position, and none where that position has no shape.
- <a id="tlmd_194"></a>TLMD_194 [human] [unwanted]: IF every unit of a run has the shape of the opening position of the group of [TLMD_193](#tlmd_193), THEN the engine SHALL leave the first unit of the run to the position whose run it is.
- <a id="tlmd_79"></a>TLMD_79 [human] [invariant]: FOR ANY prose run and any ordered prose rules, the engine SHALL assign every unit to exactly one rule or report it exactly once.
- <a id="tlmd_80"></a>TLMD_80 [human] [invariant]: FOR ANY document and schema, the engine SHALL choose an extent no longer than the run less the units that [TLMD_193](#tlmd_193) leaves to a later group, and equal to it where that extent reports no error other than units that do not fit within a hard `max` above zero, so that no limit decides where a position ends.
- <a id="tlmd_81"></a>TLMD_81 [human] [invariant]: FOR ANY position that refuses a block, the engine SHALL preserve the findings, captures, repairs, and gates that the check had before the engine tried the position.

## 6. Units and lines

A unit rule shapes one unit of a block. A code or HTML block also accepts `lines`, the grammar of its content lines. Each content line, without its indent, is a unit of its own. A blank line is layout, not a line.

- <a id="tlmd_82"></a>TLMD_82 [human] [ubiquitous]: the engine SHALL apply each rule of the `lines` of a unit rule to the content lines of a code or HTML block, and to no fence marker.
- <a id="tlmd_83"></a>TLMD_83 [human] [ubiquitous]: the engine SHALL apply a line rule that declares a guard only to the lines that match that guard.
- <a id="tlmd_84"></a>TLMD_84 [human] [unwanted]: IF the number of lines that a guard selected falls outside the `min` and the hard `max` of the rule, THEN the engine SHALL report a finding that names the number and the bound.
- <a id="tlmd_85"></a>TLMD_85 [human] [ubiquitous]: the engine SHALL read an absent `min` as one on a line rule that declares no guard.
- <a id="tlmd_86"></a>TLMD_86 [human] [ubiquitous]: the engine SHALL report a failure of a line rule against the file line that caused it.
- <a id="tlmd_87"></a>TLMD_87 [human] [unwanted]: IF no rule of the `lines` of a fence takes a line, the line matches no part of a rule with a shape, and its unit rule declares no `lines-open`, THEN the engine SHALL report a finding that names the line.
- <a id="tlmd_88"></a>TLMD_88 [human] [ubiquitous]: the engine SHALL attribute a key that a line rule without `key-of` captures to the `rule-id` of the enclosing block.
- <a id="tlmd_89"></a>TLMD_89 [human] [optional]: WHERE a unit rule or a line rule declares `key-of`, the engine SHALL file the keys that the rule captures under that name and not under the id of the block.
- <a id="tlmd_236"></a>TLMD_236 [human] [ubiquitous]: the engine SHALL file no key under the `rule-id` of a unit rule or a line rule.
- <a id="tlmd_237"></a>TLMD_237 [human] [unwanted]: IF a unit rule or a line rule declares `key-of` and captures no key, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_90"></a>TLMD_90 [human] [unwanted]: IF `lines` appears on a unit rule of a block that is neither code nor HTML, or on a line rule, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_91"></a>TLMD_91 [human] [unwanted]: IF a guard, `min`, or `max` appears on a unit rule outside the `rules` of a block and the `lines` of a unit rule, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_92"></a>TLMD_92 [human] [unwanted]: IF a rule under `rules` or `lines` declares a guard and declares no `min`, THEN the engine SHALL report a finding on the schema that names the rule.
- <a id="tlmd_235"></a>TLMD_235 [human] [unwanted]: IF a rule under `rules` or `lines` declares a `max` and neither binds a format nor declares a `rule-id`, THEN the engine SHALL report a finding on the schema that names the rule.
- <a id="tlmd_142"></a>TLMD_142 [human] [unwanted]: IF a rule declares `text` beside `select-pattern`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_93"></a>TLMD_93 [human] [unwanted]: IF a rule under `rules` or `lines` takes or counts fewer units than its `min`, THEN the engine SHALL report its selector or its shape, and the count.
- <a id="tlmd_159"></a>TLMD_159 [human] [unwanted]: IF a rule under `rules` or `lines` takes or counts more units than the count of a hard `max`, THEN the engine SHALL report its selector or its shape and the count, and quote the `when-at-limit` of the `max`, as [TL_77][sokf:tramlines-spec#tl_77] states.
- <a id="tlmd_94"></a>TLMD_94 [human] [ubiquitous]: the engine SHALL check a line of a fence against the prohibited patterns of the unit rule and the format that govern it.
- <a id="tlmd_95"></a>TLMD_95 [human] [unwanted]: IF a key of a rule that the `for-each-key-of` of a unit rule or a section names has no unit or section in scope, THEN the engine SHALL report a finding that names the key, on the line that captures it, whether or not the block that contains it is present.
- <a id="tlmd_96"></a>TLMD_96 [human] [unwanted]: IF the key of a unit or a section that declares `for-each-key-of` is not a key that a rule it names captures in scope, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_97"></a>TLMD_97 [human] [ubiquitous]: the engine SHALL check a pairing in each occurrence of the `parent` of the section where the section declares one, and in the whole document otherwise.
- <a id="tlmd_98"></a>TLMD_98 [human] [unwanted]: IF a section or a unit rule declares `for-each-key-of` and captures no key, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_167"></a>TLMD_167 [human] [unwanted]: IF a name that a `for-each-key-of` holds names no section, block, or `key-of` that holds a key, THEN the engine SHALL report a finding on the schema that names it, and that says to declare `key-of` where the name is the `rule-id` of a unit rule or a line rule.
- <a id="tlmd_238"></a>TLMD_238 [human] [ubiquitous]: the engine SHALL resolve a name that a `for-each-key-of` or a `key-derives-from` holds to the keys filed under it by the section or the block whose `rule-id` it is and by each rule whose `key-of` it is, other than the keys of a rule that pairs with or derives from that name.
- <a id="tlmd_197"></a>TLMD_197 [human] [ubiquitous]: the engine SHALL read a `for-each-key-of` that holds a list of names as a pairing with the union of the keys filed under those names, each distinct key once.
- <a id="tlmd_99"></a>TLMD_99 [human] [invariant]: FOR ANY fence text and any guard, the engine SHALL check with a rule exactly the lines that match the guard, in file order, and report no other line against that rule, which applies to units and lines alike.
- <a id="tlmd_100"></a>TLMD_100 [human] [invariant]: FOR ANY failing line rule, the engine SHALL report the finding at the line whose text failed, inside the content of the block.

## 7. Keys and anchors

A rule captures a key from a heading, an item, a row, or a line: the hole of its template, or the named group of its pattern. Each key has one generated anchor, `<a id="cli-1_ac-2"></a>`, so that a fragment opens the rendered page at the key. A repair writes, moves, and removes the anchors. A line that holds only anchors reads as a blank line. A list item whose own line holds only anchors after its marker still reads as a list item, as it does with its anchors in place.

A unit breaks its rule when the rule that takes it reports a finding on the unit, or when the engine reports that no rule or position takes it. A heading that captures no key breaks its rule when the engine, as it matches the heading to a declared section, reports a finding on the heading's line, such as that no declared section takes it ([TLMD_19](#tlmd_19)) or that the section which takes it has another level ([TLMD_144](#tlmd_144)). A heading that its repair rewrites to the text and the level of its section breaks its rule only through another such finding. While a heading that no declared section takes breaks its rule, each unit between it and the next heading breaks its rule too, because the engine checks none of them, and the units under a heading that an open region admits with no finding on its line break no rule. A paragraph, an item, a row, or a heading holds the anchors on the lines of its text, a heading inside a list item and outside every blockquote among them, and any other unit, such as a code block or one of its lines, holds those on the line of anchors before its block. No unit or heading holds an anchor with the slug of a heading. A unit that its repair rewrites to the text of its rule breaks no rule, and the repair writes the unit without its anchors. An item that its repair rewrites still breaks its rule while the engine reports content beyond its text ([TLMD_162](#tlmd_162)), because the repair rewrites only the text. The repair that writes a setext heading as ATX ([TLMD_138](#tlmd_138)) keeps the heading's text, so it leaves a heading that breaks its rule broken. A unit or a heading whose text fails its shape captures no key, so an anchor that it holds names no key until the text passes again.

- <a id="tlmd_101"></a>TLMD_101 [human] [unwanted]: IF the slug of a key that a heading or the capture that defines it holds equals the slug of another such key or of a heading in the same document, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_148"></a>TLMD_148 [human] [unwanted]: IF rules that declare no `for-each-key-of` capture one key twice in the scope that `unique` declares, other than a capture in the body of a heading that repeats where its section does not, of a key that the body it repeats captures before it, or a capture by a rule that declares `key-repeats`, THEN the engine SHALL report a finding that names the key and the line of its first capture.
- <a id="tlmd_198"></a>TLMD_198 [human] [optional]: WHERE a rule declares `key-repeats`, the engine SHALL read each of its captures as a reference to the key, which a capture by a rule without `key-repeats` defines where there is one, and otherwise its first capture.
- <a id="tlmd_102"></a>TLMD_102 [human] [ubiquitous]: the host SHALL check each scope of a key wider than the document that the host adds.
- <a id="tlmd_103"></a>TLMD_103 [human] [ubiquitous]: the engine SHALL read a key as defined by the one capture of a rule without `for-each-key-of`, and every other capture of it as a reference to it, which needs no anchor unless a heading makes it.
- <a id="tlmd_104"></a>TLMD_104 [human] [optional]: WHERE a heading captures the key, the engine SHALL put the anchor at the start of the text of the heading, and none where the slug of the heading is the slug of the key.
- <a id="tlmd_105"></a>TLMD_105 [human] [optional]: WHERE a paragraph, an item, or a table cell captures the key, the engine SHALL put the anchor directly before the text that the rule matches, and before a code span or a link that contains that point.
- <a id="tlmd_106"></a>TLMD_106 [human] [optional]: WHERE a line of a code or HTML block captures the key, the engine SHALL put the anchor on a line of its own directly before the block, the anchors of the block in the order of capture, then a blank line.
- <a id="tlmd_107"></a>TLMD_107 [human] [ubiquitous]: the engine SHALL remove each anchor, and no other text, from the text before any rule reads the text, read the blocks that the document holds with its anchors where a list item's own line holds only anchors, and form the slug of a heading without its anchors.
- <a id="tlmd_108"></a>TLMD_108 [human] [ubiquitous]: the engine SHALL read any HTML other than an anchor of this exact form as written.
- <a id="tlmd_109"></a>TLMD_109 [human] [unwanted]: IF a key has no anchor, an anchor is not where its key is, an anchor names no key and no unit or heading that breaks its rule holds it, or an anchor is written two times, other than an anchor beside a second capture of its key or of a key with its slug, or beside a capture with `for-each-key-of`, THEN the engine SHALL report a finding.
- <a id="tlmd_110"></a>TLMD_110 [human] [ubiquitous]: the engine SHALL report no anchor finding on a scaffold.

## 8. Checking

The engine checks a document against a schema at the states of the document. The check reads the frontmatter, the sections, the blocks and their units, the keys and their anchors, the line limit, and the bans.

- <a id="tlmd_111"></a>TLMD_111 [human] [ubiquitous]: the engine SHALL exclude fenced blocks and region interiors from every prohibited pattern of the schema, and match every prohibited pattern, of the schema, of a format, and of a unit rule, against the text with each code span overwritten by backticks of the same length.
- <a id="tlmd_112"></a>TLMD_112 [human] [ubiquitous]: the engine SHALL exclude a code span that opens on one line and closes on a later one.
- <a id="tlmd_113"></a>TLMD_113 [human] [ubiquitous]: the engine SHALL ask the host what an image in the scheme of the host shows, and what the target of a region names, to match a `source-pattern`.
- <a id="tlmd_174"></a>TLMD_174 [human] [ubiquitous]: the engine SHALL check a document against the rules that its host adds, after its own rules, and treat each finding and each gate of the host as its own.
- <a id="tlmd_212"></a>TLMD_212 [human] [optional]: WHERE the host asks for the units that the unit rules accept, the engine SHALL return in the output of a check, for each paragraph, item, and table row that a unit rule takes and whose text has the template or the pattern of the rule and a shape of its format, whatever the bans of the rule and of the format report, the first and the last line of the unit, the place of the rule among the unit rules of the schema, and the `rule-id` of the rule where it declares one.
- <a id="tlmd_213"></a>TLMD_213 [human] [event]: WHEN the engine releases a position, the engine SHALL discard the records of [TLMD_212](#tlmd_212) of the units that the position took.
- <a id="tlmd_228"></a>TLMD_228 [agent] [event]: WHEN a position that refused a block keeps the block with the findings of its refusal, as [TLMD_62](#tlmd_62) and [TLMD_67](#tlmd_67) state, the engine SHALL claim for the position, which in a section whose blocks are in any order is the first position that refused the block, every block that the refusal covers, and keep the records of [TLMD_212](#tlmd_212) of the units that the position took.

## 9. Scaffolds

A schema derives one scaffold for each combination of its lifecycle states. The text of a scaffold is the text of the schema: the `text` that a rule fixes, the `example` of each rule, the example of the format that a rule binds, and the `examples` of a section that repeats. A guidance line holds only the guidance, the `when-at-limit` texts, and the ban messages that the schema states. Before each `when-at-limit` it holds the words that the host gives for the limit, or, where the host gives none, the limit in the syntax of the schema, such as `` `hard: 9`, `when-at-limit`: ``. The engine adds no words of its own, so a schema in any language derives a scaffold wholly in that language from a host that words each limit, and each label, in the same language. The `example` of a block or a unit rule can be a list of texts. The scaffold then writes one unit for each text, so that a new document shows several units rather than one.

- <a id="tlmd_114"></a>TLMD_114 [human] [ubiquitous]: the engine SHALL derive one scaffold for each state combination of a schema and check it against that schema.
- <a id="tlmd_115"></a>TLMD_115 [human] [ubiquitous]: the engine SHALL write into a scaffold only text that the schema fixes, that an example, a `guidance`, a `when-at-limit`, or the message of a ban states, or that the host gives for a limit, a limit written in the syntax of the schema as [TLMD_231](#tlmd_231) and [TLMD_232](#tlmd_232) state, and a label as [TLMD_255](#tlmd_255) and [TLMD_256](#tlmd_256) write it.
- <a id="tlmd_116"></a>TLMD_116 [human] [unwanted]: IF a scaffold needs a part whose text neither the schema fixes nor an example states, THEN the engine SHALL report a finding on the schema that names the rule that demands the part.
- <a id="tlmd_154"></a>TLMD_154 [human] [unwanted]: IF the scaffold splices a key into the example of a rule that declares `key-derives-from` or `for-each-key-of`, and the example does not carry a key where the rule reads one, THEN the engine SHALL refuse the derivation with a finding on the schema that names the rule and the key.
- <a id="tlmd_205"></a>TLMD_205 [human] [event]: WHEN the scaffold splices a prefix into the key of the example of a rule that declares `key-derives-from`, the engine SHALL replace the value that the schema states for the parent, as [TL_129][sokf:tramlines-spec#tl_129] defines, that starts the key followed by the separator, with the prefix and the separator.
- <a id="tlmd_216"></a>TLMD_216 [human] [event]: WHEN the scaffold writes the key of the example of a rule that declares `key-derives-from`, the schema states no value, as [TL_129][sokf:tramlines-spec#tl_129] defines, for the parent that `key-derives-from` names, and the key does not start with a value of the parent that the scaffold wrote followed by the separator, the engine SHALL replace the text of the key before its last separator with the first value of the parent that the scaffold wrote.
- <a id="tlmd_155"></a>TLMD_155 [human] [unwanted]: IF a rule takes its text or its heading from a frontmatter key that the scaffold does not write, THEN the engine SHALL refuse the derivation with a finding on the schema that names the rule and the key.
- <a id="tlmd_206"></a>TLMD_206 [human] [event]: WHEN the scaffold writes a table block that declares `example-columns`, the engine SHALL write those columns as the header row, each `|` in a name escaped as `\|`, then a separator row with one `---` for each column, and then each row of the block.
- <a id="tlmd_208"></a>TLMD_208 [human] [unwanted]: IF the scaffold writes a row of a table block that declares `example-columns`, and the row has a number of cells other than the number of those columns, THEN the engine SHALL refuse the derivation with a finding on the schema, at the rule that gives the row or else at the block, that names the row and both numbers.
- <a id="tlmd_117"></a>TLMD_117 [human] [ubiquitous]: the engine SHALL take the `example` of the bound format for a unit rule that binds a format and has no `example`.
- <a id="tlmd_118"></a>TLMD_118 [human] [ubiquitous]: the engine SHALL open the body of a scaffold with one `<ns>:guidance` line that holds the `guidance` of the schema and then the message of each ban of the schema, or with no guidance line where that text is empty.
- <a id="tlmd_119"></a>TLMD_119 [human] [ubiquitous]: the engine SHALL write the `guidance` of a section, a preamble, or a block as one `<!-- <ns>:guidance … -->` line before that part.
- <a id="tlmd_195"></a>TLMD_195 [human] [ubiquitous]: the engine SHALL write in a scaffold the message of each ban after the guidance of the part that the ban governs, and the message of each ban of the schema in the opening guidance line.
- <a id="tlmd_231"></a>TLMD_231 [human] [optional]: WHERE a block or one of its unit rules or line rules declares a `max`, the engine SHALL write the `when-at-limit` of each such limit unchanged into the guidance line of the block, after the block's own guidance and after the words that the host gives for that limit, or, where the host gives none, after the key of the limit's kind with its count and the key `when-at-limit`, as `` `hard: 9`, `when-at-limit`: ``, and before the messages of the block's bans, and add no other text.
- <a id="tlmd_232"></a>TLMD_232 [human] [optional]: WHERE a frontmatter key declares a `max`, the engine SHALL write its `when-at-limit` unchanged into the guidance line of the key, after the key's own guidance and after the words that the host gives for the limit, or, where the host gives none, after the key of the limit's kind with its count and the key `when-at-limit`, as `` `hard: 2`, `when-at-limit`: ``, and add no other text.
- <a id="tlmd_234"></a>TLMD_234 [human] [optional]: WHERE a schema declares a `language`, the engine SHALL give the tag unchanged to the host each time it asks the host to word a limit or a label.
- <a id="tlmd_233"></a>TLMD_233 [human] [optional]: WHERE the host asks for the guidance of a schema at a combination of states, the engine SHALL return, in the order of the schema, an entry for the schema and for each frontmatter key, the preamble, each section, and each block that the states admit, each entry with the part's own guidance, the text of the guidance line that the scaffold writes for the part, whether the part is optional at those states, and the part's limits, each limit with its kind, its count, its `when-at-limit`, the format and the `rule-id` of the unit rule or the line rule that declares it, and whether that rule counts units or lines, and the entry for the schema with the key and the words of each label, and the entry for a block with the `label` of the block and of each of its unit rules that declares one.
- <a id="tlmd_196"></a>TLMD_196 [human] [ubiquitous]: the engine SHALL check no guidance line against the prohibited patterns of the schema.
- <a id="tlmd_120"></a>TLMD_120 [human] [ubiquitous]: the engine SHALL write the `guidance` of a frontmatter constraint or a lifecycle dimension as a `# <ns>:guidance …` comment line before it.
- <a id="tlmd_121"></a>TLMD_121 [human] [ubiquitous]: the engine SHALL write into a scaffold the `only` marker of each gated rule that the scaffold satisfies.
- <a id="tlmd_122"></a>TLMD_122 [human] [unwanted]: IF another rule of the schema matches a part that a gated rule writes into a scaffold, so that a marker of the scaffold gates nothing or disagrees with the rule that matches its part, or a gated part has no marker, THEN the engine SHALL report a finding on the schema, at the rule of the part where the engine knows it, that quotes the part.
- <a id="tlmd_123"></a>TLMD_123 [human] [ubiquitous]: the engine SHALL write a part that the schema does not demand at the states of the scaffold inside an `optional` region, and open no region for a part that the schema demands at those states.
- <a id="tlmd_124"></a>TLMD_124 [human] [unwanted]: IF a part inside an `optional` region has no text to write, THEN the engine SHALL leave it out and report a warning that names the rule.
- <a id="tlmd_125"></a>TLMD_125 [human] [unwanted]: IF two optional regions of a scaffold are adjacent and the second opens no section, THEN the engine SHALL write them as one.
- <a id="tlmd_192"></a>TLMD_192 [human] [ubiquitous]: the engine SHALL write each optional section of a scaffold in an optional region of its own.
- <a id="tlmd_126"></a>TLMD_126 [human] [optional]: WHERE a unit rule declares `lines` and its fence has an example, the engine SHALL copy that example into the scaffold.
- <a id="tlmd_127"></a>TLMD_127 [human] [optional]: WHERE a scaffold contains the example of a fence, the engine SHALL read the keys of the line rules out of that example.
- <a id="tlmd_128"></a>TLMD_128 [human] [optional]: WHERE a section declares `parent`, the engine SHALL write the section inside each occurrence of its parent in a scaffold.
- <a id="tlmd_129"></a>TLMD_129 [human] [optional]: WHERE a unit rule declares `for-each-key-of`, the engine SHALL write one unit for each distinct key of the referenced rules in scope, with the key spliced into the example of the rule.
- <a id="tlmd_199"></a>TLMD_199 [human] [ubiquitous]: the engine SHALL count toward the units that a guarded rule of `rules` writes into a scaffold the units that earlier rules of the block write and its guard selects, and read the keys of the rule out of those units.
- <a id="tlmd_130"></a>TLMD_130 [human] [optional]: WHERE a section that declares `parent` contains a pairing, the engine SHALL write the section where the pairing has keys, leave it out where the pairing has none, and put it in no optional region.
- <a id="tlmd_131"></a>TLMD_131 [human] [optional]: WHERE a unit rule of a list declares `blocks`, the engine SHALL write those blocks under each item that it writes, indented to the text of the item.
- <a id="tlmd_132"></a>TLMD_132 [human] [ubiquitous]: the engine SHALL write the frontmatter keys that the host owns first, and the lines of the host last, and with the null host only the keys that the schema declares.
- <a id="tlmd_133"></a>TLMD_133 [human] [invariant]: FOR ANY schema and any state combination, the engine SHALL either refuse the derivation with a finding that names a rule, or derive a scaffold that satisfies that schema at those states with an agreeing marker for every gate of that check.
- <a id="tlmd_164"></a>TLMD_164 [human] [unwanted]: IF a scaffold that the engine derives fails its schema, other than on a line that holds an example that fails its own rule or format, or at a rule whose example fails it, THEN the engine SHALL report a finding on the schema, at the rule that the scaffold fails where the failure has one, that names the states of the scaffold, quotes the line of the scaffold that fails, and names the failure.
- <a id="tlmd_134"></a>TLMD_134 [human] [invariant]: FOR ANY schema that derives, the engine SHALL write into the scaffold, outside its frontmatter and its markers, only a heading that the schema fixes, text that a rule fixes, an example that the schema states, or a label as [TLMD_255](#tlmd_255) and [TLMD_256](#tlmd_256) write it.
- <a id="tlmd_135"></a>TLMD_135 [human] [invariant]: FOR ANY schema whose subsection is inside a repeating section, optional or not, paired or not, numbered or not, and gated on any states, the engine SHALL derive a scaffold that satisfies that schema at each state.
- <a id="tlmd_149"></a>TLMD_149 [human] [unwanted]: IF a position of a prose, bullet list, or numbered list block that states no unit shape, format, ban, guard, or sequence precedes a position that opens with a block of the same kind, other than a group in the list of the first position whose first inner block is required and has a shape and whose second block is required and of another kind that no other later position opens with, and every position between them at the states of a scaffold is optional, or closes a group that opens with that kind and whose hard `max` is not one, THEN the engine SHALL refuse the derivation with a finding on the rule of the first position.
- <a id="tlmd_176"></a>TLMD_176 [human] [ubiquitous]: the engine SHALL accept as the `example` of a block or a unit rule either one string or a list of strings.
- <a id="tlmd_177"></a>TLMD_177 [human] [ubiquitous]: the engine SHALL check each string of a list `example` against its rule as it checks a single `example`.
- <a id="tlmd_178"></a>TLMD_178 [human] [unwanted]: IF a string of a list `example` of more than one string fails its rule, THEN the engine SHALL report a finding on the schema that names the position of the string in the list.
- <a id="tlmd_179"></a>TLMD_179 [human] [ubiquitous]: the engine SHALL quote the first string of a list `example` wherever a finding shows the example of the rule.
- <a id="tlmd_180"></a>TLMD_180 [human] [unwanted]: IF a section declares `examples` and does not declare `repeat: true`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_181"></a>TLMD_181 [human] [unwanted]: IF a text of the `examples` of a section does not begin with a heading of the level of the section, THEN the engine SHALL report a finding on the schema that names the position of the text.
- <a id="tlmd_182"></a>TLMD_182 [human] [unwanted]: IF the `example` of a block or a unit rule is an empty list, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_183"></a>TLMD_183 [human] [unwanted]: IF a `code` block that does not declare `repeat: true` carries, on the block or on one of its unit rules, a list `example` of more than one string, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_184"></a>TLMD_184 [human] [optional]: WHERE the `example` of a unit rule or of a block is a list, the engine SHALL write one unit for each string of the list, in order, and no more units than the hard `max` of the rule or of the block allows.
- <a id="tlmd_185"></a>TLMD_185 [human] [optional]: WHERE the `min` of a rule or of a block demands more units than its `example` gives strings, the engine SHALL write further copies of the last string.
- <a id="tlmd_186"></a>TLMD_186 [human] [optional]: WHERE a unit rule under `rules` or `lines` declares `min: 0` and a list `example`, the engine SHALL write the strings of the list.
- <a id="tlmd_187"></a>TLMD_187 [human] [optional]: WHERE a position of a `sequence` carries a list `example`, the engine SHALL write every string of the list where the position declares `repeat: true`, and only the first string otherwise.
- <a id="tlmd_188"></a>TLMD_188 [human] [optional]: WHERE a `code` block writes more than one unit, the engine SHALL write one fence for each unit, each after its own `only` marker.
- <a id="tlmd_189"></a>TLMD_189 [human] [optional]: WHERE a repeating section that declares no `for-each-key-of` declares `examples`, the engine SHALL write each text of `examples`, in order and unchanged, after the occurrence that it derives, in the same optional region and after the same `only` marker.
- <a id="tlmd_190"></a>TLMD_190 [human] [optional]: WHERE a text of the `examples` of a section has a heading that captures a key, the engine SHALL record that key for the rules that refer to the section.
- <a id="tlmd_191"></a>TLMD_191 [human] [optional]: WHERE a section that declares `for-each-key-of` declares `examples`, the engine SHALL write the occurrence for each key after the first from the next text of `examples`, unchanged, and write the occurrences for the keys that remain after the last text from the rules.
- <a id="tlmd_150"></a>TLMD_150 [human] [optional]: WHERE a unit rule of a list declares `blocks` whose first block is not a list, the engine SHALL write a blank line between the text of each item and its nested blocks.

## 10. Repairs

The engine repairs what the frontmatter and the schema determine. It never edits what belongs to the host.

- <a id="tlmd_136"></a>TLMD_136 [human] [ubiquitous]: the engine SHALL repair a heading or a unit bound to a frontmatter value to that value.
- <a id="tlmd_165"></a>TLMD_165 [human] [unwanted]: IF a heading or a unit bound to a frontmatter value differs from that value, THEN the engine SHALL report a finding that names the value and repair the text to it.
- <a id="tlmd_168"></a>TLMD_168 [human] [unwanted]: IF a heading or a unit is bound to a frontmatter key that the document does not set and that the schema does not demand in the states of the document, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_137"></a>TLMD_137 [human] [unwanted]: IF a heading has the text of a `renamed-from` of its section, THEN the engine SHALL report a finding and repair the heading to the current text.
- <a id="tlmd_143"></a>TLMD_143 [human] [unwanted]: IF a section declares `renamed-from` and its heading is a template or a pattern, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_138"></a>TLMD_138 [human] [unwanted]: IF a heading is a setext heading, THEN the engine SHALL report a finding and repair the heading to an ATX heading.
- <a id="tlmd_139"></a>TLMD_139 [human] [ubiquitous]: the engine SHALL repair the anchors of a document as section 7 places them, leave where it stands each anchor that names no key and that a unit or a heading which breaks its rule holds, other than one on the lines that a repair rewrites, which it writes at the start of the new text, keep the text before each line of anchors that it removes apart from the text after it, and keep each list item whose line the repair leaves holding only its marker apart from the block before the item.
- <a id="tlmd_140"></a>TLMD_140 [human] [ubiquitous]: the engine SHALL edit no interior of a region, no trailer, and no blank line before the trailer.

## 11. Labels

A label marks a unit with one of the labels that its schema declares, such as who wrote the unit. The schema declares the key of each label under `labels`, and a block or a unit rule declares with `label` whether its units require a label or allow one. A label is its words in square brackets, followed by one space. It stands straight after the key of a unit that begins with the key that its rule captures, and otherwise at the start of the unit. In a task item, the box, `[ ]` or `[x]` and one space, comes before both:

```markdown
- D-2 [human] The error names the failed pattern of the last rule set only, because an error string of 9P holds at most 128 bytes.
- [ ] [agent] WHEN a message matches no rule set, the file server SHALL fail the write to `send`.
```

The host gives the words of each label in the language of the schema, and a label whose host gives none has its key as its words. The engine writes no words of its own for a label: a scaffold holds each label in the words of the host, or as its key.

- <a id="tlmd_239"></a>TLMD_239 [human] [ubiquitous]: the engine SHALL accept in a schema the key `labels`, a list of the keys of its labels, each a slug, such as `labels: [human, agent]`.
- <a id="tlmd_240"></a>TLMD_240 [human] [ubiquitous]: the engine SHALL accept on a block of prose, a block quote, a list, or a table, and on a unit rule of such a block, the key `label`, whose value is `required` or `optional`.
- <a id="tlmd_241"></a>TLMD_241 [human] [unwanted]: IF a block or a unit rule declares `label` and its schema declares no `labels`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_242"></a>TLMD_242 [human] [ubiquitous]: the engine SHALL apply the `label` of a block to each unit of the block whose unit rule declares no `label`, and to a position of a `sequence` that declares none the `label` of the block's `each` before the block's.
- <a id="tlmd_243"></a>TLMD_243 [human] [ubiquitous]: the engine SHALL read as a label a text in square brackets followed by one space, whose words are not empty, hold no square bracket, do not begin with `^`, and are not the label of a link reference definition of the document.
- <a id="tlmd_244"></a>TLMD_244 [human] [optional]: WHERE the rule of a unit captures a key that begins the unit, the engine SHALL read the unit's label straight after the key.
- <a id="tlmd_245"></a>TLMD_245 [human] [optional]: WHERE no key that the rule of a unit captures begins the unit, the engine SHALL read the unit's label at the start of the unit.
- <a id="tlmd_246"></a>TLMD_246 [human] [optional]: WHERE an item of a list begins with the box of a task item, the engine SHALL read the item as beginning after the box and one space when it reads the item's label, and match the box with the rest of the item's text.
- <a id="tlmd_247"></a>TLMD_247 [human] [optional]: WHERE the rule of a unit declares no `label`, the engine SHALL read as a label only the words that the host gives for a label of the schema, or the label's key.
- <a id="tlmd_248"></a>TLMD_248 [human] [ubiquitous]: the engine SHALL match the text of a unit without its label against the shape, the key, the guard, and the bans of the unit's rule, and read no label on a line of a code block or of an HTML block.
- <a id="tlmd_249"></a>TLMD_249 [human] [unwanted]: IF a unit whose rule declares `label: required` has the shape of its rule and has no label, THEN the engine SHALL report a finding on the unit that lists the words of the labels of the schema.
- <a id="tlmd_250"></a>TLMD_250 [human] [unwanted]: IF a unit whose rule declares no `label` has a label of its schema, THEN the engine SHALL report a finding on the unit that names the label.
- <a id="tlmd_251"></a>TLMD_251 [human] [unwanted]: IF the label of a unit whose rule declares `label` is none of the words that the host gives for the labels of the schema, THEN the engine SHALL report a finding on the unit that lists those words.
- <a id="tlmd_252"></a>TLMD_252 [human] [ubiquitous]: the engine SHALL ask the host for the words of each label in the language of the schema, both when it checks a document and when it derives a scaffold.
- <a id="tlmd_253"></a>TLMD_253 [human] [optional]: WHERE the host gives no words for a label, the engine SHALL take the label's key as its words.
- <a id="tlmd_254"></a>TLMD_254 [human] [event]: WHEN a unit holds its label as the key and the host gives other words for the label, the engine SHALL derive a repair that rewrites the label in those words where it stands.
- <a id="tlmd_255"></a>TLMD_255 [human] [event]: WHEN the engine derives a scaffold, the engine SHALL write a label, in the words that the host gives for the language of the schema, on each unit whose rule requires a label, and on each unit whose rule allows one and whose example carries one.
- <a id="tlmd_256"></a>TLMD_256 [human] [ubiquitous]: the engine SHALL write on a unit of a scaffold the label whose key the host names, else the label that the unit's example carries, else the first label that the schema declares.
- <a id="tlmd_257"></a>TLMD_257 [human] [ubiquitous]: the engine SHALL check the label of each example of a rule as it checks the label of a unit, against the words that the host gives.

<!-- sokf:links -->

[sokf:tramlines-spec]: /knowledge/spec/tramlines-spec.md
[sokf:tramlines-spec#tl_129]: /knowledge/spec/tramlines-spec.md#tl_129
[sokf:tramlines-spec#tl_77]: /knowledge/spec/tramlines-spec.md#tl_77
