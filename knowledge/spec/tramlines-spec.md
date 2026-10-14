---
type: TramlinesSpecification
id: tramlines-spec
title: tramlines core specification
description: The grammar that a schema uses, how the engine builds a grammar from layers, and how the engine checks a document against a schema in any format.
version: "0.1"
status: draft
---

# tramlines core specification

This specification defines tramlines: the grammar that a schema uses, and how the engine checks a document against a schema, whatever the format of the document.

tramlines validates a document against a schema and derives the scaffold that a new document starts from. A format grammar adds what is specific to the documents of one format. The [markdown specification][sokf:tramlines-md-spec] defines the first format. The host, not the engine, handles the documents around a document. The host also adds vocabulary of its own.

## About this specification

Each normative rule of this specification is an EARS criterion, keyed `TL_n`.

## 1. Terminology

- <a id="engine"></a>`engine` - The library that reads a grammar, checks a schema against it, checks a document against a schema, and derives a scaffold.
- <a id="document"></a>`document` - A text of one format, with optional frontmatter and a body, which a schema governs.
- <a id="frontmatter"></a>`frontmatter` - The optional map of named values, written in YAML, that opens a document. The format grammar defines where it appears.
- <a id="body"></a>`body` - The part of a document that is not its frontmatter, which the format grammar describes.
- <a id="unit"></a>`unit` - The smallest part of a body that one rule checks, such as a paragraph, an item, or a line. The format grammar defines the units of its format.
- <a id="schema"></a>`schema` - A YAML map in the vocabulary of a grammar, that defines what a document of one kind contains.
- <a id="grammar"></a>`grammar` - The vocabulary in which a schema is written: the core grammar, one format grammar, and the host extensions, merged.
- <a id="core-grammar"></a>`core grammar` - The file `tramlines.grammar.yaml`: the meta vocabulary, and the schema keys that every format shares.
- <a id="format-grammar"></a>`format grammar` - The grammar file of one format, which names the core grammar as its base, such as `tramlines-md.grammar.yaml`.
- <a id="host-extension"></a>`host extension` - The grammar file of a host, which names a format grammar and only adds to it.
- <a id="host"></a>`host` - The program that uses the engine. It selects the schema that governs each document, resolves what is outside a document, and optionally extends the grammar.
- <a id="rule"></a>`rule` - One entry of a schema that a part of a document satisfies or breaks.
- <a id="shape"></a>`shape` - The required form of the text of one unit: a `text`, a `template`, or a `pattern`.
- <a id="type"></a>`type` - A named regular expression, or a list of choices, that a hole of a template matches.
- <a id="format"></a>`format` - A named shape of one line, with its example, that a rule binds by name.
- <a id="registry"></a>`registry` - One layer of named formats and types: the schema, the host, or the shared layer.
- <a id="example"></a>`example` - Text of the kind that a rule demands, which the engine checks against the rule.
- <a id="guidance"></a>`guidance` - What an example cannot express, which the scaffold includes and no finding prints.
- <a id="lifecycle-dimension"></a>`lifecycle dimension` - One ordered list of states, stored in one frontmatter key.
- <a id="key"></a>`key` - An identifier that a rule captures from a document.
- <a id="scaffold"></a>`scaffold` - The document that a schema derives, which a new document starts from.
- <a id="finding"></a>`finding` - One failure of a rule, on one line of one file.
- <a id="edit"></a>`edit` - A replacement of lines of a text: the form of every repair.
- <a id="command"></a>`command` - The `tramlines` program, which checks documents against one schema with no host.

## 2. Grammars

A grammar file is YAML. Its `meta` section is the vocabulary in which a grammar file is written, and its `schema` section is the vocabulary that a schema uses. The engine builds the grammar of a format from three layers: the core grammar, the format grammar, and the host extensions.

```yaml
grammar: tramlines-md
base: tramlines
version: "0.1"
```

