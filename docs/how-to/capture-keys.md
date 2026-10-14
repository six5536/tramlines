---
type: HowTo
id: how-to-capture-keys
title: Capture keys and anchor them
description: Capture an id from a heading, an item, or a line, keep it unique, derive it from another key, require a section or an item for each one, and write its anchor.
order: 90
---

# Capture keys and anchor them

When the parts of a document have ids, such as requirements named `CART-1` and `CART-2`, capture each id as a key. At the end, every key is unique and has an anchor that a link can open. The document also contains the sections or items that the schema pairs with each key.

## Before you begin

- A schema with sections and blocks. [Describe blocks and their units][sokf:how-to-describe-blocks] shows how to write one.
- A template with a named hole, as in [Write the shape of a line][sokf:how-to-write-line-shapes].

## Steps

1. Set `key` on the rule that matches the id, beside its shape. The value of `key` names a hole of the template, or a named group of the pattern. A heading rule captures the key from the heading. A unit rule captures the key from each item, row, or line that it matches. A unit can match the shape without a key, as with a format whose variants don't all capture one. To accept such a unit as plain text, set `key-optional: true`.

2. Give the block a `rule-id`, so that other rules can refer to the keys that it captures. Each key must be unique within the document. A second capture of a key is an error. The `unique` schema key names the scope. Its default, `document`, is the only scope that the command checks. A program that embeds the engine can add wider scopes and check them itself.

3. To require that each key starts with a known prefix, add `key-derives-from` beside `key`. `{ from: code, separator: "-" }` requires the part before the first `-` to equal the frontmatter value of `code`. `{ rule: reqs, separator: "_" }` requires the part before the first `_` to be a key that the rule `reqs` captured. With it, `CART-1_AC-1` derives from `CART-1`. This schema describes the list of requirements of one area:

**`requirements.yaml`**

```yaml
line-limit: 80
frontmatter:
  code:
    example: CART
sections:
  - heading: Requirements
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: reqs
        each:
          template: "${id}: ${}"
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: "CART-1: The cart keeps its items for a week."
```

4. Check a document. Every key needs an anchor, `<a id="slug"></a>`. The error for a missing anchor shows the anchor to write. In this document, the first item has its anchor, the second has none, the third has the wrong prefix, and the fourth repeats the key of the second:

**`cart.md`**

```markdown
---
code: CART
---

# Requirements

- <a id="cart-1"></a>CART-1: The cart keeps its items for a week.
- CART-2: The cart shows the total price.
- SHOP-1: The shop opens at nine.
- CART-2: The cart shows the tax.
```

```console
$ tramlines validate --schema requirements.yaml cart.md
cart.md:8: error[TLMD_109]: key `CART-2` has no anchor; its repair writes `<a id="cart-2"></a>` before it
cart.md:9: error[TL_33]: key `SHOP-1` derives from `SHOP`, which is not `CART`, the value of the frontmatter `code` (schema requirements, rule `reqs`)
cart.md:9: error[TLMD_109]: key `SHOP-1` has no anchor; its repair writes `<a id="shop-1"></a>` before it
cart.md:10: error[TLMD_148]: key `CART-2` is captured more than once; it was first captured at line 8 (schema requirements, rule `reqs`)
4 errors, 0 warnings
[exit 1]
```

5. Write each anchor that the messages name. The command `tramlines validate --fix` writes every anchor that a key needs, then reports what is left. To write an anchor by hand, give it the slug of the key as its id. The slug is the key in lowercase, with letters, digits, `-`, and `_` kept, spaces changed to `-`, and every other character dropped. Put the anchor in the place that fits its key:

   - In an item, a row, or a paragraph: directly before the text that the rule matches, or before the code span or link that contains the key.
   - In a heading: at the start of the text of the heading, after the `#` marks. A heading whose own slug is the slug of the key needs no anchor.
   - For a line of a code or `html` block: on a line of its own before the block, one line for each key in the order of the lines, then a blank line.

   No rule matches an anchor, because tramlines removes each anchor before it matches the text.

6. Fix the document: add the anchor of `CART-2`, give the third requirement its own id, and remove the duplicate.

**`cart.md`**

```markdown
---
code: CART
---

# Requirements

- <a id="cart-1"></a>CART-1: The cart keeps its items for a week.
- <a id="cart-2"></a>CART-2: The cart shows the total price.
- <a id="cart-3"></a>CART-3: The cart opens at nine.
```

