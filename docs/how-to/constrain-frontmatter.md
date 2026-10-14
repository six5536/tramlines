---
type: HowTo
id: how-to-constrain-frontmatter
title: Constrain frontmatter and ids
description: Require frontmatter keys, limit the values they can take, bound lists, and give the id of a document a template.
order: 40
---

# Constrain frontmatter and ids

When the frontmatter of a document must have certain keys with certain values, declare each key under `frontmatter` and constrain its value. At the end, a document with a missing key, an unknown key, or a value of the wrong shape fails with an error that names the key.

In this guide, you write the frontmatter of a requirement list: a fixed type, a numbered id, a code in capitals, a status, one or two links, and optional tags.

## Before you begin

- A schema with a section, as in [Your first schema][sokf:tutorial-first-schema].
- The shapes of a line, from [Write the shape of a line][sokf:how-to-write-line-shapes]. A frontmatter value takes the same shapes.

## Steps

1. Declare each key under `frontmatter`. The check requires every declared key unless you set `optional: true` on it. A key that the schema doesn't declare is an error.

2. Constrain each value with one of these keys:

   - `const`: the value must equal the given value.
   - `enum`: the value must be one of the listed values.
   - `template` or `pattern`: the value must match the shape. Give the key an `example`, as on any rule.
   - `format`: `url` for an absolute URL, or `slug` for lowercase letters and digits joined by single hyphens.

3. For a key whose value is a list, set `list: true`. The constraint then applies to every element, and `min` and `max` bound the number of elements. A list where no list is declared, or a single value where a list is declared, is an error.

4. Give `id` a template. The template uses only the holes that the engine fills when it writes a new document:

   - `${:seq}`: a sequence number, unique within the kind of document. A template can contain at most one. The number is three digits wide unless you set `seq-width` beside the template.
   - `${:slug}`: one or more slug words.
   - `${<key>}`: a hole with no type, named for a lifecycle key. It holds the state of the document for that key. Step 7 shows an example.

   An `id` template takes no `example`. For a kind that has only one document, set the id with `const` instead.

**`requirements.yaml`**

```yaml
line-limit: 0
types:
  code: '[A-Z]{2,6}'
frontmatter:
  type:
    const: Requirements
  id:
    template: 'req-${:seq}-${:slug}'
  code:
    template: '${:code}'
    example: LINK
  status:
    enum: [draft, approved]
  homepages:
    list: true
    format: url
    min: 1
    max: 2
    example: https://example.com/link-checker
  tags:
    list: true
    format: slug
    optional: true
    example: parsing
sections:
  - heading: Link checker
    level: 1
    blocks:
      - kind: prose
        example: The link checker follows every link in a document.
```

5. Check a document. Each error names the frontmatter key that fails:

**`link-checker.md`**

```markdown
---
type: Requirements
id: req-7-link-checker
code: Link
status: final
homepages: https://example.com/link-checker
tags: [Parsing, links]
owner: Sam
---

# Link checker

The link checker follows every link in a document.
```

```console
$ tramlines validate --schema requirements.yaml link-checker.md
link-checker.md:3: error[TL_56]: id `req-7-link-checker` does not match the template `req-${:seq}-${:slug}` (schema requirements, key `id`)
link-checker.md:4: error[TL_56]: `code` `Link` fails at column 1: expected a `code` (`[A-Z]{2,6}`), found `Link`; a value of this kind reads LINK (schema requirements, key `code`)
link-checker.md:5: error[TL_56]: `status` must be one of `draft`, `approved`; `final` is not (schema requirements, key `status`)
link-checker.md:6: error[TL_57]: `homepages` must be a list (schema requirements, key `homepages`)
link-checker.md:7: error[TL_56]: `tags` must be a slug; `Parsing` is not (schema requirements, key `tags`)
link-checker.md:8: error[TL_55]: frontmatter key `owner` is not declared (schema requirements)
6 errors, 0 warnings
[exit 1]
```

The id has a one-digit sequence number where the template needs three, `code` isn't in capitals, `final` isn't a status, `homepages` isn't a list, `Parsing` isn't a slug, and `owner` isn't declared. This version of the document corrects each one:

**`link-checker.md`**

```markdown
---
type: Requirements
id: req-007-link-checker
code: LINK
status: draft
homepages: [https://example.com/link-checker]
tags: [parsing, links]
---

# Link checker

The link checker follows every link in a document.
```

```console
$ tramlines validate --schema requirements.yaml link-checker.md
0 errors, 0 warnings
```

6. To accept keys that the schema doesn't declare, set `frontmatter-open: true` on the schema. The command still checks the declared keys. With `frontmatter-open: true`, the command accepts `owner`:

**`requirements.yaml`**

```yaml
line-limit: 0
frontmatter-open: true
types:
  code: '[A-Z]{2,6}'
frontmatter:
  type:
    const: Requirements
  id:
    template: 'req-${:seq}-${:slug}'
  code:
    template: '${:code}'
    example: LINK
  status:
    enum: [draft, approved]
  homepages:
    list: true
    format: url
    min: 1
    max: 2
    example: https://example.com/link-checker
  tags:
    list: true
    format: slug
    optional: true
    example: parsing
sections:
  - heading: Link checker
    level: 1
    blocks:
      - kind: prose
        example: The link checker follows every link in a document.
```

**`owned.md`**

```markdown
---
type: Requirements
id: req-008-owned-links
code: LINK
status: draft
homepages: [https://example.com/link-checker]
owner: Sam
---

# Link checker

The link checker follows every link in a document.
```