- <a id="tl_1"></a>TL_1 [ubiquitous]: the core grammar SHALL contain the `meta` vocabulary and the schema keys that do not depend on the format of a document: `line-limit`, `guidance`, `lifecycle`, `frontmatter`, `frontmatter-open`, `prohibited-patterns`, `formats`, `types`, and `examples`.
- <a id="tl_2"></a>TL_2 [ubiquitous]: the format grammar SHALL name itself with `grammar` and name the core grammar with `base: tramlines`.
- <a id="tl_3"></a>TL_3 [ubiquitous]: the engine SHALL merge a format grammar onto the core grammar: new keys, new rule maps, values of an enum, block kinds that a key requires, text of a `doc`, and the constraint groups `exactly-one-of` and `at-least-one-of` of any rule map.
- <a id="tl_4"></a>TL_4 [unwanted]: IF a format grammar contains a `meta` section, or changes the type, the `of`, or the `required` of a key of the core grammar, THEN the engine SHALL reject the format grammar.
- <a id="tl_5"></a>TL_5 [ubiquitous]: the host extension SHALL name itself with `extension` and name the format grammar that it extends with `extends`.
- <a id="tl_6"></a>TL_6 [ubiquitous]: the engine SHALL merge a host extension onto the grammar of its format: new keys, new rule maps, values of an enum, block kinds that a key requires, text of a `doc`, and sections of its own.
- <a id="tl_7"></a>TL_7 [unwanted]: IF a host extension contains a `meta` section or a constraint group, or changes a definition of the grammar under it, THEN the engine SHALL reject the host extension.
- <a id="tl_8"></a>TL_8 [unwanted]: IF the `extends` of a host extension names a grammar other than the format grammar that the engine reads, THEN the engine SHALL reject the host extension.
- <a id="tl_9"></a>TL_9 [ubiquitous]: the engine SHALL preserve as written a key that a host extension adds, so that the host reads and checks it.
- <a id="tl_10"></a>TL_10 [ubiquitous]: the grammar SHALL state, in the `doc` of every key `min`, what an absent `min` means.

## 3. Schemas

A schema is strict: the engine requires what the schema declares, rejects what the schema does not declare, and treats each requirement as exclusive. A relaxation is a key that the schema sets explicitly. The host decides which schema governs each document.

- <a id="tl_11"></a>TL_11 [ubiquitous]: the engine SHALL check a schema against the grammar before it uses the schema.
- <a id="tl_12"></a>TL_12 [unwanted]: IF a schema contains a key that the grammar does not define for the rule map that holds it, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tl_101"></a>TL_101 [unwanted]: IF a value of a schema does not have the type that the grammar defines for its key, THEN the engine SHALL report a finding on the schema that names the key and the type.
- <a id="tl_102"></a>TL_102 [unwanted]: IF a value of a schema is not one of the values that the grammar lists for its key, does not have the format that the grammar defines for its key, or is a `seq-width` below one, or a schema names a type with a string that is not a slug, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tl_103"></a>TL_103 [unwanted]: IF a map of a schema lacks a key that the grammar requires of it, a key that another key of the map requires beside it, or every key of a group of which the grammar requires one, THEN the engine SHALL report a finding on the schema that names the missing key.
- <a id="tl_104"></a>TL_104 [unwanted]: IF a map of a schema contains more than one key of a group of which the grammar admits one, other than a group of shapes, THEN the engine SHALL report a finding on the schema that names the group.
- <a id="tl_13"></a>TL_13 [unwanted]: IF a schema does not parse as YAML 1.2, core schema, THEN the engine SHALL report a finding on the schema.
- <a id="tl_14"></a>TL_14 [unwanted]: IF a schema, or the file that `--formats` names, contains a YAML anchor, alias, or tag, THEN the engine SHALL report a finding on that file.
- <a id="tl_15"></a>TL_15 [unwanted]: IF two rules of one schema declare the same `rule-id`, THEN the engine SHALL report a finding on the second of them that names the id.
- <a id="tl_16"></a>TL_16 [ubiquitous]: the engine SHALL read each regular expression of a schema in the Rust dialect, with no lookaround and no backreference, and with no anchor that the regular expression does not contain.
- <a id="tl_118"></a>TL_118 [unwanted]: IF a regular expression of a schema, or the expression that a template compiles to, does not compile in that dialect, THEN the engine SHALL report a finding on the schema that names the key and the reason.
- <a id="tl_17"></a>TL_17 [ubiquitous]: the engine SHALL accept `prohibited-patterns`, a list of bans, each a `pattern` and a `message`, on a schema, a rule that has a shape, and a format.
- <a id="tl_18"></a>TL_18 [ubiquitous]: the engine SHALL report each failure of a rule that declares `severity: warn`, or of a rule inside one that declares it where the inner rule declares no `severity` of its own, as a warning, and each failure of any other rule as an error.
- <a id="tl_19"></a>TL_19 [optional]: WHERE a rule declares `only`, the engine SHALL apply the rule only when the state of the document is in the listed set for every dimension that `only` names.
- <a id="tl_20"></a>TL_20 [unwanted]: IF `only` names a dimension that the lifecycle does not declare, or a state that its dimension does not declare, THEN the engine SHALL report a finding on the schema that names it.
- <a id="tl_111"></a>TL_111 [unwanted]: IF a rule declares `only` and the schema declares no lifecycle, THEN the engine SHALL report a finding on the schema.
- <a id="tl_21"></a>TL_21 [unwanted]: IF a schema declares a `line-limit` greater than zero and a document has more lines than the limit, THEN the engine SHALL report a finding on the document.
- <a id="tl_98"></a>TL_98 [unwanted]: IF a rule declares a `min` greater than its `max`, THEN the engine SHALL report a finding on the schema.

