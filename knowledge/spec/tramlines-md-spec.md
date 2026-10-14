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

## 2. Documents

A document opens with frontmatter when its first line is `---`. The body is CommonMark, with GitHub tables, footnotes, and task lists. The engine reads the body as blocks and units.

- <a id="tlmd_1"></a>TLMD_1 [event]: WHEN the first line of a document is `---`, the engine SHALL read the lines up to the next `---` line as frontmatter and the lines after it as the body.
- <a id="tlmd_2"></a>TLMD_2 [unwanted]: IF the frontmatter has no closing `---` line, THEN the engine SHALL read the frontmatter to the end of the text, with an empty body.
- <a id="tlmd_3"></a>TLMD_3 [ubiquitous]: the engine SHALL read a text with CRLF line endings as LF.
- <a id="tlmd_4"></a>TLMD_4 [ubiquitous]: the engine SHALL read the body as CommonMark, with GitHub tables, footnotes, and task lists.
- <a id="tlmd_5"></a>TLMD_5 [ubiquitous]: the engine SHALL read consecutive paragraphs as one prose block, and a paragraph of images only as an image block.
- <a id="tlmd_6"></a>TLMD_6 [ubiquitous]: the engine SHALL read a paragraph with its lines joined by one space, and a line of a fence without its indent.
- <a id="tlmd_7"></a>TLMD_7 [ubiquitous]: the engine SHALL match the item's own text, and check what the item contains beyond that text, a nested list or a fence, against the `blocks` of its rule.
- <a id="tlmd_162"></a>TLMD_162 [unwanted]: IF an item contains a block after its first paragraph, such as a nested list, a fence, a blockquote, or another paragraph, or in place of it on the item's own line, and its rule declares no `blocks`, THEN the engine SHALL report a finding on the first line of that block, and hold no text of the item to its rule where the block stands in place of the text.
- <a id="tlmd_173"></a>TLMD_173 [ubiquitous]: the engine SHALL read a heading inside a list item or a blockquote as a block of that item or quote, of a kind that no schema declares, and only a heading outside both as a heading of the document.
- <a id="tlmd_172"></a>TLMD_172 [unwanted]: IF a blockquote contains a heading, THEN the engine SHALL report a finding that names the heading, and count no paragraph missing from a quote that holds only headings.
- <a id="tlmd_8"></a>TLMD_8 [ubiquitous]: the engine SHALL record the position of every code span of a body.
- <a id="tlmd_9"></a>TLMD_9 [invariant]: FOR ANY line, the engine SHALL produce a masked line with the length and the line number of the original, and change no byte outside a recorded span.

## 3. Dialect and markers

The engine has three markers of its own, each in the namespace `<ns>` of the dialect:

- `<!-- <ns>:only k=v,v2 -->` - The part after it belongs to those states.
- `<!-- <ns>:guidance … -->` - What an example cannot say, before the part that it guides. Before a frontmatter key, the marker is `# <ns>:guidance …`.
- `<!-- <ns>:optional -->` - With `<!-- /<ns>:optional -->`, a part that the schema does not demand.

A host adds to its dialect a region, a trailer, or a scheme.

- <a id="tlmd_10"></a>TLMD_10 [unwanted]: IF a governed document contains an `only` marker, THEN the engine SHALL report a finding on the line of the marker.
- <a id="tlmd_158"></a>TLMD_158 [unwanted]: IF a governed document contains a `guidance` marker, THEN the engine SHALL report a finding on the line of the marker.
- <a id="tlmd_11"></a>TLMD_11 [unwanted]: IF a governed document contains an `optional` region, both of its markers, THEN the engine SHALL report a warning.
- <a id="tlmd_12"></a>TLMD_12 [unwanted]: IF an `optional` marker has no partner, THEN the engine SHALL report a finding.
- <a id="tlmd_171"></a>TLMD_171 [ubiquitous]: the engine SHALL count toward `line-limit` no marker line outside a fence, and no blank line that sets one apart, so that a scaffold is held to the length of the document that it leaves.
- <a id="tlmd_13"></a>TLMD_13 [optional]: WHERE the dialect names a region, the engine SHALL read the lines from `<!-- <ns>:<region> target -->` to `<!-- /<ns>:<region> -->` as one block of the kind that the dialect names, and check its interior with no rule.
- <a id="tlmd_14"></a>TLMD_14 [optional]: WHERE the dialect names a trailer, the engine SHALL read `<!-- <ns>:<trailer> -->` and the link reference definitions after it as a generated block that no rule reads.
- <a id="tlmd_15"></a>TLMD_15 [optional]: WHERE the dialect names a scheme, the engine SHALL preserve the rest of a link label `<scheme>:<rest>` as written, for the host to resolve.
- <a id="tlmd_16"></a>TLMD_16 [ubiquitous]: the engine SHALL read a marker inside a fenced code block as text.
- <a id="tlmd_17"></a>TLMD_17 [ubiquitous]: the engine SHALL assign the default dialect the namespace `tramlines`, and no region, trailer, or scheme.

## 4. Sections and headings

A schema lists the headings of a document under `sections`, in document order. The body of a section runs to the next heading of any level. Sections nest by level, except that a section that declares `parent` is inside its parent.

