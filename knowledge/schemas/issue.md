---
type: Schema
id: schema-issue
title: Issue schema
description: "An issue is a feature, a bug, a chore, or an epic, containing the work description, the plan for doing it, and (when completed) its outcome."
---

<!-- prettier-ignore -->
````yaml
line-limit: 0
labels: [human, agent]

guidance: >
  An issue records one piece of work: a feature, a bug, or a chore, or it is an epic
  that groups other issues that meet one need. Its readers are the developer, the
  agent doing the work, and anyone who later asks why the work was done. An issue can be
  filed at any time, unplanned, and is planned with the developer before any work, as the
  Plan section says. After the work is done, the review and the outcome follow the plan. An
  issue is history, not the authority. The stories, the contracts, the designs, and the code
  are the authority, and none of them cites an issue. A closed issue that predates a section
  that the schema now requires holds `Not recorded:` and the reasons in that section, as an
  item where the section is a list. Each part that states intent, such as the need, a
  criterion, a question, or a decision, carries a label that says whether the developer
  said it or an agent proposed it.

root: /knowledge/issues

# An issue is history, so no code or test cites one.
markers: []

# While an issue is open, each new file that its Files section links is planned,
# so any document, such as a design or a contract, can link and include the file
# before the code exists.
plans-paths:
  - section: files
    only: { state: [open] }

lifecycle:
  - key: state
    states: [open, closed]
    path: true
    archived: [closed]
    guidance: >
      An issue is open while its work remains to be done. It closes after the work is done
      or dropped, and a closed issue doesn't change. The issue's folder has the name of its
      state, and `sokf graph` shows a closed issue only with `--archived`.
  - key: kind
    states: [feature, bug, chore, epic]
    guidance: >
      The kind is _feature_ for something that a user can't do yet, _bug_ for something
      that doesn't work as a contract says, and _chore_ for work that changes no behaviour
      that a user relies on, such as a change of configuration, CI, or a document. The kind
      is _epic_ for a need that several issues meet together. An epic plans no code of its
      own: its issues hold the criteria, the files, and the steps, and the epic lists them in
      the order of the work. The id contains the kind, so the kind never changes. An issue of
      the wrong kind closes, and a new issue of the right kind replaces it.
  - key: plan
    states: [unplanned, planned]
    guidance: >
      An issue is _unplanned_ until its plan is written and approved, and _planned_ after, as
      the Plan section says. Only a planned issue is worked on. A closed issue that was
      worked is planned, and one dropped before planning is unplanned.

frontmatter:
  type:
    const: Issue
  id:
    template: "issue-${:seq}-${kind}-${:slug}"
  title:
    template: '${:action} ${}'
    guidance: >
      The title describes the work in a few words. It begins with a verb from the type
      `action` of the issue schema, followed by what the verb acts on. When no verb fits the
      work, add one to the type `action`, and send the change to the pack with
      `pakf patch`.
    example: Report the pattern that stops a match
  priority:
    enum: [urgent, high, medium, low, unset]
    only: { state: [open] }
    guidance: >
      The priority says how soon the work should be done, against every other open issue.
      _Urgent_ is for work that must be attended to now, such as a fault that blocks all
      other work in the project, and it stays rare, because an urgent issue interrupts the rest. _High_ is for
      work on the path of what comes next, _medium_ for the work after it, and _low_ for work
      that can wait. _Unset_ says that nobody has set the priority yet, such as for an issue
      filed in the moment, and planning is a good time to set it. An epic's priority orders it
      among the epics, and its issues' priorities order the work within it, beside the epic's
      list. A closed issue can keep the priority that it had.
    example: medium
  description:
    example: 'When a message with the attribute `explain=yes` matches no rule set, the error names the pattern that failed.'

links:
  - rel: depends-on
    to-type: Issue
    min: 0
    guidance: A depends-on link points to an issue that has to close before this one can.
  - rel: supersedes
    to-type: Issue
    min: 0
    guidance: A supersedes link points to an issue that this one replaces.
  - rel: relates-to
    to-type: Issue
    min: 0
  - rel: part-of
    to-type: Issue
    min: 0
    max: { hard: 1, when-at-limit: "Link each further epic with `relates-to`." }
    guidance: A part-of link points to the epic whose need this issue helps to meet. An epic is part of no other issue.

types:
  decision-id: 'D-[1-9][0-9]*'
  question-id: 'Q-[1-9][0-9]*'
  action: [Add, Allow, Change, Fix, Make, Move, Remove, Rename, Replace, Report, Split, Tighten]