## 4. Shapes and types

A shape defines the text of one unit, in one of three forms:

- `text` - The text itself.
- `template` - The text, with a typed hole `${name:type}` where it varies.
- `pattern` - A regular expression, for what a template cannot say.

Each shape matches the text with each run of whitespace collapsed to one space.

- <a id="tl_22"></a>TL_22 [unwanted]: IF a format, a variant, or a rule of the format grammar declares more than one of `text`, `template`, `pattern`, and `format`, a frontmatter constraint declares more than one of `text`, `template` and `pattern`, or a format declares one of them beside `variants`, THEN the engine SHALL report a finding on the schema that names them.
- <a id="tl_23"></a>TL_23 [ubiquitous]: the engine SHALL read the text of a unit with each run of whitespace as one space, for every shape, key, guard, and ban.
- <a id="tl_24"></a>TL_24 [ubiquitous]: the engine SHALL match a template against the whole text of a unit, each hole against the regular expression of its type.
- <a id="tl_105"></a>TL_105 [unwanted]: IF a format declares none of `text`, `template`, `pattern`, and `variants`, or a variant or a heading rule declares none of `text`, `template` and `pattern`, THEN the engine SHALL report a finding on the schema.
- <a id="tl_25"></a>TL_25 [unwanted]: IF a template has no hole, THEN the engine SHALL report a finding on the schema that names the template.
- <a id="tl_119"></a>TL_119 [unwanted]: IF a template names one hole twice, THEN the engine SHALL report a finding on the schema that names the hole.
- <a id="tl_120"></a>TL_120 [unwanted]: IF a hole of a template names a type that is not visible to the schema, THEN the engine SHALL report a finding on the schema that names the type.
- <a id="tl_121"></a>TL_121 [unwanted]: IF a template contains a `${` that no `}` closes, or a hole whose name is not letters, digits, and `_` that do not start with a digit, THEN the engine SHALL report a finding on the schema that names the template.
- <a id="tl_26"></a>TL_26 [unwanted]: IF a text fails a template, THEN the engine SHALL report the column where the text stops matching and what the template expects there: the end of a text that lacks only the end of the template, where only literals follow the last hole or the hole does not take the literal that follows it, and the start of a hole whose value runs on to the literal after the hole and that the hole does not take.
- <a id="tl_27"></a>TL_27 [ubiquitous]: the engine SHALL resolve a type by name through the `types` of the schema, of the host, and of the shared registry, and the built-in types, the nearest first.
- <a id="tl_28"></a>TL_28 [ubiquitous]: the engine SHALL reserve the built-in types `text`, `word`, `int`, `slug`, and `seq`, and the types that the host reserves.
- <a id="tl_29"></a>TL_29 [unwanted]: IF a schema defines a type with the name of a reserved type, THEN the engine SHALL report a finding on the schema that names the type.
- <a id="tl_109"></a>TL_109 [unwanted]: IF the regular expression of a type that a schema defines contains an anchor or a capture group, THEN the engine SHALL report a finding on the schema that names the type.
- <a id="tl_30"></a>TL_30 [ubiquitous]: the engine SHALL read a key as the capture of the hole or the named group that `key` names.
- <a id="tl_31"></a>TL_31 [unwanted]: IF the `key` of a rule names no hole or named group of the shape of the rule, or of every shape of the format that the rule binds, or the `key` of a format names none of every shape of the format, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tl_113"></a>TL_113 [unwanted]: IF a rule declares `key` beside a format that declares one, THEN the engine SHALL report a finding on the schema that names the format.
- <a id="tl_32"></a>TL_32 [ubiquitous]: the engine SHALL check a `key-derives-from` against the key of the rule that declares it.
- <a id="tl_33"></a>TL_33 [unwanted]: IF the part of a key before the first separator is not a prefix that its `key-derives-from` allows, THEN the engine SHALL report a finding that names the key.
- <a id="tl_96"></a>TL_96 [unwanted]: IF a `key-derives-from` sits on a unit rule that captures no key, THEN the engine SHALL report a finding on the schema.
- <a id="tl_117"></a>TL_117 [unwanted]: IF a `key-derives-from` names a rule or a frontmatter key that the schema does not declare, THEN the engine SHALL report a finding on the schema that names it.