```console
$ tramlines validate --schema requirements.yaml cart.md
0 errors, 0 warnings
```

7. To require a section for each key, give the section three keys: a heading rule that captures a key, `repeat: true`, and `for-each-key-of`. The value of `for-each-key-of` is the `rule-id` of the rule that captured the keys. A key with no section is an error. A section whose key the rule didn't capture is an error too. When a heading captures a key, the anchor belongs on the heading instead of the item:

**`requirements.yaml`**

```yaml
line-limit: 80
frontmatter:
  code:
    example: CART
sections:
  - heading: Requirements
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: reqs
        each:
          template: "${id}: ${}"
          key: id
          key-derives-from: { from: code, separator: "-" }
          example: "CART-1: The cart keeps its items for a week."
  - heading: { template: "${id} tests", key: id, example: "CART-1 tests" }
    level: 2
    repeat: true
    for-each-key-of: reqs
    blocks:
      - kind: bullet-list
        example: Add an item and reopen the cart a day later.
```

**`cart.md`**

```markdown
---
code: CART
---

# Requirements

- CART-1: The cart keeps its items for a week.
- CART-2: The cart shows the total price.

## <a id="cart-1"></a>CART-1 tests

- Add an item and reopen the cart a day later.

## <a id="cart-3"></a>CART-3 tests

- Check the total.
```

```console
$ tramlines validate --schema requirements.yaml cart.md
cart.md:8: error[TLMD_109]: key `CART-2` has no anchor; its repair writes `<a id="cart-2"></a>` before it
cart.md:8: error[TLMD_95]: no `${id} tests` section for key `CART-2` (schema requirements, section "${id} tests")
cart.md:14: error[TLMD_96]: section names key `CART-3`, which rule `reqs` did not capture (schema requirements, section "${id} tests")
3 errors, 0 warnings
[exit 1]
```

`CART-2` has no section, so its anchor still belongs on its item. The last section describes a test of the total, so it belongs to `CART-2`. Change its heading to `<a id="cart-2"></a>CART-2 tests`, and the document passes.

8. To require an item for each key, set `for-each-key-of` on the unit rule instead. The rule must capture a key itself. A key with no item is an error, even when the whole block is absent.

   A key captured from a line of a code block belongs to the `rule-id` of its line rule. When the line rule has no `rule-id`, the key belongs to the `rule-id` of the block. When two kinds of line in one block each pair with a different rule, give the unit rule or the line rule its own `rule-id`. In this example, each service of a configuration file needs an owner:

**`services.yaml`**

```yaml
line-limit: 60
sections:
  - heading: Services
    level: 1
    blocks:
      - kind: code
        lang: yaml
        rule-id: services
        each:
          lines:
            - text: "services:"
            - template: "${name:slug}:"
              key: name
              example: "web:"
            - template: "image: ${}"
              example: "image: nginx"
  - heading: Owners
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: "${name:slug}: ${}"
          key: name
          for-each-key-of: services
          example: "web: Ana"
```

**`services.md`**

````markdown
# Services

<a id="web"></a>
<a id="db"></a>

```yaml
services:
  web:
    image: nginx
  db:
    image: postgres
```

## Owners

- web: Ana
- db: Ben
````

```console
$ tramlines validate --schema services.yaml services.md
0 errors, 0 warnings
```

9. In a section that declares a `parent`, a pairing uses only the keys captured in the same occurrence of the parent, so each occurrence pairs with its own keys. Elsewhere it uses the keys of the whole document. [Describe sections and headings][sokf:how-to-describe-sections] shows `parent`.

10. To pair with the keys of several rules, give `for-each-key-of` a list of `rule-id`s. Each distinct key of the rules needs one section or item. A rule that names one key on several units, such as the owner of each item, declares `key-repeats: true`. None of its captures then counts as a second capture. In this map, each group needs a design, and so does each block that provides a contract. The guarded rule `providers` reads the block from the line that the rule `contracts` takes. `Loading` is both a group and a provider, and needs one design:

**`map.yaml`**

```yaml
line-limit: 60
sections:
  - heading: Map
    level: 1
    blocks:
      - kind: bullet-list
        rule-id: groups
        each:
          template: "Group ${name:word}"
          key: name
          example: "Group Loading"
      - kind: numbered-list
        rules:
          - template: "${name:slug} (${:word})"
            key: name
            rule-id: contracts
            example: "format (Loading)"
          - select-pattern: '\('
            pattern: '\((?P<block>[A-Z][a-z]+)\)$'
            key: block
            key-repeats: true
            rule-id: providers
            min: 0
            example: "format (Loading)"
  - heading: Designs
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: "${name:word} design"
          key: name
          for-each-key-of: [groups, providers]
          example: "Loading design"
```