- <a id="tlmd_18"></a>TLMD_18 [unwanted]: IF a declared section that does not declare `optional` is missing from the document, or from an occurrence of its parent where it declares `parent`, THEN the engine SHALL report a finding that names the section.
- <a id="tlmd_151"></a>TLMD_151 [unwanted]: IF the heading of a declared section that does not declare `repeat` occurs more than once in the document, or in one occurrence of its parent where it declares `parent`, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_19"></a>TLMD_19 [unwanted]: IF a heading matches no section and no open region admits it, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_20"></a>TLMD_20 [unwanted]: IF a heading is more than one level deeper than the heading before it, matches no section or matches a section at the level that the section declares, and does not lie outside every occurrence of the parent of its section, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_21"></a>TLMD_21 [unwanted]: IF the sections of a schema that orders them skip a level, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_22"></a>TLMD_22 [unwanted]: IF a declared section is out of the listed order, its heading does not repeat where the section does not declare `repeat`, and the schema does not set `sections-ordered: false`, THEN the engine SHALL report a finding that names the first such heading.
- <a id="tlmd_144"></a>TLMD_144 [unwanted]: IF a heading matches a section at another level than the section declares, THEN the engine SHALL report a finding that names the level.
- <a id="tlmd_23"></a>TLMD_23 [ubiquitous]: the engine SHALL assign a heading that an exact heading and a varying heading both match to the exact one.
- <a id="tlmd_175"></a>TLMD_175 [ubiquitous]: the engine SHALL match a heading against the varying headings of the sections of the heading's own level before it matches the heading against the varying heading of a section of another level.
- <a id="tlmd_24"></a>TLMD_24 [optional]: WHERE a section declares `headings-open`, the engine SHALL accept undeclared headings of its level or deeper after it, up to the next declared heading, and check their bodies against only the rules of the document scope.
- <a id="tlmd_25"></a>TLMD_25 [optional]: WHERE a section declares `subheadings-open`, the engine SHALL accept undeclared headings deeper than its level after it, up to the next declared heading.
- <a id="tlmd_26"></a>TLMD_26 [unwanted]: IF a heading in an open region has the exact text of a prohibited heading, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_170"></a>TLMD_170 [unwanted]: IF `sections-prohibited` names a heading that a declared section matches in some states that both admit, THEN the engine SHALL report a finding on the schema that names the heading.
- <a id="tlmd_27"></a>TLMD_27 [ubiquitous]: the engine SHALL accept on the heading of a section the keys that a unit rule accepts for one line: a shape, `key`, `unique`, `key-derives-from`, and `example`.
- <a id="tlmd_28"></a>TLMD_28 [ubiquitous]: the engine SHALL accept a heading key equal to a key that a rule named in the `for-each-key-of` of the section captures.
- <a id="tlmd_29"></a>TLMD_29 [unwanted]: IF two sections or two units that one `for-each-key-of` pairs to a rule capture the same key in one scope, other than a capture in the body of a heading that repeats where its section does not, of a key that the body it repeats captures before it, THEN the engine SHALL report a finding on the second that names the key and the line of the first.
- <a id="tlmd_30"></a>TLMD_30 [ubiquitous]: the engine SHALL check each pairing for a repeated key on its own, in each scope that the engine reads it in.
- <a id="tlmd_31"></a>TLMD_31 [unwanted]: IF a heading of a section that declares `parent` follows no occurrence of the parent before the next heading of the level of the parent or higher, THEN the engine SHALL report a finding that names the heading.
- <a id="tlmd_32"></a>TLMD_32 [optional]: WHERE a section declares `parent`, the engine SHALL count its `optional` and its `repeat` within each occurrence of the parent.
- <a id="tlmd_166"></a>TLMD_166 [unwanted]: IF a heading matches a section whose `parent` does not apply at the states of the document, THEN the engine SHALL report a finding that names the heading and the parent.
- <a id="tlmd_33"></a>TLMD_33 [ubiquitous]: the engine SHALL accept a heading of a repeating section after the headings of the sections inside it.
- <a id="tlmd_34"></a>TLMD_34 [unwanted]: IF a `parent` names no section, THEN the engine SHALL report a finding on the schema that names it.
- <a id="tlmd_156"></a>TLMD_156 [unwanted]: IF a `parent` names a section of a level that is not smaller than the level of the section that declares it, THEN the engine SHALL report a finding on the schema that names both levels.
- <a id="tlmd_157"></a>TLMD_157 [unwanted]: IF a section that declares `parent` is not listed after its parent with only the other sections inside that parent between, THEN the engine SHALL report a finding on the schema that names the section.
- <a id="tlmd_35"></a>TLMD_35 [unwanted]: IF a section declares `number-from-parent` and its heading does not begin with the number of the heading of its parent, a dot and a number, THEN the engine SHALL report a finding that names the expected number.
- <a id="tlmd_36"></a>TLMD_36 [unwanted]: IF the heading of the parent of a section that declares `number-from-parent` has no number, THEN the engine SHALL report a finding that names that heading.
- <a id="tlmd_37"></a>TLMD_37 [optional]: WHERE a schema declares a `preamble`, the engine SHALL check the blocks before the first heading of a document against the `blocks` of the preamble.
- <a id="tlmd_38"></a>TLMD_38 [unwanted]: IF a body has content before its first heading and the schema declares no `preamble`, THEN the engine SHALL report a finding.