## 5. Formats

A format names the shape of one line, so that rules in many schemas share it. A format has one shape and an example, or variants, which each have a shape and an example of their own. The registries are layers: the schema, then the host, then the shared formats.

- <a id="tl_34"></a>TL_34 [ubiquitous]: the engine SHALL resolve a format by name through the `formats` of the schema, of the host, and of the shared registry, the nearest first.
- <a id="tl_35"></a>TL_35 [ubiquitous]: the engine SHALL compile each format that is visible to a schema against the types that are visible to the schema.
- <a id="tl_36"></a>TL_36 [optional]: WHERE the nearer entry declares no patch, the engine SHALL replace a farther entry whole.
- <a id="tl_37"></a>TL_37 [ubiquitous]: the engine SHALL merge a format entry that declares itself a patch onto the entry that it shadows, by RFC 7386.
- <a id="tl_38"></a>TL_38 [ubiquitous]: the engine SHALL drop a key that a patch states as null.
- <a id="tl_39"></a>TL_39 [ubiquitous]: the engine SHALL replace a list-valued key of a patch whole and not join its elements.
- <a id="tl_40"></a>TL_40 [unwanted]: IF a format that is not a patch has one shape and no `example`, THEN the engine SHALL report a finding on the schema that names the format.
- <a id="tl_110"></a>TL_110 [unwanted]: IF a format declares `variants` and an `example` of its own, THEN the engine SHALL report a finding on the schema that names the format.
- <a id="tl_41"></a>TL_41 [unwanted]: IF a patch names an entry that no farther layer defines, THEN the engine SHALL report a finding that names the entry.
- <a id="tl_97"></a>TL_97 [unwanted]: IF a rule binds a format that no layer declares, THEN the engine SHALL report a finding on the schema that names the format.
- <a id="tl_125"></a>TL_125 [ubiquitous]: the engine SHALL report a type or a format of a layer that does not load on its own declaration, and report no finding on a rule or a template that names it.
- <a id="tl_42"></a>TL_42 [ubiquitous]: the engine SHALL accept a line that conforms to one variant of its format, trying the variants in order, and report a line that conforms to none against the variant that it matches furthest, or each variant that it matches as far, with the example of each.
- <a id="tl_43"></a>TL_43 [unwanted]: IF the example of a variant matches its own variant and another variant of its format, THEN the engine SHALL report a finding at the format.
- <a id="tl_44"></a>TL_44 [unwanted]: IF a unit satisfies a rule or a bound format that declares a key, and the text of the unit contains no such key, THEN the engine SHALL report a finding that names the key that it expects.
- <a id="tl_45"></a>TL_45 [optional]: WHERE a rule sets `key-optional`, the engine SHALL accept a unit that contains no key of its bound format.
- <a id="tl_46"></a>TL_46 [ubiquitous]: the engine SHALL accept `key-optional` on a rule that declares `key` or binds a format that declares one.
- <a id="tl_47"></a>TL_47 [unwanted]: IF a rule sets `key-optional` and neither declares a key nor binds a format that declares one, THEN the engine SHALL report a finding on the schema.
- <a id="tl_48"></a>TL_48 [ubiquitous]: the engine SHALL check a unit against the prohibited patterns of the rule and of the format that govern it.
- <a id="tl_49"></a>TL_49 [invariant]: FOR ANY unit bound to a format that declares a key, the engine SHALL either capture the key or report one finding that names it, but never neither and never both.
- <a id="tl_50"></a>TL_50 [equivalent]: FOR ANY format entry and any patch of it, the engine SHALL produce a merged entry equal to RFC 7386 applied to the two as written.
- <a id="tl_51"></a>TL_51 [invariant]: FOR ANY two layers whose nearer entry declares no patch, the engine SHALL take the nearer entry as the effective entry, with nothing of the farther one.