```console
$ tramlines validate --schema requirements.yaml owned.md
0 errors, 0 warnings
```

7. To put a state of the document in its id, name a hole of the `id` template after a lifecycle key. [Apply a rule in some states only][sokf:how-to-gate-rules-by-state] shows how to declare one. The id must then contain the state of the document for that key. This schema requires a four-digit number in the id of each issue:

**`issue.yaml`**

```yaml
line-limit: 0
lifecycle:
  - { key: kind, states: [bug, feature] }
frontmatter:
  id:
    template: 'issue-${:seq}-${kind}-${:slug}'
    seq-width: 4
sections:
  - heading: Issue
    level: 1
    blocks:
      - kind: prose
        example: A list item indented with a tab is read as a code block.
```

**`tab.md`**

```markdown
---
id: issue-0007-feature-tab
kind: bug
---

# Issue

A list item indented with a tab is read as a code block.
```

```console
$ tramlines validate --schema issue.yaml tab.md
tab.md:2: error[TL_59]: id `issue-0007-feature-tab` holds `feature` for `kind`, but `kind` is `bug` (schema issue, key `id`)
1 error, 0 warnings
[exit 1]
```

The command reserves no frontmatter key. A program that embeds the engine can reserve its own keys. The check never reports a reserved key as undeclared. A schema can still declare a reserved key to require it or to narrow its value.

## Troubleshooting

- `TL_54`: a declared key is missing. Add it, or set `optional: true` on the key. See [TL_54][sokf:errors-frontmatter#tl_54].
- `TL_55`: a key isn't declared. Remove it, declare it, or set `frontmatter-open: true`. See [TL_55][sokf:errors-frontmatter#tl_55].
- `TL_56`: a value fails its `const`, `template`, `pattern`, `enum`, or `format`. See [TL_56][sokf:errors-frontmatter#tl_56].
- `TL_106`, `TL_107`: a list has fewer elements than its `min`, or more than its `max`. See [TL_106][sokf:errors-frontmatter#tl_106] and [TL_107][sokf:errors-frontmatter#tl_107].
- `TL_57`: a list is declared and the value is a single value, or the other way round. See [TL_57][sokf:errors-frontmatter#tl_57].
- `TL_58`: the `id` template contains a hole that the engine can't fill for a new document. See [TL_58][sokf:errors-lifecycle-and-ids#tl_58].
- `TL_114`: the `id` template contains more than one `${:seq}`. See [TL_114][sokf:errors-lifecycle-and-ids#tl_114].
- `TL_115`: the `id` declares another key beside its template. See [TL_115][sokf:errors-lifecycle-and-ids#tl_115].
- `TL_59`: the id contains a state other than the state of the document for that key. See [TL_59][sokf:errors-lifecycle-and-ids#tl_59].
- `TL_53`: the frontmatter isn't valid YAML. See [TL_53][sokf:errors-frontmatter#tl_53].
- `TL_123`: the frontmatter contains an anchor, an alias, or a tag. See [TL_123][sokf:errors-frontmatter#tl_123].

## See also

- [Schema keys: `frontmatter-constraint`][sokf:reference-schema-keys#frontmatter-constraint]
- [Built-in types][sokf:reference-types]
- [Frontmatter as the authority over the body][sokf:explanation-frontmatter-authority]
- [Section 6 of the core specification][sokf:tramlines:tramlines-spec#6-frontmatter-and-lifecycle]

<!-- sokf:links -->

[sokf:errors-frontmatter#tl_106]: /docs/reference/errors-frontmatter.md#tl_106
[sokf:errors-frontmatter#tl_107]: /docs/reference/errors-frontmatter.md#tl_107
[sokf:errors-frontmatter#tl_123]: /docs/reference/errors-frontmatter.md#tl_123
[sokf:errors-frontmatter#tl_53]: /docs/reference/errors-frontmatter.md#tl_53
[sokf:errors-frontmatter#tl_54]: /docs/reference/errors-frontmatter.md#tl_54
[sokf:errors-frontmatter#tl_55]: /docs/reference/errors-frontmatter.md#tl_55
[sokf:errors-frontmatter#tl_56]: /docs/reference/errors-frontmatter.md#tl_56
[sokf:errors-frontmatter#tl_57]: /docs/reference/errors-frontmatter.md#tl_57
[sokf:errors-lifecycle-and-ids#tl_114]: /docs/reference/errors-lifecycle-and-ids.md#tl_114
[sokf:errors-lifecycle-and-ids#tl_115]: /docs/reference/errors-lifecycle-and-ids.md#tl_115
[sokf:errors-lifecycle-and-ids#tl_58]: /docs/reference/errors-lifecycle-and-ids.md#tl_58
[sokf:errors-lifecycle-and-ids#tl_59]: /docs/reference/errors-lifecycle-and-ids.md#tl_59
[sokf:explanation-frontmatter-authority]: /docs/explanation/frontmatter-authority.md
[sokf:how-to-gate-rules-by-state]: /docs/how-to/gate-rules-by-state.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-schema-keys#frontmatter-constraint]: /docs/reference/schema-keys.md#frontmatter-constraint
[sokf:reference-types]: /docs/reference/types.md
[sokf:tramlines:tramlines-spec#6-frontmatter-and-lifecycle]: /knowledge/spec/tramlines-spec.md#6-frontmatter-and-lifecycle
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
