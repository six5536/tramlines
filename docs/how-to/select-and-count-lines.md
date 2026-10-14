---
type: HowTo
id: how-to-select-and-count-lines
title: Select and count units and lines
description: Write the rules of a block whose units come in more than one kind, pick units by a label, count or ban them, and check the lines of a code block.
order: 80
---

# Select and count units and lines

When one list mixes items of several kinds, or a code block must contain certain lines, write the grammar of the block with `rules` or `lines`. At the end, the rule of its kind takes each unit or line, and each kind stays within the count that you set. Anything else fails at its line.

## Before you begin

- A schema with blocks. [Describe blocks and their units][sokf:how-to-describe-blocks] shows how to write one.
- Templates and patterns, as in [Write the shape of a line][sokf:how-to-write-line-shapes].

## Steps

1. Replace `each` or `sequence` with `rules`, a list of unit rules. A rule must take every unit. A rule acts in one of four ways:

   - A rule with a shape and no `select-pattern` is a production. It takes the units that its shape matches. Each unit goes to the first production that matches it, so list the narrowest production first.
   - A rule with a `select-pattern` is a guard. It takes the units whose text the regular expression matches, and checks them. Use a guard to pick units by a label rather than by their shape, such as an item whose nested blocks you check.
   - A rule with only a `select-pattern` and a count is a counter. It counts the units that it matches and takes none, so a production still checks them.
   - A rule with neither a shape nor a `select-pattern` takes every unit.

2. Bound each rule with `min` and `max`. By default, a production requires at least one unit, and `min: 0` allows none. A rule with a `select-pattern` has no default, so you must set its `min`. A counter with `max: 0` bans what it matches.

   A guard selects units independently of other rules: two guards can select one unit, and both check it. A counter counts a unit that a production takes.

   This changelog schema accepts added and fixed entries, at most one breaking entry with a migration command under it, and no `TODO`:

**`changelog.yaml`**

```yaml
line-limit: 80
sections:
  - heading: { template: "${:int}.${:int}.${:int}", example: "2.1.0" }
    level: 2
    repeat: true
    blocks:
      - kind: bullet-list
        rules:
          - template: "Added: ${}."
            min: 0
            example: "Added: a dark theme."
          - template: "Fixed: ${}."
            min: 0
            example: "Fixed: the crash on start."
          - select-pattern: "^Breaking: "
            min: 0
            max: 1
            example: "Breaking: the config file moved."
            blocks:
              - kind: code
                lang: console
                example: "$ app migrate"
          - select-pattern: "TODO"
            min: 0
            max: 0
```

3. Check a document:

**`CHANGELOG.md`**

````markdown
## 2.1.0

- Added: a dark theme.
- Fixed: the crash on start.
- Breaking: the config file moved.

  ```console
  $ app migrate
  ```
````

```console
$ tramlines validate --schema changelog.yaml CHANGELOG.md
0 errors, 0 warnings
```

4. Read the errors for rules. A unit that no rule takes fails against the production that it matches furthest, at the column where it stops matching. A unit that matches no production at all fails with the text of the unit. The error for a counter outside its bounds names its pattern and the count:

**`CHANGELOG.md`**

````markdown
## 2.1.0

- Added: a dark theme.
- Fixed: the slow search
- Added: export to PDF (TODO: docs).
- Removed: the old menu.
````

```console
$ tramlines validate --schema changelog.yaml CHANGELOG.md
CHANGELOG.md:4: error[TLMD_57]: `Fixed: the slow search` matches no rule of the block; the nearest fails at column 23: expected `.`, found the end; an item of this kind reads Fixed: the crash on start. (schema changelog, section "${:int}.${:int}.${:int}" block 1)
CHANGELOG.md:5: error[TLMD_159]: the items matching `TODO`: at most 0 are allowed; there is 1 (schema changelog, section "${:int}.${:int}.${:int}" block 1)
CHANGELOG.md:6: error[TLMD_53]: `Removed: the old menu.` matches no rule of the block; declare it or set `units-open` (schema changelog, section "${:int}.${:int}.${:int}" block 1)
3 errors, 0 warnings
[exit 1]
```

5. To accept units that no rule takes, set `units-open: true` on the block. The rules still check the units that they take, and still apply their counts.

6. To check the lines of a code or `html` block, set `lines` on the `each` of the block. A code block has one unit, its text. The key `lines` is the grammar of that text, as `rules` is the grammar of the units of a block. The same productions, guards, and counts apply to each line. tramlines ignores the indent of each line and skips blank lines. A line rule can't declare `lines` or `blocks`.

   To accept lines that no rule takes, set `lines-open: true` beside `lines`. This schema requires a settings file with a `[server]` line, one to three settings, and no comments:

**`settings.yaml`**

```yaml
line-limit: 60
sections:
  - heading: Settings
    level: 1
    blocks:
      - kind: code
        lang: ini
        each:
          lines:
            - text: "[server]"
            - template: "${name:slug} = ${}"
              min: 1
              max: 3
              example: "port = 8080"
            - select-pattern: "^#"
              min: 0
              max: 0
```

**`settings.md`**

````markdown
# Settings

```ini
[server]
port = 8080
  host = example.org

# TODO
debug: true
```
````