## 6. Frontmatter and lifecycle

Frontmatter is the authority over the body. A repair rewrites the body to agree with the frontmatter, and the engine fills the body of a new document from its frontmatter. A lifecycle defines the states that a document moves through, each dimension a key of its frontmatter.

- <a id="tl_52"></a>TL_52 [ubiquitous]: the engine SHALL read frontmatter as YAML 1.2, core schema.
- <a id="tl_53"></a>TL_53 [unwanted]: IF frontmatter does not parse, THEN the engine SHALL report a finding on its first line and check no frontmatter rule.
- <a id="tl_123"></a>TL_123 [unwanted]: IF frontmatter contains an anchor, an alias, or a tag, THEN the engine SHALL report a finding on its first line and check no frontmatter rule.
- <a id="tl_94"></a>TL_94 [state]: WHILE a document has no frontmatter, the engine SHALL check it as a document whose frontmatter is an empty map.
- <a id="tl_54"></a>TL_54 [unwanted]: IF a frontmatter key that the schema declares, and does not mark `optional`, is missing, THEN the engine SHALL report a finding that names the key.
- <a id="tl_55"></a>TL_55 [unwanted]: IF a frontmatter key is not declared, not reserved by the host, and not the key of a lifecycle dimension, and the schema declares no `frontmatter-open`, THEN the engine SHALL report a finding that names the key.
- <a id="tl_56"></a>TL_56 [unwanted]: IF a frontmatter value, or an element of the value of a key that declares `list`, fails its `const`, `template`, `pattern`, `enum`, or `format`, THEN the engine SHALL report a finding that names the key.
- <a id="tl_106"></a>TL_106 [unwanted]: IF the value of a key that declares `list` has fewer elements than its `min`, THEN the engine SHALL report a finding that names the key, the bound, and the count.
- <a id="tl_107"></a>TL_107 [unwanted]: IF the value of a key that declares `list` has more elements than its `max`, THEN the engine SHALL report a finding that names the key, the bound, and the count.
- <a id="tl_57"></a>TL_57 [unwanted]: IF a key that declares `list` has a value that is not a list, or a key that does not declare `list`, or an element of a list, has a value that is not a scalar, THEN the engine SHALL report a finding that names the key.
- <a id="tl_58"></a>TL_58 [unwanted]: IF the template of `id` contains a hole other than a `seq` hole, a `slug` hole, and a hole with no type named for a lifecycle key, THEN the engine SHALL report a finding on the schema that names the template.
- <a id="tl_114"></a>TL_114 [unwanted]: IF the template of `id` contains more than one `seq` hole, THEN the engine SHALL report a finding on the schema.
- <a id="tl_115"></a>TL_115 [unwanted]: IF the constraint of `id` declares `const`, `enum`, `format`, `list`, `min`, or `max` beside its template, THEN the engine SHALL report a finding on the schema that names the key.
- <a id="tl_116"></a>TL_116 [unwanted]: IF `seq-width` sits beside the template of a key other than `id`, or beside an `id` template with no `seq` hole, THEN the engine SHALL report a finding on the schema.
- <a id="tl_122"></a>TL_122 [unwanted]: IF the template or the `const` of `id` admits an id that is not a slug, THEN the engine SHALL report a finding on the schema that names the id or the state that is not a slug.
- <a id="tl_59"></a>TL_59 [unwanted]: IF the template of `id` contains a hole named for a lifecycle key and the id of a document has another state at that hole than the state of the document for that key, THEN the engine SHALL report a finding that names the key.
- <a id="tl_60"></a>TL_60 [ubiquitous]: the engine SHALL ask the host whether a value matches a frontmatter format that the host added.
- <a id="tl_61"></a>TL_61 [ubiquitous]: the engine SHALL read the state of a document as the value of the key of each lifecycle dimension in its frontmatter.
- <a id="tl_62"></a>TL_62 [unwanted]: IF the key of a lifecycle dimension is missing, THEN the engine SHALL report a finding that names the key.
- <a id="tl_108"></a>TL_108 [unwanted]: IF the value of the key of a lifecycle dimension is not one of the `states` of its dimension, THEN the engine SHALL report a finding that names the key and the states.
- <a id="tl_95"></a>TL_95 [unwanted]: IF a lifecycle declares one dimension twice, THEN the engine SHALL report a finding on the schema that names the dimension.
- <a id="tl_112"></a>TL_112 [unwanted]: IF the frontmatter of a schema declares the key of a lifecycle dimension, THEN the engine SHALL report a finding on the schema that names the key.

