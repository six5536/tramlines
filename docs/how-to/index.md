# how-to

- [Ban text in a document][sokf:how-to-ban-text] - Ban text from every line of a document, from one kind of line, or from every line of a format, with `prohibited-patterns`.
- [Capture keys and anchor them][sokf:how-to-capture-keys] - Capture an id from a heading, an item, or a line, keep it unique, derive it from another key, require a section or an item for each one, and write its anchor.
- [Constrain frontmatter and ids][sokf:how-to-constrain-frontmatter] - Require frontmatter keys, limit the values they can take, bound lists, and give the id of a document a template.
- [Describe blocks and their units][sokf:how-to-describe-blocks] - Declare the paragraphs, lists, tables, code blocks, and images of a section, the number of units in each, and the shape of each unit.
- [Describe sections and headings][sokf:how-to-describe-sections] - List the headings that a document must contain, which of them are optional or repeat, where writers can add headings of their own, and what comes before the first heading.
- [Embed the engine in a Rust program][sokf:how-to-embed-the-engine] - Use the tramlines-md library to compile a schema, check documents, repair them, derive a starter document, and add schema keys of your own.
- [Apply a rule in some states only][sokf:how-to-gate-rules-by-state] - Declare the states that a document moves through, and gate a section, a block, or a frontmatter key to some of them with `only`.
- [Read an error and fix the document][sokf:how-to-read-a-finding] - Read each part of an error from tramlines, find what its key means and which rule of the schema it breaks, and change the document so that it passes.
- [Repair a document and start a new one][sokf:how-to-repair-and-start-documents] - Start a document from the scaffold of its schema with `tramlines new`, and let `tramlines validate --fix` rewrite the headings and anchors that the schema derives.
- [Select and count units and lines][sokf:how-to-select-and-count-lines] - Write the rules of a block whose units come in more than one kind, pick units by a label, count or ban them, and check the lines of a code block.
- [Share line formats across schemas][sokf:how-to-share-line-formats] - Define the shape of a line once in a formats file, so that two schemas check a kind of line in the same way.
- [Write examples and guidance][sokf:how-to-write-examples-and-guidance] - Give each rule an example that shows the text it requires, and guidance for what no example can show, so that the error for every failure shows the writer what to write.
- [Write the shape of a line][sokf:how-to-write-line-shapes] - Describe the text of a heading, a paragraph, a list item, or a frontmatter value with exact text, a template with typed holes, or a regular expression.

<!-- sokf:links -->

[sokf:how-to-ban-text]: /docs/how-to/ban-text.md
[sokf:how-to-capture-keys]: /docs/how-to/capture-keys.md
[sokf:how-to-constrain-frontmatter]: /docs/how-to/constrain-frontmatter.md
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:how-to-embed-the-engine]: /docs/how-to/embed-the-engine.md
[sokf:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:how-to-read-a-finding]: /docs/how-to/read-a-finding.md
[sokf:how-to-repair-and-start-documents]: /docs/how-to/repair-and-start-documents.md
[sokf:how-to-select-and-count-lines]: /docs/how-to/select-and-count-lines.md
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:how-to-write-examples-and-guidance]: /docs/how-to/write-examples-and-guidance.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
