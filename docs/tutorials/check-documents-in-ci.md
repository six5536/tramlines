---
type: Tutorial
id: tutorial-check-documents-in-ci
title: Check documents in CI
description: Add a schema to a repository of meeting notes, and a GitHub Actions workflow that checks every note on each push.
order: 20
---

# Check documents in CI

In this tutorial you add a schema for meeting notes to a repository and check the notes with `tramlines validate`. You then add a GitHub Actions workflow that runs the same check on each push and pull request. At the end you know what each exit code of the command means for a continuous integration (CI) job. You also know how to add a rule without breaking the build.

A CI workflow reads the exit code of the command: the workflow fails when the command exits with a code other than 0.

## Before you begin

- tramlines installed, as the [README](https://github.com/six5536/tramlines#installation) shows, to run the check on your own computer.
- A GitHub repository that you can push to. The CI job installs tramlines itself, with `npx`.
- The steps of [Your first schema][sokf:tutorial-first-schema], or some experience reading a schema.

## Step 1: add a schema and a note

Keep the schema in the repository, beside the documents it checks, so that reviewers see a change to the schema and a change to the notes together. This tutorial keeps schemas in `schemas/` and notes in `notes/`.

The meeting-note schema requires a `title` and a `date` in the frontmatter, a heading that equals the title, and three sections. Each item under `Actions` starts with the name of its owner:

**`schemas/meeting.yaml`**

```yaml
line-limit: 200
frontmatter:
  title:
    example: Planning meeting
  date:
    pattern: '^\d{4}-\d{2}-\d{2}$'
    example: '2026-09-14'
sections:
  - heading: { from: title }
    level: 1
    blocks: []
  - heading: Attendees
    level: 2
    blocks:
      - kind: bullet-list
        example: Ana Lima
  - heading: Decisions
    level: 2
    blocks:
      - kind: bullet-list
        example: Ship the beta on 1 October.
  - heading: Actions
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${owner:word}: ${}'
          example: 'Ana: book the room for the launch'
```

Add the first note:

**`notes/2026-09-14.md`**

```markdown
---
title: Planning meeting
date: 2026-09-14
---

# Planning meeting

## Attendees

- Ana Lima
- Ben Okafor

## Decisions

- Ship the beta on 1 October.

## Actions

- Ana: book the room for the launch
- Ben: write the release notes
```

Run the check from the root of the repository, with paths relative to it:

```console
$ tramlines validate --schema schemas/meeting.yaml notes/2026-09-14.md
0 errors, 0 warnings
```

- The command finds no errors, so it exits with code 0, and a CI job that runs it passes.

## Step 2: see a failing check

Add a second note with two mistakes: the date isn't in the form `YYYY-MM-DD`, and the action names no owner:

**`notes/2026-09-21.md`**

```markdown
---
title: Launch review
date: 21 September
---

# Launch review

## Attendees

- Ana Lima

## Decisions

- Move the launch to 8 October.

## Actions

- book a second room
```

Check both notes in one run. The command checks every document that you name, and reports all the errors before it exits:

```console
$ tramlines validate --schema schemas/meeting.yaml notes/2026-09-14.md notes/2026-09-21.md
notes/2026-09-21.md:3: error[TL_56]: `date` `21 September` does not match `^\d{4}-\d{2}-\d{2}$`; a value of this kind reads 2026-09-14 (schema meeting, key `date`)
notes/2026-09-21.md:18: error[TL_74]: `book a second room` fails at column 5: expected `: `, found ` a second room`; an item of this kind reads Ana: book the room for the launch (schema meeting, section "Actions" block 1)
2 errors, 0 warnings
[exit 1]
```

- Exit code 1 means that at least one document breaks the schema. A CI job that runs this command fails, and its log shows these error lines.
- Each error line starts with `path:line`, so you can open the note at the line that's wrong.

Fix the note:

**`notes/2026-09-21.md`**

```markdown
---
title: Launch review
date: 2026-09-21
---

# Launch review

## Attendees

- Ana Lima

## Decisions

- Move the launch to 8 October.

## Actions

- Ana: book a second room
```

```console
$ tramlines validate --schema schemas/meeting.yaml notes/2026-09-14.md notes/2026-09-21.md
0 errors, 0 warnings
```

## Step 3: see a check that can't run

Exit code 2 means that the command didn't check the documents at all: its command line is wrong, or it can't read a file. A CI job fails on code 2 too. To fix the failure, change the workflow rather than the notes.

Name a note that doesn't exist:

```console
$ tramlines validate --schema schemas/meeting.yaml notes/2026-09-28.md
error: notes/2026-09-28.md: No such file or directory (os error 2)
[exit 2]
```

Leave out `--schema`:

```console
$ tramlines validate notes/2026-09-14.md
error: the following required arguments were not provided:
  --schema <FILE>

Usage: tramlines validate --schema <FILE> <DOC>...

For more information, try '--help'.
[exit 2]
```

- The command picks the format of each document by its extension, so a document with a `.txt` extension stops the command with code 2 too. [The command][sokf:reference-cli] lists the formats, and [Errors: the command][sokf:errors-command] lists each error that stops the command.

## Step 4: add the workflow

Add a workflow that installs Node.js and runs the check with `npx`. The `npx` command downloads the `@six5536/tramlines` package and runs its `tramlines` command. The job runs on each push and each pull request:

**`.github/workflows/check-notes.yml`**

```yaml
name: Check notes

on:
  push:
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npx @six5536/tramlines validate --github --warnings --schema schemas/meeting.yaml notes/*.md
```

The job's shell expands `notes/*.md` to every note, so the job checks each added note with no change to the workflow. The `--github` option prints each error and warning as a GitHub workflow command, so that a pull request shows it on the line of the note. The `--warnings` option lists the warnings as well as the errors. To pin the version of tramlines, add it to the package name, as in `@six5536/tramlines@0.1.0`.

To run the same check on your computer, name each note:

```console
$ tramlines validate --schema schemas/meeting.yaml notes/2026-09-14.md notes/2026-09-21.md
0 errors, 0 warnings
```

Commit the schema, the notes, and the workflow, and push. The job passes when the command exits with code 0, and fails on code 1 or 2.

## Step 5: add a rule as a warning first

An added rule can break the notes already in the repository, and so break the build for everyone. Add the rule with `severity: warn` instead. A failure of the rule is then a warning, and warnings don't change the exit code.

Require each decision to end with a full stop, as a warning:

**`schemas/meeting.yaml`**

```yaml
line-limit: 200
frontmatter:
  title:
    example: Planning meeting
  date:
    pattern: '^\d{4}-\d{2}-\d{2}$'
    example: '2026-09-14'
sections:
  - heading: { from: title }
    level: 1
    blocks: []
  - heading: Attendees
    level: 2
    blocks:
      - kind: bullet-list
        example: Ana Lima
  - heading: Decisions
    level: 2
    blocks:
      - kind: bullet-list
        each:
          pattern: '\.$'
          severity: warn
          example: Ship the beta on 1 October.
  - heading: Actions
    level: 2
    blocks:
      - kind: bullet-list
        each:
          template: '${owner:word}: ${}'
          example: 'Ana: book the room for the launch'
```

A note written after the change leaves out the full stop:

**`notes/2026-09-28.md`**

```markdown
---
title: Launch retrospective
date: 2026-09-28
---

# Launch retrospective

## Attendees

- Ben Okafor

## Decisions

- Keep the weekly release call

## Actions

- Ben: send the survey to the beta users
```

```console
$ tramlines validate --warnings --schema schemas/meeting.yaml notes/2026-09-14.md notes/2026-09-21.md notes/2026-09-28.md
notes/2026-09-28.md:14: warning[TL_74]: `Keep the weekly release call` does not match `\.$`; an item of this kind reads Ship the beta on 1 October. (schema meeting, section "Decisions" block 1)
0 errors, 1 warning
```

- With `--warnings`, the command lists the warning and exits with code 0, so the CI job passes. The workflow runs with `--warnings` too, so the pull request shows the decision to fix. Without `--warnings`, the command only counts the warning.
- When every note follows the rule, remove `severity: warn`. From then on, a decision with no full stop is an error, and the job fails on it.

## What's next

- [Read an error and fix the document][sokf:how-to-read-a-finding] shows how to read each part of an error line.
- [Share line formats across schemas][sokf:how-to-share-line-formats] shows how to keep the line formats of several schemas in one file that you pass with `--formats`.
- [The command][sokf:reference-cli] lists every option, the output, and the exit codes.
- [Errors][sokf:reference-errors] explains each key that an error line can name.

<!-- sokf:links -->

[sokf:errors-command]: /docs/reference/errors-command.md
[sokf:how-to-read-a-finding]: /docs/how-to/read-a-finding.md
[sokf:how-to-share-line-formats]: /docs/how-to/share-line-formats.md
[sokf:reference-cli]: /docs/reference/cli.md
[sokf:reference-errors]: /docs/reference/errors.md
[sokf:tutorial-first-schema]: /docs/tutorials/first-schema.md