## 7. Examples and guidance

The example of a rule is text of the kind that the rule demands. The example shows the reader of a finding what to write, and the scaffold copies it. What an example cannot say is guidance, which the scaffold places before the part that it guides.

- <a id="tl_63"></a>TL_63 [ubiquitous]: the engine SHALL accept `example` on a frontmatter constraint and on each rule of the format grammar that has a shape.
- <a id="tl_64"></a>TL_64 [unwanted]: IF the `example` of a rule fails the rule that declares it, THEN the engine SHALL report a finding on the schema that names the rule.
- <a id="tl_65"></a>TL_65 [unwanted]: IF the `example` of a format fails the shape, a prohibited pattern, or the key of that format, or the example of a variant fails that variant, THEN the engine SHALL report a finding at the format that names it.
- <a id="tl_66"></a>TL_66 [unwanted]: IF a rule other than the frontmatter constraint of `id` declares `template`, `pattern`, or `source-pattern`, and declares no `example` and binds no format that declares one, THEN the engine SHALL report a finding on the schema that names the key that declares the shape.
- <a id="tl_67"></a>TL_67 [ubiquitous]: the engine SHALL demand no `example` of a rule that declares only a `select-pattern`, a count, or a `text`.
- <a id="tl_68"></a>TL_68 [unwanted]: IF the `guidance` and the `example` of a rule, or the guidance of a section or a block and the example of its heading or of a unit rule inside it, contain the same words, THEN the engine SHALL report a finding on the schema that names both keys.
- <a id="tl_69"></a>TL_69 [unwanted]: IF an entry of `examples`, checked as a complete document at its own states and with no markers, fails the schema, THEN the engine SHALL report a finding on the schema that names the entry and the failure.
- <a id="tl_70"></a>TL_70 [ubiquitous]: the engine SHALL accept `guidance` on a schema, a frontmatter constraint, a lifecycle dimension, and each rule map of the format grammar that declares the key.
- <a id="tl_71"></a>TL_71 [ubiquitous]: the engine SHALL print no `guidance` in a finding.

## 8. Findings

A finding has a file, a line where one applies, a severity, the key of the criterion that it breaks, and a message. The key is how a reader looks the finding up. Where a rule depends on a fact outside the document, the engine asks the host. The null host supplies no fact beyond the document.