## 5. Blocks

A section declares its body as an ordered list of blocks, each of one kind with rules for its units. A position claims the blocks of its kind. Where a position refuses a block, the engine offers the block to the positions after it.

- <a id="tlmd_39"></a>TLMD_39 [unwanted]: IF no declared position claims a block of a body, in order unless the section sets `blocks-ordered: false`, THEN the engine SHALL report a finding that names the kind of the block.
- <a id="tlmd_152"></a>TLMD_152 [unwanted]: IF a position that does not declare `optional` claims no block and no block of another kind stands where it expects one, THEN the engine SHALL report a finding that names the kind that it expects.
- <a id="tlmd_40"></a>TLMD_40 [optional]: WHERE a section declares no `blocks`, the engine SHALL read its body as one prose block of one or more paragraphs.
- <a id="tlmd_41"></a>TLMD_41 [ubiquitous]: the engine SHALL exempt generated blocks, region interiors, footnote definitions, and HTML comments from block matching.
- <a id="tlmd_42"></a>TLMD_42 [unwanted]: IF a block declares a key that does not apply to its kind, or a unit rule declares a key that does not apply where the rule sits, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tlmd_153"></a>TLMD_153 [unwanted]: IF a block lacks a key that its kind requires, `lang` for a `code` block, `columns` for a `table` block, or `blocks` for a `group` block, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tlmd_43"></a>TLMD_43 [unwanted]: IF a fence has no tag, or a tag that the `lang` of its block does not list, THEN the engine SHALL report a finding.
- <a id="tlmd_145"></a>TLMD_145 [unwanted]: IF a fence has fewer backticks than the `fence` of its block, THEN the engine SHALL report a finding that names both numbers.
- <a id="tlmd_44"></a>TLMD_44 [unwanted]: IF the header row of a table differs from the declared `columns`, beyond further columns that `columns-open` accepts, THEN the engine SHALL report a finding.
- <a id="tlmd_45"></a>TLMD_45 [unwanted]: IF a unit contains HTML other than a comment and its block does not set `inline-html`, THEN the engine SHALL report a finding.
- <a id="tlmd_46"></a>TLMD_46 [ubiquitous]: the engine SHALL match the path of an image or the target of a region against a `source-pattern` by what the host reports that it shows where the path is in the scheme of the host, and as written otherwise.
- <a id="tlmd_163"></a>TLMD_163 [unwanted]: IF the path of an image or the target of a region does not match the `source-pattern` of its block, THEN the engine SHALL report a finding that names the path and the pattern.
- <a id="tlmd_200"></a>TLMD_200 [unwanted]: IF the host reports that an image or the target of a region names a document, THEN the engine SHALL report the finding of [TLMD_163](#tlmd_163) whatever the `source-pattern` of its block, with a message that says that it names a document.
- <a id="tlmd_47"></a>TLMD_47 [unwanted]: IF a block declares `rules` beside `each` or beside `sequence`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_146"></a>TLMD_146 [unwanted]: IF a block has fewer units than its `min`, THEN the engine SHALL report a finding that names the bound and the count.
- <a id="tlmd_160"></a>TLMD_160 [unwanted]: IF a block has more units than its `max`, THEN the engine SHALL report a finding that names the bound and the count.
- <a id="tlmd_147"></a>TLMD_147 [unwanted]: IF a position of a `sequence` that does not declare `optional` takes no unit, THEN the engine SHALL report a finding that names the position.
- <a id="tlmd_161"></a>TLMD_161 [unwanted]: IF units of a block that declares `sequence` remain after its last position, THEN the engine SHALL report a finding on each of them.
- <a id="tlmd_48"></a>TLMD_48 [ubiquitous]: the engine SHALL let consecutive declared prose rules take successive runs of units from one prose block.
- <a id="tlmd_49"></a>TLMD_49 [ubiquitous]: the engine SHALL bound each run that one of these prose rules takes by the `min` and the `max` of that rule.
- <a id="tlmd_50"></a>TLMD_50 [unwanted]: IF units of a prose, bullet list, or numbered list block remain after the last position that takes a run of them, THEN the engine SHALL report a finding against the first unit that remains.
- <a id="tlmd_51"></a>TLMD_51 [ubiquitous]: the engine SHALL divide a prose run only under ordered block matching.
- <a id="tlmd_52"></a>TLMD_52 [ubiquitous]: the engine SHALL apply each entry of the `rules` of a block that declares a guard to the units that match that guard.
- <a id="tlmd_53"></a>TLMD_53 [unwanted]: IF no entry of `rules` takes a unit, the unit matches no part of a rule with a shape, and the block declares no `units-open`, THEN the engine SHALL report a finding that names the unit.
- <a id="tlmd_54"></a>TLMD_54 [ubiquitous]: the engine SHALL bound the units that an entry selected by the `min` and the `max` of that entry.
- <a id="tlmd_55"></a>TLMD_55 [ubiquitous]: the engine SHALL assign a unit to the first rule with a shape and no guard whose shape the unit has.
- <a id="tlmd_56"></a>TLMD_56 [ubiquitous]: the engine SHALL count, and not take, the units that a rule matches when that rule declares only a guard and a count.
- <a id="tlmd_57"></a>TLMD_57 [unwanted]: IF no rule takes a unit or a line that partly matches a rule with a shape, and the block declares no `units-open` or its unit rule no `lines-open`, THEN the engine SHALL report the unit against the rule that it matches furthest, with the column where it stops.
- <a id="tlmd_58"></a>TLMD_58 [unwanted]: IF an earlier rule with a shape and no guard takes the example of a later one, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_59"></a>TLMD_59 [ubiquitous]: the engine SHALL claim a block for an optional, repeating, or grouped position only where that position reports no error against the block.
- <a id="tlmd_60"></a>TLMD_60 [ubiquitous]: the engine SHALL claim a block for a required position that does not repeat, whatever that position reports against the block.
- <a id="tlmd_61"></a>TLMD_61 [unwanted]: IF the engine cannot complete a group repetition, THEN the engine SHALL release every block that the repetition claimed.
- <a id="tlmd_62"></a>TLMD_62 [ubiquitous]: the engine SHALL apply one claim test whether a section fixes the block order or frees it.
- <a id="tlmd_63"></a>TLMD_63 [ubiquitous]: the engine SHALL offer a block that a position released to the positions after it that declare its kind.
- <a id="tlmd_64"></a>TLMD_64 [ubiquitous]: the engine SHALL discard the findings, captures, repairs, and gates of a position that it releases.
- <a id="tlmd_65"></a>TLMD_65 [unwanted]: IF a position reports only warnings against a block, THEN the engine SHALL claim the block.
- <a id="tlmd_66"></a>TLMD_66 [unwanted]: IF a position refuses a block of its own kind and then reports itself missing, THEN the engine SHALL report why the position refused the block.
- <a id="tlmd_67"></a>TLMD_67 [unwanted]: IF no position claims a block that a position refused, THEN the engine SHALL report why the last position that tried the block refused it.
- <a id="tlmd_68"></a>TLMD_68 [event]: WHEN a block of a kind that no later position declares appears before the block that a position declares, the engine SHALL move the position past it, as a block that no position claims.
- <a id="tlmd_169"></a>TLMD_169 [unwanted]: IF the block of a position that is no group and does not declare `optional` stands after a block that a later position declares, and no later position declares the block of the position, THEN the engine SHALL report a finding that names the block, and check the block against its position.
- <a id="tlmd_69"></a>TLMD_69 [unwanted]: IF a position meets a block of another kind and no block of its own kind follows, THEN the engine SHALL name the expected kind and the found kind in one finding.
- <a id="tlmd_70"></a>TLMD_70 [unwanted]: IF a finding names the found kind, THEN the engine SHALL claim that block for the position that named it.
- <a id="tlmd_71"></a>TLMD_71 [unwanted]: IF a repetition of a group fails where the group needs it to reach its `min`, where no later position takes its opening block, or where the group's own rule accepted its opening block, THEN the engine SHALL report why that repetition failed and claim its blocks for the group.
- <a id="tlmd_72"></a>TLMD_72 [unwanted]: IF a block fits an inner block of a group after the first and no later position takes it, THEN the engine SHALL read it as a repetition that has no opening blocks and name the expected kind.
- <a id="tlmd_73"></a>TLMD_73 [ubiquitous]: the engine SHALL claim for a group every block that the group took, whatever the group reports about them.
- <a id="tlmd_74"></a>TLMD_74 [ubiquitous]: the engine SHALL assign to a position over a run the longest extent of units that its bounds admit and that it reports no error against.
- <a id="tlmd_75"></a>TLMD_75 [unwanted]: IF no extent of a run satisfies a position that claims a block whatever it reports, THEN the engine SHALL assign to that position the longest extent that its bounds admit.
- <a id="tlmd_76"></a>TLMD_76 [ubiquitous]: the engine SHALL choose an extent only for a prose, bullet list, or numbered list block.
- <a id="tlmd_77"></a>TLMD_77 [ubiquitous]: the engine SHALL choose an extent only where a section fixes the block order.
- <a id="tlmd_78"></a>TLMD_78 [ubiquitous]: the engine SHALL choose an extent only where a section declares two or more positions of the kind of the block, counting a group that opens with a position of that kind as one more position, whatever closes the group, and counting the next repetition of a group that opens and closes with a position of that kind as one more position, where the group's `max` admits that repetition.
- <a id="tlmd_193"></a>TLMD_193 [optional]: WHERE a later group opens with a position of the kind of a run, and the block after the run fits that group only after its opening, the engine SHALL leave the units that the group's opening position demands at the end of the run to that group.
- <a id="tlmd_194"></a>TLMD_194 [unwanted]: IF the bounds of the position admit no extent that leaves those units, THEN the engine SHALL choose the extent that TLMD_74 chooses.
- <a id="tlmd_79"></a>TLMD_79 [invariant]: FOR ANY prose run and any ordered prose rules, the engine SHALL assign every unit to exactly one rule or report it exactly once.
- <a id="tlmd_80"></a>TLMD_80 [invariant]: FOR ANY document and schema, the engine SHALL choose an extent no longer than the extent that the bounds alone produce, and equal to it where that extent reports no error, so that a document with no error under the bounds alone has no error.
- <a id="tlmd_81"></a>TLMD_81 [invariant]: FOR ANY position that refuses a block, the engine SHALL preserve the findings, captures, repairs, and gates that the check had before the engine tried the position.

## 6. Units and lines

A unit rule shapes one unit of a block. A code or HTML block also accepts `lines`, the grammar of its content lines. Each content line, without its indent, is a unit of its own. A blank line is layout, not a line.

- <a id="tlmd_82"></a>TLMD_82 [ubiquitous]: the engine SHALL apply each rule of the `lines` of a unit rule to the content lines of a code or HTML block, and to no fence marker.
- <a id="tlmd_83"></a>TLMD_83 [ubiquitous]: the engine SHALL apply a line rule that declares a guard only to the lines that match that guard.
- <a id="tlmd_84"></a>TLMD_84 [unwanted]: IF the number of lines that a guard selected falls outside the `min` and the `max` of the rule, THEN the engine SHALL report a finding that names the number and the bound.
- <a id="tlmd_85"></a>TLMD_85 [ubiquitous]: the engine SHALL read an absent `min` as one on a line rule that declares no guard.
- <a id="tlmd_86"></a>TLMD_86 [ubiquitous]: the engine SHALL report a failure of a line rule against the file line that caused it.
- <a id="tlmd_87"></a>TLMD_87 [unwanted]: IF no rule of the `lines` of a fence takes a line, the line matches no part of a rule with a shape, and its unit rule declares no `lines-open`, THEN the engine SHALL report a finding that names the line.
- <a id="tlmd_88"></a>TLMD_88 [ubiquitous]: the engine SHALL attribute a key that a line rule captures to the `rule-id` of the enclosing block.
- <a id="tlmd_89"></a>TLMD_89 [optional]: WHERE a unit rule declares `rule-id`, the engine SHALL file the keys that the unit rule captures under that id and not under the id of the block.
- <a id="tlmd_90"></a>TLMD_90 [unwanted]: IF `lines` appears on a unit rule of a block that is neither code nor HTML, or on a line rule, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_91"></a>TLMD_91 [unwanted]: IF a guard, `min`, or `max` appears on a unit rule outside the `rules` of a block and the `lines` of a unit rule, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_92"></a>TLMD_92 [unwanted]: IF a rule under `rules` or `lines` declares a guard and declares no `min`, THEN the engine SHALL report a finding on the schema that names the rule.
- <a id="tlmd_142"></a>TLMD_142 [unwanted]: IF a rule declares `text` beside `select-pattern`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_93"></a>TLMD_93 [unwanted]: IF a rule under `rules` or `lines` takes or counts fewer units than its `min`, THEN the engine SHALL report its selector or its shape, and the count.
- <a id="tlmd_159"></a>TLMD_159 [unwanted]: IF a rule under `rules` or `lines` takes or counts more units than its `max`, THEN the engine SHALL report its selector or its shape, and the count.
- <a id="tlmd_94"></a>TLMD_94 [ubiquitous]: the engine SHALL check a line of a fence against the prohibited patterns of the unit rule and the format that govern it.
- <a id="tlmd_95"></a>TLMD_95 [unwanted]: IF a key of a rule that the `for-each-key-of` of a unit rule or a section names has no unit or section in scope, THEN the engine SHALL report a finding that names the key, on the line that captures it, whether or not the block that contains it is present.
- <a id="tlmd_96"></a>TLMD_96 [unwanted]: IF the key of a unit or a section that declares `for-each-key-of` is not a key that a rule it names captures in scope, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_97"></a>TLMD_97 [ubiquitous]: the engine SHALL check a pairing in each occurrence of the `parent` of the section where the section declares one, and in the whole document otherwise.
- <a id="tlmd_98"></a>TLMD_98 [unwanted]: IF a section or a unit rule declares `for-each-key-of` and captures no key, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_167"></a>TLMD_167 [unwanted]: IF a rule-id of a `for-each-key-of` names no rule that captures a key, THEN the engine SHALL report a finding on the schema that names it.
- <a id="tlmd_197"></a>TLMD_197 [ubiquitous]: the engine SHALL read a `for-each-key-of` that holds a list of rule-ids as a pairing with the union of the keys that those rules capture, each distinct key once.
- <a id="tlmd_99"></a>TLMD_99 [invariant]: FOR ANY fence text and any guard, the engine SHALL check with a rule exactly the lines that match the guard, in file order, and report no other line against that rule, which applies to units and lines alike.
- <a id="tlmd_100"></a>TLMD_100 [invariant]: FOR ANY failing line rule, the engine SHALL report the finding at the line whose text failed, inside the content of the block.

## 7. Keys and anchors

A rule captures a key from a heading, an item, a row, or a line: the hole of its template, or the named group of its pattern. Each key has one generated anchor, `<a id="cli-1_ac-2"></a>`, so that a fragment opens the rendered page at the key. A repair writes, moves, and removes the anchors.

- <a id="tlmd_101"></a>TLMD_101 [unwanted]: IF the slug of a key that a heading or the capture that defines it holds equals the slug of another such key or of a heading in the same document, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_148"></a>TLMD_148 [unwanted]: IF rules that declare no `for-each-key-of` capture one key twice in the scope that `unique` declares, other than a capture in the body of a heading that repeats where its section does not, of a key that the body it repeats captures before it, or a capture by a rule that declares `key-repeats`, THEN the engine SHALL report a finding that names the key and the line of its first capture.
- <a id="tlmd_198"></a>TLMD_198 [optional]: WHERE a rule declares `key-repeats`, the engine SHALL read each of its captures as a reference to the key, which a capture by a rule without `key-repeats` defines where there is one, and otherwise its first capture.
- <a id="tlmd_102"></a>TLMD_102 [ubiquitous]: the host SHALL check each scope of a key wider than the document that the host adds.
- <a id="tlmd_103"></a>TLMD_103 [ubiquitous]: the engine SHALL read a key as defined by the one capture of a rule without `for-each-key-of`, and every other capture of it as a reference to it, which needs no anchor unless a heading makes it.
- <a id="tlmd_104"></a>TLMD_104 [optional]: WHERE a heading captures the key, the engine SHALL put the anchor at the start of the text of the heading, and none where the slug of the heading is the slug of the key.
- <a id="tlmd_105"></a>TLMD_105 [optional]: WHERE a paragraph, an item, or a table cell captures the key, the engine SHALL put the anchor directly before the text that the rule matches, and before a code span or a link that contains that point.
- <a id="tlmd_106"></a>TLMD_106 [optional]: WHERE a line of a code or HTML block captures the key, the engine SHALL put the anchor on a line of its own directly before the block, the anchors of the block in the order of capture, then a blank line.
- <a id="tlmd_107"></a>TLMD_107 [ubiquitous]: the engine SHALL remove each anchor from the text before any rule reads the text, and form the slug of a heading without it.
- <a id="tlmd_108"></a>TLMD_108 [ubiquitous]: the engine SHALL read any HTML other than an anchor of this exact form as written.
- <a id="tlmd_109"></a>TLMD_109 [unwanted]: IF a key has no anchor, an anchor is not where its key is, an anchor names no key, or an anchor is written two times, other than an anchor beside a second capture of its key or of a key with its slug, or beside a capture with `for-each-key-of`, THEN the engine SHALL report a finding.
- <a id="tlmd_110"></a>TLMD_110 [ubiquitous]: the engine SHALL report no anchor finding on a scaffold.

## 8. Checking

The engine checks a document against a schema at the states of the document. The check reads the frontmatter, the sections, the blocks and their units, the keys and their anchors, the line limit, and the bans.

- <a id="tlmd_111"></a>TLMD_111 [ubiquitous]: the engine SHALL exclude fenced blocks, code spans, and region interiors from every prohibited pattern of the schema.
- <a id="tlmd_112"></a>TLMD_112 [ubiquitous]: the engine SHALL exclude a code span that opens on one line and closes on a later one.
- <a id="tlmd_113"></a>TLMD_113 [ubiquitous]: the engine SHALL ask the host what an image in the scheme of the host shows, and what the target of a region names, to match a `source-pattern`.
- <a id="tlmd_174"></a>TLMD_174 [ubiquitous]: the engine SHALL check a document against the rules that its host adds, after its own rules, and treat each finding and each gate of the host as its own.

## 9. Scaffolds

A schema derives one scaffold for each combination of its lifecycle states. The text of a scaffold is the text of the schema: the `text` that a rule fixes, the `example` of each rule, the example of the format that a rule binds, and the `examples` of a section that repeats. The `example` of a block or a unit rule can be a list of texts. The scaffold then writes one unit for each text, so that a new document shows several units rather than one.

- <a id="tlmd_114"></a>TLMD_114 [ubiquitous]: the engine SHALL derive one scaffold for each state combination of a schema and check it against that schema.
- <a id="tlmd_115"></a>TLMD_115 [ubiquitous]: the engine SHALL write into a scaffold no text that neither the schema fixes nor an example states.
- <a id="tlmd_116"></a>TLMD_116 [unwanted]: IF a scaffold needs a part whose text neither the schema fixes nor an example states, THEN the engine SHALL report a finding on the schema that names the rule that demands the part.
- <a id="tlmd_154"></a>TLMD_154 [unwanted]: IF the scaffold splices a key into the example of a rule that declares `key-derives-from` or `for-each-key-of`, and the example does not carry a key where the rule reads one, THEN the engine SHALL refuse the derivation with a finding on the schema that names the rule and the key.
- <a id="tlmd_155"></a>TLMD_155 [unwanted]: IF a rule takes its text or its heading from a frontmatter key that the scaffold does not write, THEN the engine SHALL refuse the derivation with a finding on the schema that names the rule and the key.
- <a id="tlmd_117"></a>TLMD_117 [ubiquitous]: the engine SHALL take the `example` of the bound format for a unit rule that binds a format and has no `example`.
- <a id="tlmd_118"></a>TLMD_118 [ubiquitous]: the engine SHALL open every scaffold with one `<ns>:guidance` line before its title heading, whose text starts `The writer replaces each of the following examples with their own text.` and continues with the `guidance` of the schema.
- <a id="tlmd_119"></a>TLMD_119 [ubiquitous]: the engine SHALL write the `guidance` of a section, a preamble, or a block as one `<!-- <ns>:guidance … -->` line before that part.
- <a id="tlmd_195"></a>TLMD_195 [ubiquitous]: the engine SHALL write in a scaffold the message of each ban after the guidance of the part that the ban governs, and the message of each ban of the schema in the opening guidance line.
- <a id="tlmd_196"></a>TLMD_196 [ubiquitous]: the engine SHALL check no guidance line against the prohibited patterns of the schema.
- <a id="tlmd_120"></a>TLMD_120 [ubiquitous]: the engine SHALL write the `guidance` of a frontmatter constraint or a lifecycle dimension as a `# <ns>:guidance …` comment line before it.
- <a id="tlmd_121"></a>TLMD_121 [ubiquitous]: the engine SHALL write into a scaffold the `only` marker of each gated rule that the scaffold satisfies.
- <a id="tlmd_122"></a>TLMD_122 [unwanted]: IF another rule of the schema matches a part that a gated rule writes into a scaffold, so that a marker of the scaffold gates nothing or disagrees with the rule that matches its part, or a gated part has no marker, THEN the engine SHALL report a finding on the schema, at the rule of the part where the engine knows it, that quotes the part.
- <a id="tlmd_123"></a>TLMD_123 [ubiquitous]: the engine SHALL write a part that the schema does not demand inside an `optional` region.
- <a id="tlmd_124"></a>TLMD_124 [unwanted]: IF a part inside an `optional` region has no text to write, THEN the engine SHALL leave it out and report a warning that names the rule.
- <a id="tlmd_125"></a>TLMD_125 [unwanted]: IF two optional regions of a scaffold are adjacent and the second opens no section, THEN the engine SHALL write them as one.
- <a id="tlmd_192"></a>TLMD_192 [ubiquitous]: the engine SHALL write each optional section of a scaffold in an optional region of its own.
- <a id="tlmd_126"></a>TLMD_126 [optional]: WHERE a unit rule declares `lines` and its fence has an example, the engine SHALL copy that example into the scaffold.
- <a id="tlmd_127"></a>TLMD_127 [optional]: WHERE a scaffold contains the example of a fence, the engine SHALL read the keys of the line rules out of that example.
- <a id="tlmd_128"></a>TLMD_128 [optional]: WHERE a section declares `parent`, the engine SHALL write the section inside each occurrence of its parent in a scaffold.
- <a id="tlmd_129"></a>TLMD_129 [optional]: WHERE a unit rule declares `for-each-key-of`, the engine SHALL write one unit for each distinct key of the referenced rules in scope, with the key spliced into the example of the rule.
- <a id="tlmd_199"></a>TLMD_199 [ubiquitous]: the engine SHALL count toward the units that a guarded rule of `rules` writes into a scaffold the units that earlier rules of the block write and its guard selects, and read the keys of the rule out of those units.
- <a id="tlmd_130"></a>TLMD_130 [optional]: WHERE a section that declares `parent` contains a pairing, the engine SHALL write the section where the pairing has keys, leave it out where the pairing has none, and put it in no optional region.
- <a id="tlmd_131"></a>TLMD_131 [optional]: WHERE a unit rule of a list declares `blocks`, the engine SHALL write those blocks under each item that it writes, indented to the text of the item.
- <a id="tlmd_132"></a>TLMD_132 [ubiquitous]: the engine SHALL write the frontmatter keys that the host owns first, and the lines of the host last, and with the null host only the keys that the schema declares.
- <a id="tlmd_133"></a>TLMD_133 [invariant]: FOR ANY schema and any state combination, the engine SHALL either refuse the derivation with a finding that names a rule, or derive a scaffold that satisfies that schema at those states with an agreeing marker for every gate of that check.
- <a id="tlmd_164"></a>TLMD_164 [unwanted]: IF a scaffold that the engine derives fails its schema, other than on a line that holds an example that fails its own rule or format, or at a rule whose example fails it, THEN the engine SHALL report a finding on the schema, at the rule that the scaffold fails where the failure has one, that names the states of the scaffold, quotes the line of the scaffold that fails, and names the failure.
- <a id="tlmd_134"></a>TLMD_134 [invariant]: FOR ANY schema that derives, the engine SHALL write into the scaffold, outside its frontmatter and its markers, only a heading that the schema fixes, text that a rule fixes, or an example that the schema states.
- <a id="tlmd_135"></a>TLMD_135 [invariant]: FOR ANY schema whose subsection is inside a repeating section, optional or not, paired or not, numbered or not, and gated on any states, the engine SHALL derive a scaffold that satisfies that schema at each state.
- <a id="tlmd_149"></a>TLMD_149 [unwanted]: IF a scaffold writes a prose, bullet list, or numbered list block directly after a block of the same kind whose rule admits more units than the scaffold writes for it and states no unit shape, format, ban, guard, or sequence, THEN the engine SHALL refuse the derivation with a finding on the rule of the first block.
- <a id="tlmd_176"></a>TLMD_176 [ubiquitous]: the engine SHALL accept as the `example` of a block or a unit rule either one string or a list of strings.
- <a id="tlmd_177"></a>TLMD_177 [ubiquitous]: the engine SHALL check each string of a list `example` against its rule as it checks a single `example`.
- <a id="tlmd_178"></a>TLMD_178 [unwanted]: IF a string of a list `example` of more than one string fails its rule, THEN the engine SHALL report a finding on the schema that names the position of the string in the list.
- <a id="tlmd_179"></a>TLMD_179 [ubiquitous]: the engine SHALL quote the first string of a list `example` wherever a finding shows the example of the rule.
- <a id="tlmd_180"></a>TLMD_180 [unwanted]: IF a section declares `examples` and does not declare `repeat: true`, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_181"></a>TLMD_181 [unwanted]: IF a text of the `examples` of a section does not begin with a heading of the level of the section, THEN the engine SHALL report a finding on the schema that names the position of the text.
- <a id="tlmd_182"></a>TLMD_182 [unwanted]: IF the `example` of a block or a unit rule is an empty list, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_183"></a>TLMD_183 [unwanted]: IF a `code` block that does not declare `repeat: true` carries, on the block or on one of its unit rules, a list `example` of more than one string, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_184"></a>TLMD_184 [optional]: WHERE the `example` of a unit rule or of a block is a list, the engine SHALL write one unit for each string of the list, in order, and no more units than the `max` of the rule or of the block allows.
- <a id="tlmd_185"></a>TLMD_185 [optional]: WHERE the `min` of a rule or of a block demands more units than its `example` gives strings, the engine SHALL write further copies of the last string.
- <a id="tlmd_186"></a>TLMD_186 [optional]: WHERE a unit rule under `rules` or `lines` declares `min: 0` and a list `example`, the engine SHALL write the strings of the list.
- <a id="tlmd_187"></a>TLMD_187 [optional]: WHERE a position of a `sequence` carries a list `example`, the engine SHALL write every string of the list where the position declares `repeat: true`, and only the first string otherwise.
- <a id="tlmd_188"></a>TLMD_188 [optional]: WHERE a `code` block writes more than one unit, the engine SHALL write one fence for each unit, each after its own `only` marker.
- <a id="tlmd_189"></a>TLMD_189 [optional]: WHERE a repeating section that declares no `for-each-key-of` declares `examples`, the engine SHALL write each text of `examples`, in order and unchanged, after the occurrence that it derives, in the same optional region and after the same `only` marker.
- <a id="tlmd_190"></a>TLMD_190 [optional]: WHERE a text of the `examples` of a section has a heading that captures a key, the engine SHALL record that key for the rules that refer to the section.
- <a id="tlmd_191"></a>TLMD_191 [optional]: WHERE a section that declares `for-each-key-of` declares `examples`, the engine SHALL write the occurrence for each key after the first from the next text of `examples`, unchanged, and write the occurrences for the keys that remain after the last text from the rules.
- <a id="tlmd_150"></a>TLMD_150 [optional]: WHERE a unit rule of a list declares `blocks` whose first block is not a list, the engine SHALL write a blank line between the text of each item and its nested blocks.

## 10. Repairs

The engine repairs what the frontmatter and the schema determine. It never edits what belongs to the host.

- <a id="tlmd_136"></a>TLMD_136 [ubiquitous]: the engine SHALL repair a heading or a unit bound to a frontmatter value to that value.
- <a id="tlmd_165"></a>TLMD_165 [unwanted]: IF a heading or a unit bound to a frontmatter value differs from that value, THEN the engine SHALL report a finding that names the value and repair the text to it.
- <a id="tlmd_168"></a>TLMD_168 [unwanted]: IF a heading or a unit is bound to a frontmatter key that the document does not set and that the schema does not demand in the states of the document, THEN the engine SHALL report a finding that names the key.
- <a id="tlmd_137"></a>TLMD_137 [unwanted]: IF a heading has the text of a `renamed-from` of its section, THEN the engine SHALL report a finding and repair the heading to the current text.
- <a id="tlmd_143"></a>TLMD_143 [unwanted]: IF a section declares `renamed-from` and its heading is a template or a pattern, THEN the engine SHALL report a finding on the schema.
- <a id="tlmd_138"></a>TLMD_138 [unwanted]: IF a heading is a setext heading, THEN the engine SHALL report a finding and repair the heading to an ATX heading.
- <a id="tlmd_139"></a>TLMD_139 [ubiquitous]: the engine SHALL repair the anchors of a document as section 7 places them.
- <a id="tlmd_140"></a>TLMD_140 [ubiquitous]: the engine SHALL edit no interior of a region, no trailer, and no blank line before the trailer.

<!-- sokf:links -->

[sokf:tramlines-spec]: /knowledge/spec/tramlines-spec.md