**`map.md`**

```markdown
# Map

- <a id="loading"></a>Group Loading

1. <a id="format"></a>format (Loading)
2. <a id="rows"></a>rows <a id="cache"></a>(Cache)
3. <a id="keys"></a>keys (Cache)

## Designs

- Loading design
- Cache design
```

```console
$ tramlines validate --schema map.yaml map.md
0 errors, 0 warnings
```

The anchor of `Cache` sits on its first capture. The anchor of `Loading` sits on the group, because a rule without `key-repeats` defines a key at the place where it captures the key.

## Troubleshooting

- `TLMD_109`: a key has no anchor, an anchor isn't where its key is, an anchor names no key, or an anchor is written twice. Write or move the anchor as the message says. See [TLMD_109][sokf:errors-keys-and-repairs#tlmd_109].
- `TLMD_101`: two keys, or a key and a heading, have the same slug and would share one anchor. Change one of them. See [TLMD_101][sokf:errors-keys-and-repairs#tlmd_101].
- `TLMD_148`: a rule captures one key twice. See [TLMD_148][sokf:errors-keys-and-repairs#tlmd_148].
- `TL_33`: the prefix of a key isn't one that its `key-derives-from` allows. See [TL_33][sokf:errors-schema-types-keys-lifecycle#tl_33].
- `TL_44`: a line has the shape of its rule but no key. Add the key, or set `key-optional` on the rule. See [TL_44][sokf:errors-formats-and-examples#tl_44].
- `TL_31`: the `key` names no hole or named group of the shape. See [TL_31][sokf:errors-schema-types-keys-lifecycle#tl_31].
- `TLMD_95`, `TLMD_96`: a key has no section or item that pairs with it, or a section or item names a key that the rule didn't capture. See [Errors: lines and pairings][sokf:errors-lines-and-pairings].
- `TLMD_98`: a rule with `for-each-key-of` captures no key. See [TLMD_98][sokf:errors-lines-and-pairings#tlmd_98].
- `TLMD_167`: the rule that a `for-each-key-of` names captures no key. See [TLMD_167][sokf:errors-lines-and-pairings#tlmd_167].

## See also

- [Schema keys: `unit`][sokf:reference-schema-keys#unit]
- [Schema keys: `heading`][sokf:reference-schema-keys#heading]
- [Schema keys: `key-derivation`][sokf:reference-schema-keys#key-derivation]
- [Errors: keys, markers, and scaffolds][sokf:errors-keys-markers-scaffolds]
- [Section 7 of the markdown specification][sokf:tramlines:tramlines-md-spec#7-keys-and-anchors]

<!-- sokf:links -->

[sokf:errors-formats-and-examples#tl_44]: /docs/reference/errors-formats-and-examples.md#tl_44
[sokf:errors-keys-and-repairs#tlmd_101]: /docs/reference/errors-keys-and-repairs.md#tlmd_101
[sokf:errors-keys-and-repairs#tlmd_109]: /docs/reference/errors-keys-and-repairs.md#tlmd_109
[sokf:errors-keys-and-repairs#tlmd_148]: /docs/reference/errors-keys-and-repairs.md#tlmd_148
[sokf:errors-keys-markers-scaffolds]: /docs/reference/errors-keys-markers-scaffolds.md
[sokf:errors-lines-and-pairings]: /docs/reference/errors-lines-and-pairings.md
[sokf:errors-lines-and-pairings#tlmd_167]: /docs/reference/errors-lines-and-pairings.md#tlmd_167
[sokf:errors-lines-and-pairings#tlmd_98]: /docs/reference/errors-lines-and-pairings.md#tlmd_98
[sokf:errors-schema-types-keys-lifecycle#tl_31]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_31
[sokf:errors-schema-types-keys-lifecycle#tl_33]: /docs/reference/errors-schema-types-keys-lifecycle.md#tl_33
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-describe-sections]: /docs/how-to/describe-sections.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-schema-keys#heading]: /docs/reference/schema-keys.md#heading
[sokf:reference-schema-keys#key-derivation]: /docs/reference/schema-keys.md#key-derivation
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
[sokf:tramlines:tramlines-md-spec#7-keys-and-anchors]: /knowledge/spec/tramlines-md-spec.md#7-keys-and-anchors