formats:
  criterion-link:
    pattern: '^\[[ x]\] \[[A-Z]+(?:_[A-Z]+)*(?:-[0-9]+)?(?:_AC-[0-9]+|_P-[0-9]+|_[0-9]+)?\]\[sokf:[^\]]+\](?:(?:, and |, | and | to )\[[A-Z]+(?:_[A-Z]+)*(?:-[0-9]+)?(?:_AC-[0-9]+|_P-[0-9]+|_[0-9]+)?\]\[sokf:[^\]]+\])*$'
    example: '[x] [PLUMB-1_AC-5][sokf:contract-plumbing#plumb-1_ac-5]'
  criterion-link-or-permalink:
    pattern: '^\[[ x]\] \[[A-Z]+(?:_[A-Z]+)*(?:-[0-9]+)?(?:_AC-[0-9]+|_P-[0-9]+|_[0-9]+)?\](?:\[sokf:[^\]]+\]|\(https://[^/)\s]+(?:/[^/)\s]+)*/[0-9a-f]{40}/[^)\s]+\))(?:(?:, and |, | and | to )\[[A-Z]+(?:_[A-Z]+)*(?:-[0-9]+)?(?:_AC-[0-9]+|_P-[0-9]+|_[0-9]+)?\](?:\[sokf:[^\]]+\]|\(https://[^/)\s]+(?:/[^/)\s]+)*/[0-9a-f]{40}/[^)\s]+\)))*$'
    example: '[x] [PLUMB-1_AC-5][sokf:contract-plumbing#plumb-1_ac-5]'
  ears-criterion:
    pattern: '^\[[ x]\] (?:(?:WHEN|WHILE|WHERE) [^,]+, |IF .+, THEN )?[Tt]he .+ SHALL .+\.$'
    example: '[ ] WHEN a message matches no rule set, the file server SHALL fail the write to `send` with an error that begins with `no matching plumb rule`.'
  finding-fixed:
    template: '${} (fixed)'
    example: 'The matcher read each rule set twice. (fixed)'
  finding-left:
    template: '${} (left: ${})'
    example: 'The loop in `matchruleset` could return early. (left: the function is 12 lines long)'
  finding-moved:
    template: '${} (moved: [${}][sokf:${:ref}])'
    example: 'The file server copies the error for each message. (moved: [issue-013][sokf:issue-013-chore-error-copies])'
  review-rounds:
    template: 'Rounds: ${:int}'
    example: 'Rounds: 2'