- <a id="tl_72"></a>TL_72 [ubiquitous]: the engine SHALL give each finding the key of the most specific criterion that it breaks, the criterion of a format over a criterion of this specification, and end the message of a finding that a schema rule raises with `(schema <id>, <where>)`.
- <a id="tl_100"></a>TL_100 [ubiquitous]: the engine SHALL give each finding the key of an unwanted criterion that describes one fault, which one kind of change to the document or the schema fixes.
- <a id="tl_73"></a>TL_73 [ubiquitous]: the engine SHALL name in each finding that a schema rule raises the rule that it came from, by `rule-id` where the rule declares one, and by its place in the schema, its key, or its pattern otherwise, a section by its heading as the schema writes it.
- <a id="tl_74"></a>TL_74 [unwanted]: IF a text fails the shape of a rule or a format, THEN the engine SHALL report the text, the shape, or the column where a template stops matching, and the example of that shape.
- <a id="tl_75"></a>TL_75 [unwanted]: IF a text matches a prohibited pattern, THEN the engine SHALL report the span that the pattern matched and the message of the ban.
- <a id="tl_76"></a>TL_76 [unwanted]: IF a text matches a prohibited pattern of a rule or a format, THEN the engine SHALL report the example of that rule beside the span and the message.
- <a id="tl_77"></a>TL_77 [ubiquitous]: the engine SHALL include in a finding no text that a schema writes to explain it.
- <a id="tl_78"></a>TL_78 [ubiquitous]: the host SHALL handle a scope, a format, or a block kind that it added.
- <a id="tl_79"></a>TL_79 [unwanted]: IF the host cannot supply what a rule needs to know, THEN the engine SHALL skip the rule.
- <a id="tl_80"></a>TL_80 [invariant]: FOR ANY format and any line that it rejects on shape, the engine SHALL report text that contains the example of the expected shape and nothing else that a schema wrote.
- <a id="tl_81"></a>TL_81 [invariant]: FOR ANY ban and any line that it matches, the engine SHALL report a span that is a substring of the line and that the pattern of the ban matches.

## 9. Repairs

A repair is an edit: a replacement of lines of the text as the engine read it. The format grammar defines which parts of its documents the engine repairs. A host builds edits of its own, which write its generated content.

- <a id="tl_82"></a>TL_82 [ubiquitous]: the engine SHALL apply all edits of one text in one pass.
- <a id="tl_83"></a>TL_83 [unwanted]: IF two edits target one line, or an edit extends beyond the text, THEN the engine SHALL reject the edits.
- <a id="tl_84"></a>TL_84 [ubiquitous]: the engine SHALL apply an insertion at a line before a replacement at the same line.

## 10. The command

The command `tramlines validate` checks documents against one schema, with the null host and the default dialect of each format. Its options are `--schema <FILE>`, `--formats <FILE>`, and `--format <NAME>`, then the documents. This section states what the schema language requires of a check. The command's repairs, its other subcommands, and the forms of its report are outside the language.

```sh
tramlines validate --schema contract.yaml --formats formats.yaml docs/contract-cache.md
```

- <a id="tl_85"></a>TL_85 [ubiquitous]: the command SHALL check each document that it names against the schema that `--schema` names, with the null host.
- <a id="tl_86"></a>TL_86 [event]: WHEN `--formats` names a file, the command SHALL read it as a `registry` map of formats and an optional `types` map, the layer under the schema.
- <a id="tl_87"></a>TL_87 [unwanted]: IF the file that `--formats` names does not parse or has no `registry` key, THEN the command SHALL exit with code 2 and name the file.
- <a id="tl_124"></a>TL_124 [unwanted]: IF the registry or the types of the file that `--formats` names break a criterion that applies to the `formats` or the `types` of a schema, THEN the command SHALL exit with code 2 and print each finding with the key of that criterion and the name of the file.
- <a id="tl_88"></a>TL_88 [ubiquitous]: the command SHALL select the format of a document by the extension of its file, `.md` for markdown.
- <a id="tl_89"></a>TL_89 [event]: WHEN `--format` names a format, the command SHALL read every document in that format.
- <a id="tl_90"></a>TL_90 [unwanted]: IF a document has an extension that no format matches and the command has no `--format`, THEN the command SHALL exit with code 2 and name the document.
- <a id="tl_91"></a>TL_91 [unwanted]: IF `--format` names no format, THEN the command SHALL exit with code 2 and name the formats that it knows.
- <a id="tl_92"></a>TL_92 [state]: WHILE the command line asks for no other form of report, the command SHALL print each error, and each warning that the command line asks it to list, as one line `path:line: severity[KEY]: message`, then a count of errors and warnings.
- <a id="tl_93"></a>TL_93 [ubiquitous]: the command SHALL exit with code 0 when no finding is an error, 1 when a finding is an error, and 2 for a usage error or a file that the command fails to read.
- <a id="tl_99"></a>TL_99 [unwanted]: IF the command stops on a criterion of this specification, THEN the command SHALL print the error as `error[KEY]: message`.

<!-- sokf:links -->

[sokf:tramlines-md-spec]: /knowledge/spec/tramlines-md-spec.md
