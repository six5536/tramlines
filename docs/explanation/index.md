# explanation

- [How blocks are matched to a schema][sokf:explanation-block-matching] - How tramlines pairs the blocks of a section with the blocks that its schema declares, and how that pairing determines which block an error lands on and which kind it names.
- [Frontmatter as the authority over the body][sokf:explanation-frontmatter-authority] - Why the frontmatter of a document is the authority over the text of its body, how a schema binds body text to frontmatter values, and how the lifecycle states in the frontmatter determine which rules apply.
- [How the grammar layers fit together][sokf:explanation-grammar-layers] - Why a schema is checked against three grammars merged into one, and what each one can add.
- [What tramlines doesn't do][sokf:explanation-non-goals] - The things tramlines leaves out on purpose, why each is left out, and where that work belongs instead.
- [Pitfalls in writing a schema][sokf:explanation-pitfalls] - The mistakes that schema authors make most often, why each one goes wrong, and the error or silence it causes.
- [How a schema derives its scaffold][sokf:explanation-scaffolds] - Why the engine builds a starter document from every schema, what the starter contains, and why a starter that fails the check is an error in its schema.
- [Strict by default][sokf:explanation-strict-by-default] - Why tramlines requires every part that a schema declares and refuses every part that it doesn't, and how each relaxation loosens one rule at a time.

<!-- sokf:links -->

[sokf:explanation-block-matching]: /docs/explanation/block-matching.md
[sokf:explanation-frontmatter-authority]: /docs/explanation/frontmatter-authority.md
[sokf:explanation-grammar-layers]: /docs/explanation/grammar-layers.md
[sokf:explanation-non-goals]: /docs/explanation/non-goals.md
[sokf:explanation-pitfalls]: /docs/explanation/pitfalls.md
[sokf:explanation-scaffolds]: /docs/explanation/scaffolds.md
[sokf:explanation-strict-by-default]: /docs/explanation/strict-by-default.md