sections:
  - heading: { from: title }
    level: 1
    guidance: >
      This section says who needs the work and why, and ends with a contents list. The list
      has one item for each level-2 section, in order, and each item links to its section's
      heading. It is optional in a closed issue, which keeps the form it had when it closed.
    blocks:
      - kind: prose
        guidance: >
          The prose says who needs what and why, adding to the description rather than
          repeating it. Each reason for the need has a sentence of its own. Where the opening
          names only the kind of user, the sections that follow give the examples of who
          needs the work, such as named groups or projects. Those sections, such as Problem,
          Impact, or Motivation, hold the detail of the need, and the Plan's decisions hold
          each constraint.
        label: required
        example: '[agent] A user who writes plumbing rules needs to know which pattern stopped a message from matching, so that the user can correct the rule set rather than guess.'
      - kind: bullet-list
        only: { state: [open], kind: [feature] }
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Problem](#problem)'
            - '[Proposed solution](#proposed-solution)'
            - '[Alternatives considered](#alternatives-considered)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
      - kind: bullet-list
        only: { state: [open], kind: [bug] }
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Steps to reproduce](#steps-to-reproduce)'
            - '[Expected behaviour](#expected-behaviour)'
            - '[Actual behaviour](#actual-behaviour)'
            - '[Impact](#impact)'
            - '[Family](#family)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
      - kind: bullet-list
        only: { state: [open], kind: [chore] }
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Motivation](#motivation)'
            - '[Drawbacks](#drawbacks)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
      - kind: bullet-list
        only: { state: [open], kind: [epic] }
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Problem](#problem)'
            - '[Proposed solution](#proposed-solution)'
            - '[Issues](#issues)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
      - kind: bullet-list
        only: { state: [closed], kind: [feature] }
        optional: true
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Problem](#problem)'
            - '[Proposed solution](#proposed-solution)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
            - '[Review](#review)'
            - '[Outcome](#outcome)'
      - kind: bullet-list
        only: { state: [closed], kind: [bug] }
        optional: true
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Steps to reproduce](#steps-to-reproduce)'
            - '[Expected behaviour](#expected-behaviour)'
            - '[Actual behaviour](#actual-behaviour)'
            - '[Impact](#impact)'
            - '[Family](#family)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
            - '[Review](#review)'
            - '[Outcome](#outcome)'
      - kind: bullet-list
        only: { state: [closed], kind: [chore] }
        optional: true
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Motivation](#motivation)'
            - '[Drawbacks](#drawbacks)'
            - '[Acceptance criteria](#acceptance-criteria)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
            - '[Review](#review)'
            - '[Outcome](#outcome)'
      - kind: bullet-list
        only: { state: [closed], kind: [epic] }
        optional: true
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Problem](#problem)'
            - '[Proposed solution](#proposed-solution)'
            - '[Issues](#issues)'
            - '[Out of scope](#out-of-scope)'
            - '[Plan](#plan)'
            - '[Outcome](#outcome)'

  # The issue: a feature or an epic.
  - heading: Problem
    level: 2
    only: { kind: [feature, epic] }
    guidance: >
      This section says what the user can't do yet, why that matters, and what the code does
      now, with a link to the code or to the section of the design concerned. Each reason,
      and each cost of the gap, such as the time that it takes from people, has a sentence
      of its own. After the issue is planned, and in a closed issue, the section links the story that the
      issue serves by its key, such as [ST-4][sokf:stories-plumbing#st-4]. Where no story covers the need,
      the story is written first, as a new story with a `Planned:` paragraph. An open issue
      links no deprecated story, because the system no longer meets a deprecated story's
      need.
    blocks:
      - kind: prose
        only: { state: [open], plan: [unplanned] }
        label: required
        example: '[agent] A user who writes a rule set can''t tell why a message doesn''t match it. When a message matches no rule set, the write to `send` fails with the same error, `no matching plumb rule`, whichever pattern failed. The code of [`matchruleset`][sokf:design-plumber#plmb-1] and of the file server causes this:'
      - kind: prose
        only: [{ plan: [planned] }, { state: [closed] }]
        label: required
        rules:
          - pattern: '\[ST-[1-9][0-9]*\]\[sokf:[^\]]*#st-[1-9][0-9]*\]'
            min: 1
            example: '[agent] A user who writes a rule set can''t tell why a message doesn''t match it, which [ST-4][sokf:stories-plumbing#st-4] needs. When a message matches no rule set, the write to `send` fails with the same error, `no matching plumb rule`, whichever pattern failed. The code of [`matchruleset`][sokf:design-plumber#plmb-1] and of the file server causes this:'
          - pattern: '.'
            min: 0
            example: '[agent] The file server reads the result of `matchruleset` only for its port.'
      - kind: bullet-list
        optional: true
        guidance: Each item states one finding, where the section has more than one.
        label: required
        example:
          - '[agent] `matchruleset` stops at the first pattern that fails, returns nil, and discards the pattern.'
          - '[agent] The file server turns every nil result into the same error.'
  - heading: Proposed solution
    level: 2
    optional: { plan: [unplanned] }
    only: { kind: [feature, epic] }
    guidance: This section describes the change as the user sees it. The plan describes how the change is built.
    blocks:
      - kind: prose
        label: required
        example: '[agent] When a message with the attribute `explain=yes` matches no rule set, the `plumb` command prints an error. The error names the failed pattern of the last rule set, such as `arg isfile $0`.'

  # The issue: a bug.
  - heading: Steps to reproduce
    level: 2
    only: { kind: [bug] }
    guidance: >
      This section lists the fewest steps that show the bug, with one action in each step.
      The setting that every step shares, such as the programs that run during the steps,
      is a sentence before the list, not part of the first step.
    blocks:
      - kind: prose
        optional: true
        example: 'Every step runs on a terminal where the plumber and the editor `acme` are running.'
      - kind: numbered-list
        example:
          - 'Use a rule set that matches a file name with the pattern `data matches ''[a-zA-Z0-9_\-./]+''` and sends it to the editor with `plumb to edit`.'
          - 'Run `plumb -a click=5 mem.c`, which sends the text `mem.c` with the click position at its end.'
  - heading: Expected behaviour
    level: 2
    only: { kind: [bug] }
    guidance: >
      This section says what should happen and links the contract criterion that says so.
      When no criterion says so, the contract has a gap: the contract gains a new criterion,
      and this section links it.
    blocks:
      - kind: prose
        label: required
        example: '[agent] The editor receives `mem.c`, because the click position lies at the end of the matched text, as [PLUMB-1_AC-4][sokf:contract-plumbing#plumb-1_ac-4] says.'
  - heading: Actual behaviour
    level: 2
    only: { kind: [bug] }
    guidance: >
      This section says what happens instead, with the output where there is any. Each
      observation of the steps has a sentence of its own. Impact says how long the fault
      lasts and how often it strikes, and the Approach section of the plan states the root
      cause.
    blocks:
      - kind: prose
        example: 'The write to `send` fails with `no matching plumb rule`. The editor receives nothing.'
      - kind: code
        optional: true
        lang: any
        example: |
          % plumb -a click=5 mem.c
          plumb: no matching plumb rule
  - heading: Environment
    level: 2
    optional: true
    only: { kind: [bug] }
    guidance: This section states the version and the platform, where they affect the bug.
    blocks:
      - kind: bullet-list
        example:
          - 'Plan 9 from Bell Labs, fourth edition'
          - 'A 386 terminal'
  - heading: Impact
    level: 2
    optional: { plan: [unplanned] }
    only: { kind: [bug] }
    guidance: >
      This section says what happens when the bug strikes, how badly, and how likely it is,
      which sets the priority. It also says how the bug was found, by whom and when, with
      the numbers, such as how many users met it. A bug found by a review counts as fully
      as one found in use, because no one might have met it yet. Each consequence has a
      sentence or an item of its own, and no sentence lists several consequences.
    blocks:
      - kind: prose
        example: 'Every click after the last character of the data meets the bug. The user then has to click inside the name instead. A review of `match.c` found the bug before any user reported it.'
      - kind: bullet-list
        optional: true
        guidance: Each item states one consequence of the bug.
        example:
          - 'A click after `mem.c:27` in a compiler''s error message opens nothing in the editor.'
          - 'A click after the name of an image file in a mail message shows no image.'
  - heading: Family
    level: 2
    optional: { plan: [unplanned] }
    only: { kind: [bug] }
    guidance: >
      This section says whether the bug's cause can recur wherever the same pattern exists,
      with the other bugs and the places in the code that share it, or that the cause is
      local. The family is weighed for every bug, because fixing one bug of a family at a
      time leaves the next one waiting. A family is judged by its cause, not by a number of bugs, so a cause such
      as a line ending of Windows is a family from its first instance.
    blocks:
      - kind: prose
        example: 'The cause can recur wherever the plumber looks for text past the end of the data, because each such place needs a case of its own for the end:'
      - kind: bullet-list
        optional: true
        guidance: Each item names one place or bug that shares the cause, with what happens there.
        example:
          - 'The rules reader parses the last rule set of a write only when the text ends with a blank line, so the write reports no error in that rule set.'

  # The issue: a chore.
  - heading: Motivation
    level: 2
    only: { kind: [chore] }
    guidance: >
      This section says why the work is worth doing now, though it changes no behaviour that
      a user relies on, and the current state of what the work changes, such as the code, a
      configuration file, or a workflow. It names what the work saves, and each saving has a
      sentence of its own. A chore must save more than it costs, and Drawbacks names the
      costs. The section links the story or the issue that the work serves.
    blocks:
      - kind: prose
        label: required
        example: '[agent] A new verb needs an entry in the list of verbs in `plumber.h`, a name in the parser in `rules.c`, and a case in `match.c`. Nothing checks that the three agree, so a verb that one of them lacks fails only when a message uses it.'
      - kind: bullet-list
        optional: true
        guidance: Each item states one finding, when there is more than one.
        label: required
        example:
          - '[agent] The parser in `rules.c` turns the name of a verb into its number.'
          - '[agent] The matcher in `match.c` handles each verb in a case of its own.'

  # Every kind of issue; a chore must have it.
  - heading: Drawbacks
    level: 2
    optional: { kind: [feature, bug, epic] }
    guidance: >
      This section lists what the work costs: its effort, the lasting complexity that it adds,
      and its risks, including each consequence of a decision that makes something worse. A
      chore must save more than it costs, so a chore has the section. A cost that a decision
      causes cites the decision by its key.
    blocks:
      - kind: bullet-list
        only: { state: [open] }
        guidance: >
          Each item states one cost, and its reasons, each reason a sentence of its own. Its
          label says whether the developer named the cost or an agent found it.
        label: required
        example:
          - '[agent] A check of the verbs costs one table and a test of 40 lines.'
          - '[agent] A verb added later needs a row in the table as well, so a developer who forgets the row gets a failing test, not a wrong match.'
      - kind: bullet-list
        only: { state: [closed] }
        guidance: >
          In a closed issue, the costs are as in an open one. An issue that closed before its
          costs were recorded in a section of their own holds the one item `Not recorded:` and
          the reasons instead. Each reason is one sentence, and the item can give several.
        label: required
        rules:
          - pattern: '^Not recorded: .+\.$'
            rule-id: not-recorded
            min: 0
            max: { hard: 1, when-at-limit: "Give each further reason as a sentence of its own in the one `Not recorded:` item. Never join several reasons into one sentence to stay within the count." }
            example: '[agent] Not recorded: the issue closed before its costs were recorded in a section of their own.'
          - pattern: '.'
            min: 0
            example:
              - '[agent] A check of the verbs costs one table and a test of 40 lines.'
              - '[agent] A verb added later needs a row in the table as well, so a developer who forgets the row gets a failing test, not a wrong match.'

  # Every kind of issue.
  - heading: Alternatives considered
    level: 2
    optional: true
    guidance: >
      This section lists each other way to do the work that was really weighed, with the
      reasons that the issue doesn't choose it. Each reason is one sentence, and an item can
      give several. An option written only to be rejected tells a later
      reader nothing, so the section holds none, and an issue without real alternatives has
      no such section.
    blocks:
      - kind: bullet-list
        example:
          - 'A log of every match in a file: the user has to find the log and read it after each failure.'
          - 'A command that tests one message against one rule set: it reports nothing about the other rule sets.'

  # An epic.
  - heading: Issues
    level: 2
    optional: { plan: [unplanned] }
    only: { kind: [epic] }
    guidance: >
      This section lists the issues of the epic in the order of the work, and says why the
      order matters where the reason isn't obvious. Each issue links the epic with part-of,
      and cites the epic's decisions where they settle its own. An issue that closes keeps its
      place in the list, because the list is the history of the epic's order. An epic whose
      work falls into phases, such as the releases of a tool, lists its issues in a subsection
      for each phase instead of one list.
    blocks:
      - kind: numbered-list
        optional: true
        label: required
        each:
          template: '[${}][sokf:${:ref}]: ${}.'
          to-type: Issue
          example:
            - '[agent] [Report the failed pattern][sokf:issue-041-feature-report-failed-pattern]: the matcher records the pattern that stops a match, and `plumb` prints it.'
            - '[agent] [Report the failed pattern in acme][sokf:issue-042-feature-acme-failed-pattern]: the editor shows the error in its window.'
      - kind: prose
        optional: true
        guidance: >
          The prose says why the order matters, where the reason isn't obvious. Each reason
          has a sentence of its own, and the reasons for a phase are in the subsection of that
          phase.
        label: required
        example: '[agent] The editor shows only what the file server reports, so the report in `plumb` comes first.'
  - heading: { template: '${}', example: The first release of the plumber }
    level: 3
    optional: true
    repeat: true
    only: { kind: [epic] }
    guidance: >
      Each subsection names one phase of the work and lists its issues in order, and the
      phases come in the order of the work. A paragraph after the list says why the phase
      comes where it does, where the reason isn't obvious.
    blocks:
      - kind: numbered-list
        label: required
        each:
          template: '[${}][sokf:${:ref}]: ${}.'
          to-type: Issue
          example:
            - '[agent] [Report the failed pattern][sokf:issue-041-feature-report-failed-pattern]: the matcher records the pattern that stops a match, and `plumb` prints it.'
      - kind: prose
        optional: true
        guidance: >
          The prose says why the phase comes where it does, where the reason isn't obvious.
          Each reason has a sentence of its own.
        label: required
        example: '[agent] The editor shows only what the file server reports, so the report in `plumb` comes first.'

  - heading: Acceptance criteria
    level: 2
    optional: { plan: [unplanned] }
    only: { kind: [feature, bug, chore] }
    guidance: >
      This section is a checklist of what is true when the work is done, with one item for
      each criterion. A check that applies to every issue, such as a clean validation or a
      passing test suite, isn't a criterion, and the list leaves it out.
    blocks:
      - kind: bullet-list
        only: { state: [open] }
        guidance: >
          A criterion of behaviour that code implements, a quality, or a property is in the
          contract, built or, until its code exists, planned, and its item is its key as a link,
          without its wording. Any other item states its criterion in EARS (Easy
          Approach to Requirements Syntax). Before the issue closes, a criterion that states a
          lasting rule moves into the document that holds the rule, and its item keeps its EARS
          sentence and adds a link to that document. A criterion that only describes the result
          of the work stays as it is. An item is ticked when it's true.
        label: required
        rules:
          - format: criterion-link
            min: 0
            example:
              - '[x] [agent] [PLUMB-1_AC-5][sokf:contract-plumbing#plumb-1_ac-5]'
              - '[ ] [agent] [PLUMB-1_AC-6][sokf:contract-plumbing#plumb-1_ac-6] and [PLUMB-1_AC-7][sokf:contract-plumbing#plumb-1_ac-7]'
          - format: ears-criterion
            min: 0
            example:
              - '[ ] [agent] WHEN a message with the attribute `explain=yes` matches no rule set, the `plumb` command SHALL print the failed pattern of the last rule set.'
              - '[ ] [agent] WHEN a message matches no rule set, the file server SHALL fail the write to `send` with an error that begins with `no matching plumb rule`.'
              - '[ ] [agent] The file server SHALL keep each message that no rule set matches out of every port.'
      - kind: bullet-list
        only: { state: [closed] }
        guidance: >
          In a closed issue, an item that isn't ticked ends with `(not done:` and the reason.
          A criterion can leave the contract after the issue closes. The item then links the
          criterion by a permalink to the last commit that holds it.
        label: required
        rules:
          - select-pattern: '^\[ \]'
            pattern: '\(not done: .+\)$'
            min: 0
            example: '[ ] [agent] [PLUMB-1_AC-6][sokf:contract-plumbing#plumb-1_ac-6] (not done: an error string of 9P, the Plan 9 file protocol, holds at most 128 bytes)'
          - format: criterion-link-or-permalink
            min: 0
            example: '[x] [agent] [PLUMB-1_AC-5][sokf:contract-plumbing#plumb-1_ac-5]'
          - format: ears-criterion
            min: 0
            example: '[x] [agent] WHEN a message matches no rule set, the file server SHALL fail the write to `send` with an error that begins with `no matching plumb rule`.'

  - heading: Out of scope
    level: 2
    optional: { plan: [unplanned] }
    guidance: >
      This section lists what the issue leaves out, one thing in each item. An item links
      the issue that covers it, where there is one.
    blocks:
      - kind: bullet-list
        label: required
        example:
          - '[agent] A report on a message that matches the wrong rule set, because no pattern fails in that case.'
          - '[agent] A change to the editor `acme`, because the error string reaches it unchanged through `libplumb`.'

  # The plan.
  - heading: Plan
    level: 2
    optional: { plan: [unplanned] }
    guidance: >
      This section says how the work is done. An unplanned issue's plan holds only its open
      questions and the decisions taken so far, and everything that the issue holds is a lead
      to check, not a fact, because it was written when the issue was filed. The issue is
      planned with the developer before any work starts. The plan rests on a fresh reading of
      the architecture, the contract and the design of each part that the work changes, the
      files that it touches, with `sokf graph` on each, the existing code that already does
      part of the job, the decisions of the closed issues that touched the same files, and
      the project rules and coding standards, because a plan built on what the writer assumes
      of the code, rather than on what the code holds, fails in the work. The planning weighs
      whether the issue's premise still holds in the big picture, settles each open question
      with the developer, and writes the plan and its certification. The issue's `plan`
      becomes `planned` when the developer approves the plan, or when the agent plans the issue on its own, as the Certification section says. The depth of the plan matches
      the stakes: a fix with a local cause takes a few lines, and a new capability takes a
      full discussion. A planned issue's plan is complete: every question is settled as a
      decision, and only a planned issue is worked on. When the work starts long after the
      plan was written, the work begins with a check of the plan against the code.
    blocks:
      - kind: prose
        optional: [{ plan: [unplanned] }, { state: [closed] }]
        guidance: >
          The prose states the plan in outline. The Approach subsection holds the detail,
          and in a planned issue the Steps subsection holds the order of the work.
        label: required
        example: '[agent] The plan changes the matcher and the file server and adds the test `nomatch.rc`. The matcher records the failed pattern, and the file server puts it in the error of a message with the attribute `explain=yes`.'
  - heading: Open questions
    level: 3
    optional: true
    only: { state: [open], plan: [unplanned] }
    guidance: >
      This section lists the questions that the plan has to settle, each with a key (Q-1,
      Q-2, and so on) and the recommended answer. The recommendation gives its reason
      after _because_, and each further reason is a sentence of its own. The section lists
      only the questions that are hard to reverse, reach widely, or set the direction, and
      leaves the rest to the agent's judgement. Where the developer has said which question
      they most want to discuss, that question comes first, and a sentence of its own in its
      item says so. The other questions follow, the biggest first. The agent puts each
      question to the developer in turn, in the conversation, with its context and its
      recommendation. It writes each answer into the Approach as a decision, and removes
      the question, before it asks the next. A planned issue has no open questions.
    blocks:
      - kind: bullet-list
        each:
          template: '${id:question-id} ${}? Recommended: ${}.'
          key: id
          label: required
          example:
            - 'Q-1 [agent] Does the error name a pattern from every rule set, or only from the last one? Recommended: only the last one, because an error string of 9P, the Plan 9 file protocol, holds at most 128 bytes. The developer most wants to discuss this question.'
            - 'Q-2 [agent] Does a message ask for the report with an attribute, or does every error carry it? Recommended: an attribute, `explain=yes`, because a program that reads the error expects `no matching plumb rule` alone.'
  - heading: Approach
    level: 3
    optional: { plan: [unplanned] }
    guidance: >
      This section describes the chosen approach, then lists its decisions. A rejected option
      belongs in Alternatives considered.
    blocks:
      - kind: prose
        optional: true
        guidance: >
          The prose states the root cause of a bug, where it's known, then the approach. It
          links the design that records the decisions.
        label: required
        example: '[agent] The file server appends the failed pattern and the port of its rule set to the error, as in `no matching plumb rule: arg isfile $0 failed in the rule set for edit`. The [plumber''s design][sokf:design-plumber] records the decisions:'
      - kind: bullet-list
        guidance: >
          Each decision has a key (D-1, D-2, and so on), because a later issue cites a decision
          by its key. Its label says who took it: `[human]` for the developer's answer, and
          `[agent]` for a decision that the agent took on its own, where the developer
          delegated it or it is easy to reverse. A decision states one choice. Each constraint
          that the developer stated is a decision of its own, and a decision whose reason is
          such a constraint cites that decision by its key. Each decision states its reason
          after _because_, and each further reason is a sentence of its own. The reason
          lets a later writer judge whether the decision still holds, and a decision without
          one can't be revisited safely. A decision that refines an earlier one is a new
          decision that cites the earlier one. A decision converted from a plan that gave no
          reason ends with "because the original plan records no reason". A reason that the
          agent inferred or proposed for the developer's choice is an `[agent]` decision of its
          own that cites the choice by its key, because an issue keeps the context of a
          decision as it was, and a reader has to tell a recorded reason from a reconstructed
          one. A decision's consequences, and the findings that support it, are a list nested
          under it.
        only: { state: [open] }
        each:
          template: '${id:decision-id} ${}, because ${}.'
          key: id
          label: required
          blocks:
            - kind: bullet-list
              optional: true
              guidance: >
                The decision's consequences: each item states one effect of the choice, or one
                finding that supports it, in a sentence of its own. A consequence that makes
                something worse is a cost, and Drawbacks states it instead, citing the
                decision.
              label: required
              example:
                - '[agent] Every caller of `matchruleset` other than the file server passes nil and is unchanged.'
                - '[agent] The error of a message that matches no rule set names the last rule set''s port, which the user can open.'
          example:
            - 'D-1 [human] `matchruleset` returns the number of the failed pattern through a new argument, because a nil result then means the same as before to every other caller.'
            - 'D-2 [human] The error fits in one error string of 9P, because the plumber reports every error as the error string of a 9P request.'
            - 'D-3 [agent] The error names the failed pattern of the last rule set only, because D-2 limits the error to the 128 bytes of a 9P error string.'
      - kind: bullet-list
        only: { state: [closed] }
        guidance: >
          In a closed issue, the decisions are as in an open one. A closed issue whose plan
          recorded no decision holds the one item `Not recorded:` and the reasons instead.
          Each reason is one sentence, and the item can give several.
        label: required
        rules:
          - template: '${id:decision-id} ${}, because ${}.'
            key: id
            min: 0
            blocks:
              - kind: bullet-list
                optional: true
                guidance: >
                  The decision's consequences: each item states one effect of the choice, or one
                  finding that supports it, in a sentence of its own. A consequence that makes
                  something worse is a cost, and Drawbacks states it instead, citing the
                  decision.
                label: required
                example:
                  - '[agent] Every caller of `matchruleset` other than the file server passes nil and is unchanged.'
                  - '[agent] The error of a message that matches no rule set names the last rule set''s port, which the user can open.'
            example: 'D-1 [human] `matchruleset` returns the number of the failed pattern through a new argument, because a nil result then means the same as before to every other caller.'
          - pattern: '^Not recorded: .+\.$'
            rule-id: not-recorded
            min: 0
            max: { hard: 1, when-at-limit: "Give each further reason as a sentence of its own in the one `Not recorded:` item. Never join several reasons into one sentence to stay within the count." }
            example: '[agent] Not recorded: the feature closed before its decisions were recorded.'
  - heading: Files
    level: 3
    only: { kind: [feature, bug, chore], plan: [planned] }
    rule-id: files
    guidance: >
      This section lists the files that the work changes or creates, and the existing code to
      reuse, one in each item. Each item gives the path as a link, a colon, and what changes.
      While the issue is open, a link to a file that doesn't exist yet plans that file, so any
      document, such as a design or a contract, can link and include it before the code
      exists. A removed file is in code font.
    blocks:
      - kind: bullet-list
        example:
          - '[`/sys/src/cmd/plumb/match.c`](/sys/src/cmd/plumb/match.c): `matchruleset` records the number of the pattern that failed.'
          - '[`/sys/src/cmd/plumb/fsys.c`](/sys/src/cmd/plumb/fsys.c): the file server adds the failed patterns to the error of a message with the attribute `explain=yes`.'
          - '[`/sys/src/cmd/plumb/test/nomatch.rc`](/sys/src/cmd/plumb/test/nomatch.rc): a test that sends a message with no matching rule set and reads the error.'
  - heading: Steps
    level: 3
    only: { kind: [feature, bug, chore], plan: [planned] }
    guidance: >
      This section lists the steps in the order of the work, with one action in each step. It
      says why the order matters where the reason isn't obvious. When the work starts long
      after the plan was written, its first step checks the plan against the code again. The
      first step of a bug's fix writes a test that fails as the bug does.
    blocks:
      - kind: numbered-list
        example:
          - 'Add the new argument to `matchruleset`, and pass nil from every caller, so that the plumber still builds and behaves as before.'
          - 'Record the failed pattern in `matchruleset`.'
          - 'In the file server, add the failed patterns to the error of a message with the attribute `explain=yes`.'
  - heading: Verification
    level: 3
    only: { kind: [feature, bug, chore], plan: [planned] }
    guidance: >
      This section lists the checks that show the work is done, from end to end. Each check
      states an action and the result that shows the work is done. It also names the tests
      that cover the check. The section leaves out a check that applies to every issue, such
      as a clean validation or a passing test suite.
    blocks:
      - kind: bullet-list
        example:
          - 'Plumb the name of a file that doesn''t exist, such as `nosuch.c`, with the attribute `explain=yes`, and check that the error names the pattern `arg isfile $0`. The test `nomatch.rc` checks the same thing.'
  - heading: Certification
    level: 3
    only: { plan: [planned] }
    guidance: >
      This section certifies the research that the plan rests on, and records who approved
      the plan, as the Plan section's guidance describes. The reasons why the issue is
      relevant answer the core
      question of its kind: for a feature, what it really is; for an epic, what it is, and
      how it splits into issues; for a bug, whether it is worth fixing, and how; for a chore,
      whether it saves more than it costs. The label of the last line, `Planned:`, says who
      approved the plan: `[human]` when the developer approved it in the conversation, and
      `[agent]`, with the reasons, when the developer told the agent to handle the issue
      automatically, or the issue is clear and trivial. Each reason is one sentence, and the
      line can give several.
    blocks:
      - kind: prose
        each:
          text: 'I have read the following code and documents:'
      - kind: bullet-list
        guidance: Each item links a file or a document that was read, a colon, and what it showed.
        each:
          pattern: '\]\(|\]\[sokf:'
          example:
            - '[`/sys/src/cmd/plumb/match.c`](/sys/src/cmd/plumb/match.c): `matchruleset` returns nil at the first pattern that fails, and no caller sees which.'
            - '[The plumber''s design][sokf:design-plumber#plmb-1]: a rule set''s patterns are tried in order, and the first failure ends the match.'
      - kind: prose
        each:
          text: 'I certify that the issue is relevant, because:'
      - kind: bullet-list
        guidance: Each item gives one reason, with a link where it rests on one.
        example:
          - 'Every failed match reports the same error, so a user who writes a rule set can only guess which pattern failed, as [ST-4][sokf:stories-plumbing#st-4] says.'
      - kind: prose
        each:
          text: 'I certify that the solution follows widely adopted conventions and patterns rather than inventing new ones, because:'
      - kind: bullet-list
        guidance: Each item links where a convention or a pattern that the solution follows is widely adopted, a colon, and how the solution follows it.
        each:
          pattern: '\]\(|\]\[sokf:'
          example:
            - '[`errstr(2)`](/sys/man/2/errstr): the error adds the failed pattern after a colon, which is how a Plan 9 program adds a detail to an error string.'
      - kind: prose
        label: required
        each:
          template: 'Planned: ${}.'
          example: '[human] Planned: the developer approved the plan.'

  # The reviews.
  - heading: Review
    level: 2
    only: { state: [closed], kind: [feature, bug, chore] }
    guidance: >
      This section records the reviews of the work. An issue that closed without delivered
      work, or before reviews were part of the workflow, holds only the line `Not reviewed:`
      and the reasons. Each reason is one sentence, and the line can give several.
    blocks:
      - kind: prose
        optional: true
        max: { hard: 1, when-at-limit: "Give each further reason as a sentence of its own in the one `Not reviewed:` line. Never join several reasons into one sentence to stay within the count." }
        each:
          template: 'Not reviewed: ${}.'
          example: 'Not reviewed: dropped, because the plumber''s next release removes the rule sets.'
  - heading: Double-check
    level: 3
    optional: true
    only: { state: [closed], kind: [feature, bug, chore] }
    guidance: >
      This section records the double-check of the implementation, the tests, and the
      documents. A round is one run of the review and its fixes, and the last round leaves
      no correctness finding, or ends a code review at its third round with each open
      correctness finding moved to an issue of its own. The list holds each finding that was
      fixed, and each finding that was left.
    blocks:
      - kind: prose
        max: { hard: 1, when-at-limit: "Put each finding in the list that follows it." }
        each:
          format: review-rounds
          example: 'Rounds: 3'
      - kind: bullet-list
        optional: true
        rules:
          - format: finding-fixed
            min: 0
            example: 'The file server dropped the failed pattern of a rule set with one rule. (fixed)'
          - format: finding-left
            min: 0
            example: 'The name `failpat` could be `failedpattern`. (left: both names say what the variable holds)'
          - format: finding-moved
            min: 0
            to-type: Issue
            example: 'The error names no pattern of an earlier rule set. (moved: [issue-012][sokf:issue-012-feature-earlier-rule-sets])'
  - heading: Code review
    level: 3
    optional: true
    only: { state: [closed], kind: [feature, bug, chore] }
    guidance: >
      This section records the code review, in the same form as the double-check. An issue
      whose branch changed no code has none.
    blocks:
      - kind: prose
        max: { hard: 1, when-at-limit: "Put each finding in the list that follows it." }
        each:
          format: review-rounds
          example: 'Rounds: 2'
      - kind: bullet-list
        optional: true
        rules:
          - format: finding-fixed
            min: 0
            example: 'The matcher read each rule set twice. (fixed)'
          - format: finding-left
            min: 0
            example: 'The loop in `matchruleset` could return early. (left: the function is 12 lines long)'
          - format: finding-moved
            min: 0
            to-type: Issue
            example: 'The file server copies the error for each message. (moved: [issue-013][sokf:issue-013-chore-error-copies])'

  - heading: Outcome
    level: 2
    only: { state: [closed] }
    guidance: >
      This section says what the work delivered, with a link to each document that the work
      changed. When another issue's work included this one's, the section says so, and the
      issue links the other issue with relates-to. Work that is still wanted but left undone
      becomes a new issue before this one closes, because a list of work in a closed issue is
      read by nobody who plans the next work. The section describes the result, because git
      records the steps, the counts, and the dates. Each result has a sentence of its own,
      and the subsection Differs from the plan holds each place where the work departed from
      the plan.
    blocks:
      - kind: prose
        each:
          prohibited-patterns: [{pattern: '\b20[0-9]{2}-[0-9]{2}-[0-9]{2}\b', message: 'The text contains no date, because git records when the work happened.'}, {pattern: '^Step [0-9]+:', message: 'An item begins with no step number, because the outcome describes the result, not the steps.'}]
        example: 'With the attribute `explain=yes`, the `plumb` command prints the pattern that failed in the last rule set, as [PLUMB-1_AC-5][sokf:contract-plumbing#plumb-1_ac-5] says. The [plumber''s design][sokf:design-plumber#plmb-1] describes the new argument of `matchruleset`.'

  - heading: Differs from the plan
    level: 3
    optional: true
    only: { state: [closed] }
    guidance: >
      This section lists each place where the work departed from the plan, and why, one
      place in each item. Each reason is one sentence, and an item can give several.
    blocks:
      - kind: bullet-list
        each:
          prohibited-patterns: [{pattern: '\b20[0-9]{2}-[0-9]{2}-[0-9]{2}\b', message: 'The text contains no date, because git records when the work happened.'}, {pattern: '^Step [0-9]+:', message: 'An item begins with no step number, because the outcome describes the result, not the steps.'}]
        example: 'The error names the failed pattern of the last rule set only, because an error string of 9P, the Plan 9 file protocol, holds at most 128 bytes.'
````