```console
$ tramlines validate --schema settings.yaml settings.md
settings.md:8: error[TLMD_87]: `# TODO` matches no rule of the fence; declare it or set `lines-open` (schema settings, section "Settings" block 1)
settings.md:8: error[TLMD_159]: the lines matching `^#`: at most 0 are allowed; there is 1 (schema settings, section "Settings" block 1)
settings.md:9: error[TLMD_57]: `debug: true` matches no rule of the fence; the nearest fails at column 6: expected ` = `, found `: true`; a line of this kind reads port = 8080 (schema settings, section "Settings" block 1)
3 errors, 0 warnings
[exit 1]
```

The indented `host` line passes, and the comment fails twice: the counter counts it but doesn't take it, and no other rule takes it.

7. Fix the document: remove the comment and write `debug = true`.

**`settings.md`**

````markdown
# Settings

```ini
[server]
port = 8080
  host = example.org

debug = true
```
````

```console
$ tramlines validate --schema settings.yaml settings.md
0 errors, 0 warnings
```

8. Order the productions from narrowest to widest. If an earlier production takes the example of a later one, the later one can take nothing, and the schema fails:

**`tasks.yaml`**

```yaml
line-limit: 60
sections:
  - heading: Tasks
    level: 1
    blocks:
      - kind: bullet-list
        rules:
          - template: "${}."
            min: 0
            example: "Water the plants."
          - template: "Urgent: ${}."
            min: 0
            example: "Urgent: pay the rent."
```

**`tasks.md`**

```markdown
# Tasks

- Water the plants.
```

```console
$ tramlines validate --schema tasks.yaml tasks.md
tasks.yaml:11: error[TLMD_58]: the example `Urgent: pay the rent.` is taken by the rule at line 8 first, so this rule takes nothing: order the rules from the narrowest (schema tasks, section "Tasks" block 1)
1 error, 0 warnings
[exit 1]
```

Move the `Urgent` rule before the other, and the schema passes.

## Troubleshooting

- `TLMD_53`, `TLMD_57`: no rule takes a unit or a line. Correct the unit or line from the column that the message names, add a rule for it, or set `units-open` or `lines-open`. See [TLMD_53][sokf:errors-units#tlmd_53] and [TLMD_57][sokf:errors-units#tlmd_57].
- `TLMD_87`: no rule of `lines` takes a line. See [TLMD_87][sokf:errors-lines-and-pairings#tlmd_87].
- `TLMD_93`: a rule takes or counts fewer units than its `min`. See [TLMD_93][sokf:errors-units#tlmd_93].
- `TLMD_159`: a rule takes or counts more units than its `max`. See [TLMD_159][sokf:errors-units#tlmd_159].
- `TLMD_92`: a rule with a `select-pattern` lacks a `min`. Add `min: 0` to check the units without requiring any. See [TLMD_92][sokf:errors-units#tlmd_92].
- `TLMD_91`: a unit rule declares `select-pattern`, `min`, or `max` outside `rules` and `lines`. See [TLMD_91][sokf:errors-units#tlmd_91].
- `TLMD_142`: a rule declares `text` beside `select-pattern`. A repair of `text` rewrites the whole unit and would drop what the guard selected on. See [TLMD_142][sokf:errors-units#tlmd_142].
- `TLMD_90`: a rule declares `lines` under a block that isn't a code block or an `html` block. See [TLMD_90][sokf:errors-lines-and-pairings#tlmd_90].

## See also

- [Schema keys: `unit`][sokf:reference-schema-keys#unit]
- [Schema keys: `block`][sokf:reference-schema-keys#block]
- [Errors: units][sokf:errors-units]
- [Errors: lines and pairings][sokf:errors-lines-and-pairings]
- [Ban text in a document][sokf:how-to-ban-text]
- [Pitfalls in writing a schema][sokf:explanation-pitfalls]

<!-- sokf:links -->

[sokf:errors-lines-and-pairings]: /docs/reference/errors-lines-and-pairings.md
[sokf:errors-lines-and-pairings#tlmd_87]: /docs/reference/errors-lines-and-pairings.md#tlmd_87
[sokf:errors-lines-and-pairings#tlmd_90]: /docs/reference/errors-lines-and-pairings.md#tlmd_90
[sokf:errors-units]: /docs/reference/errors-units.md
[sokf:errors-units#tlmd_142]: /docs/reference/errors-units.md#tlmd_142
[sokf:errors-units#tlmd_159]: /docs/reference/errors-units.md#tlmd_159
[sokf:errors-units#tlmd_53]: /docs/reference/errors-units.md#tlmd_53
[sokf:errors-units#tlmd_57]: /docs/reference/errors-units.md#tlmd_57
[sokf:errors-units#tlmd_91]: /docs/reference/errors-units.md#tlmd_91
[sokf:errors-units#tlmd_92]: /docs/reference/errors-units.md#tlmd_92
[sokf:errors-units#tlmd_93]: /docs/reference/errors-units.md#tlmd_93
[sokf:explanation-pitfalls]: /docs/explanation/pitfalls.md
[sokf:how-to-ban-text]: /docs/how-to/ban-text.md
[sokf:how-to-describe-blocks]: /docs/how-to/describe-blocks.md
[sokf:how-to-write-line-shapes]: /docs/how-to/write-line-shapes.md
[sokf:reference-schema-keys#block]: /docs/reference/schema-keys.md#block
[sokf:reference-schema-keys#unit]: /docs/reference/schema-keys.md#unit
