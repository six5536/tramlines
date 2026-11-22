---
type: Schema
id: schema-architecture
title: Architecture schema
description: "An architecture introduces a system to a reader new to it, in a hierarchy of diagrams: its context, its subsystems and their parts, its flows, where it runs, and its technology, with links to its contracts and designs."
---

<!-- prettier-ignore -->
````yaml
line-limit: 1500
# A `Planned:` paragraph promises work that an open issue owns, so it links no
# closed issue.
current-links: ['^Planned: ']

guidance: >
  An architecture introduces one system to a reader who has never seen it. It shows the
  building blocks of the system and how they connect, in a hierarchy of diagrams: the
  system, its subsystems, and the parts of each subsystem. Each block's design describes
  how the block is built inside. Each diagram stays small, because a larger one shows too
  much for one level.

frontmatter:
  type:
    const: Architecture
  id:
    const: architecture
  description:
    example: 'The plumbing system passes text between the interactive programs of Plan 9. A file server, the plumber, delivers each message to the program that handles it, according to the user''s rules.'

# A story cites each flow, because a flow that no user needs is waste.
cited-by:
  - from-type: Stories
    keys: flows
types:
  link-text: '[^\]]+'
  link-name: '[A-Z][^\]]*'
  path: '[^) ]+'
  block-name: '[A-Z][^:"<]*'
  purpose: '[a-z][^"<]*'
  article: [a, an]
  part-path: '[A-Z][^:"<]*: [A-Z][^:"<]*'

formats:
  group-boundary:
    template: 'subgraph ${id:word}["${:phrase}<br/><i>[Group]</i>"]'
    example: '    subgraph fsys["File server<br/><i>[Group]</i>"]'
  legend-node:
    prohibited-patterns:
      - pattern: '^(graph|subgraph|end|class|classDef|style|linkStyle|click|call|href|default|direction)[\[({]'
        message: 'A node''s id is no Mermaid keyword.'
    variants:
      user:
        template: '${:word}(["${:label}"]):::user'
        example: '    user(["User"]):::user'
      block:
        template: '${:word}["${:label}"]:::block'
        example: '    block["Block"]:::block'
      neighbour:
        template: '${:word}["${:label}"]:::neighbour'
        example: '    neighbour["Neighbour"]:::neighbour'
      external:
        template: '${:word}["${:label}"]:::external'
        example: '    external["External system"]:::external'
      store:
        template: '${:word}[("${:label}")]:::store'
        example: '    store[("Data store")]:::store'
      contract:
        template: '${:word}{{"${:label}"}}:::contract'
        example: '    contract{{"Contract"}}:::contract'
      group:
        template: '${:word}[["${:label}"]]:::group'
        example: '    group[["Group"]]:::group'
  provides:
    template: '${from:word} -.- ${to:word}'
    example: '    fsys -.- plumbing'
  nav-link-subsystem:
    template: '[Subsystem: ${:link-text}][sokf:${:store}architecture#subsystem-${:slug}]'
    example: '[Subsystem: Plumber][sokf:architecture#subsystem-plumber]'
  nav-link-group:
    template: '[Group: ${:link-text}][sokf:${:store}architecture#group-${:slug}]'
    example: '[Group: File server][sokf:architecture#group-file-server]'
  design-link:
    variants:
      written:
        template: '[${:link-name}][sokf:${:store}design-${:slug}] (${block:block-name}): ${:purpose}.'
        example: '[Plumber][sokf:design-plumber] (Plumber): how the plumber reads the rules, matches each message, and delivers it to a port.'
      planned:
        template: 'Planned: [${:link-name}][sokf:${:store}issue-${:slug}] (${block:block-name}): ${:purpose}.'
        example: 'Planned: [Plumbing library][sokf:issue-004-chore-document-plumbing-library] (Plumbing library): how the library converts a message to and from text.'
  part-design-link:
    template: '[${:link-name}][sokf:${:store}design-${:slug}] (${part:part-path}): ${:purpose}.'
    example: '[Rules reader][sokf:design-rules-reader] (File server: Rules reader): how the file server parses a rules file and reports the line of each error.'
  contract-link:
    variants:
      written:
        template: '[${name:link-name}][sokf:${:store}contract-${:slug}] (${:block-name}): ${:purpose}.'
        example: '[Plumbing][sokf:contract-plumbing] (Plumber): the files in `/mnt/plumb`, the format of a message, and what a program that sends or reads a message can rely on.'
      planned:
        template: 'Planned: [${name:link-name}][sokf:${:store}issue-${:slug}] (${:block-name}): ${:purpose}.'
        example: 'Planned: [Plumbing library][sokf:issue-004-chore-document-plumbing-library] (Plumbing library): the functions that a program calls to open a port, send a message, and read one.'

sections:
  - heading: { from: title }
    level: 1
    blocks:
      - kind: prose
        guidance: >
          This paragraph says what the architecture describes, because an architecture
          describes the structure of a system, and its reader studies that structure. The
          overview introduces the system itself.
        each:
          pattern: '^[A-Z].*\.$'
          example: 'This architecture describes the structure of the plumbing system.'
      - kind: bullet-list
        guidance: >
          The contents list has one item for each level-2 section of
          the architecture, in order. Each item links to the section's heading.
        each:
          template: '[${}](#${:slug})'
          example:
            - '[Overview](#overview)'
            - '[Context](#context)'
            - '[Subsystems](#subsystems)'
            - '[Contracts](#contracts)'
            - '[Designs](#designs)'
            - '[Flows](#flows)'
            - '[Deployment](#deployment)'
            - '[Technology](#technology)'
            - '[Legend](#legend)'

  - heading: Overview
    level: 2
    guidance: >
      The overview introduces the system. It says what the system provides, how it
      provides it, and what it connects to. Each kind of system that it connects to has a
      sentence of its own, and the Context section names each system. The Subsystems
      section says what the system is built from, and the Technology section what it builds
      on. The overview's last paragraph links to the non-goals. While the store has no
      non-goals document, a `Planned:` paragraph in its place links the open issue that
      writes the non-goals.
    blocks:
      - kind: prose
        example:
          - 'The plumbing system passes text between the interactive programs of Plan 9. A program writes a message to `/mnt/plumb/send`, and the plumber delivers it to the port that the user''s rules choose. A program can send text without knowing which program handles it.'
          - 'The plumbing system connects the window system, the editor, the image viewer, and the mail file system. It reads each user''s rules from the Plan 9 file server.'
          - 'The [non-goals][sokf:non-goals] say what the plumbing system doesn''t do, and why.'

  - heading: Context
    level: 2
    guidance: >
      This section shows who uses the system at run time and which external systems it
      depends on or serves. The Deployment section shows how the system is delivered and
      hosted.
    blocks:
      - kind: group
        max: { hard: 1, when-at-limit: "Draw the system's parts in the Subsystems section, and each request in a Flow section." }
        guidance: The section has one caption and one diagram of the system in its context.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves as
              the diagram's title, because a reader reads it first. It states the diagram's
              main point: how the system relates to the people and systems around it. Each
              exchange that it describes has a sentence of its own, and the caption takes as
              many paragraphs as its exchanges need. The overview says more about what the
              system connects to.
            each:
              format: caption
              example: A user clicks on text in an interactive program. The plumbing system delivers the text to the program that handles it. The plumbing system reads each user's rules from the Plan 9 file server.
          - kind: code
            guidance: >
              The diagram shows only what a reader needs at this level: the system as one
              block, its users, and the external systems that it connects to, each as a
              node of its own. The people whom the system serves appear as users. An
              artificial intelligence (AI) agent or other software that acts for them
              appears as an external system. Where the nodes don't fit, an external system
              that only one subsystem uses appears in that subsystem's diagram instead, and
              one that several subsystems use stays in this diagram.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                flowchart LR
                accTitle: The plumbing system in its context
                  subgraph canvas[" "]
                    user(["User<br/><i>a person</i>"]):::user
                    plumbing["Plumbing system<br/><i>C</i>"]:::block
                    rio["Window system<br/><i>rio</i>"]:::external
                    editor["Editor<br/><i>acme</i>"]:::external
                    viewer["Image viewer<br/><i>page</i>"]:::external
                    mailfs["Mail file system<br/><i>upas/fs</i>"]:::external
                    fileserver["Plan 9 file server<br/><i>a separate machine</i>"]:::external
                    user -->|"Clicks on<br/><i>text</i>"| rio
                    rio -->|"Sends<br/><i>messages</i>"| plumbing
                    editor -->|"Sends and reads<br/><i>messages</i>"| plumbing
                    mailfs -->|"Sends<br/><i>messages</i>"| plumbing
                    plumbing -->|"Delivers<br/><i>messages</i>"| viewer
                    plumbing -->|"Reads over 9P<br/><i>rules</i>"| fileserver
                  end
                  classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
                  classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
                  classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
                  classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
                  classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
                  classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
                  classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Give a node its colours through its class." }
                - format: flowchart-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw how the system is delivered and hosted in the Deployment section." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: node
                  min: 2
                  max: { hard: 9, when-at-limit: "Draw one user for each kind of person, and move each external system that only one subsystem uses to that subsystem's diagram. Never fold several external systems into one node to stay within the count." }
                - select-pattern: ':::block$'
                  rule-id: context-block
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw the system's parts in the Subsystems section." }
                - format: arrow
                  min: 1
                  max: { hard: 9, when-at-limit: "Join the exchanges between two nodes into one arrow, such as `Sends and reads`, and move each external system that only one subsystem uses to that subsystem's diagram. Never join the exchanges with different systems into one arrow to stay within the count." }
                - format: class-style
                  min: 7
                  max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
                - select-pattern: '^classDef user '
                  min: 1
                - select-pattern: '^classDef block '
                  min: 1
                - select-pattern: '^classDef neighbour '
                  min: 1
                - select-pattern: '^classDef external '
                  min: 1
                - select-pattern: '^classDef store '
                  min: 1
                - select-pattern: '^classDef contract '
                  min: 1
                - select-pattern: '^classDef group '
                  min: 1
                - format: canvas
                  min: 1
                  max: { hard: 1, when-at-limit: "Mark a system that runs on another machine in its label, such as `a separate machine`." }
                - format: ground-end
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw the system's parts in the Subsystems section, and each external system as a node of its own, outside any boundary." }

  - heading: Subsystems
    level: 2
    guidance: >
      This section shows the subsystems that make up the system, the external systems they
      connect to, and the stores they use.
    blocks:
      - kind: group
        max: { hard: 1, when-at-limit: "Show each request between the subsystems in a Flow section." }
        guidance: The section has one caption and one diagram, which shows every subsystem.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves as
              the diagram's title, because a reader reads it first. It says how the system
              is divided and what each subsystem is responsible for. Each exchange that it
              describes has a sentence of its own, and the caption takes as many paragraphs
              as its exchanges need. Each subsystem's own section says more about the
              subsystem.
            each:
              format: caption
              example: The plumbing system consists of the plumber, the plumbing library `libplumb`, and the plumb command. The window system, the mail file system, and the plumb command send messages through the library. The image viewer reads messages through it. The editor both sends and reads messages through it. The library reaches the plumber over 9P, the Plan 9 file protocol. The plumber reads the user's rules from `lib/plumbing`.
          - kind: code
            guidance: >
              The diagram shows only what a reader needs at this level: every subsystem, and
              the external systems and stores that the subsystems connect to, each as a node
              of its own. It stays small, because a reader follows each arrow as one step of
              a flow, and a larger diagram shows too much for one level. It has few
              subsystems, because a reader holds only a few things in mind at once, and the
              subsystems are what a reader takes from this level. An external system or a
              store that several subsystems use is a node of this diagram, with an arrow from
              each subsystem that uses it, because this is the level where those subsystems
              meet. Where the nodes don't fit, an external system or a store that only one
              subsystem uses appears in that subsystem's diagram instead. Each arrow's label
              says what the node at its tail does, and what the node at its head does is an
              arrow of its own.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                flowchart LR
                accTitle: The subsystems of the plumbing system
                  subgraph canvas[" "]
                    subgraph system["Plumbing system<br/><i>[System]</i>"]
                      plumber["Plumber<br/><i>/sys/src/cmd/plumb</i>"]:::block
                      libplumb["Plumbing library<br/><i>/sys/src/libplumb</i>"]:::block
                      plumbcmd["Plumb command<br/><i>/sys/src/cmd/plumb/plumb.c</i>"]:::block
                    end
                    rio["Window system<br/><i>rio</i>"]:::external
                    editor["Editor<br/><i>acme</i>"]:::external
                    viewer["Image viewer<br/><i>page</i>"]:::external
                    mailfs["Mail file system<br/><i>upas/fs</i>"]:::external
                    rules[("Plumbing rules<br/><i>lib/plumbing on the file server</i>")]:::store
                    rio -->|"Sends through<br/><i>message</i>"| libplumb
                    editor -->|"Sends and reads through<br/><i>messages</i>"| libplumb
                    viewer -->|"Reads through<br/><i>message</i>"| libplumb
                    mailfs -->|"Sends through<br/><i>message</i>"| libplumb
                    plumbcmd -->|"Sends through<br/><i>message</i>"| libplumb
                    libplumb -->|"Writes and reads over 9P<br/><i>messages</i>"| plumber
                    plumber -->|"Reads over 9P<br/><i>rules</i>"| rules
                  end
                  style system fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
                  classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
                  classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
                  classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
                  classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
                  classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
                  classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
                  classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Give a node its colours through its class." }
                - format: flowchart-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw where the subsystems run in the Deployment section." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: node
                  min: 2
                  max: { hard: 9, when-at-limit: "Move each external system or store that only one subsystem uses to that subsystem's diagram, or join related subsystems into one larger subsystem. Never fold several external systems or stores into one node, or draw a store that several subsystems use as an arrow to its owner, to stay within the count." }
                - select-pattern: ':::block$'
                  rule-id: subsystems
                  key-of: subsystems
                  min: 1
                  max: { hard: 7, when-at-limit: "Join related subsystems into one larger subsystem. Its diagram draws each of them as a group, and each group has a Group section of its own." }
                  pattern: '^\w+\["(?P<name>[^"<]+)'
                  key: name
                  example: '      plumber["Plumber<br/><i>/sys/src/cmd/plumb</i>"]:::block'
                - select-pattern: ':::contract$'
                  rule-id: subsystems-contract
                  min: 0
                  max: { hard: 0, when-at-limit: "Draw each contract in the diagram of the subsystem that provides it." }
                - format: arrow
                  min: 1
                  max: { hard: 9, when-at-limit: "Join the exchanges between two nodes into one arrow, such as `Sends and reads`, move each external system or store that only one subsystem uses to that subsystem's diagram, or join related subsystems into one larger subsystem. Never join the exchanges with different systems into one arrow, or draw a store that several subsystems use as an arrow to its owner, to stay within the count." }
                - select-pattern: ' -\.- '
                  rule-id: subsystems-contract-line
                  min: 0
                  max: { hard: 0, when-at-limit: "Name the subsystem that provides each contract in the Contracts section." }
                - format: class-style
                  min: 7
                  max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
                - select-pattern: '^classDef user '
                  min: 1
                - select-pattern: '^classDef block '
                  min: 1
                - select-pattern: '^classDef neighbour '
                  min: 1
                - select-pattern: '^classDef external '
                  min: 1
                - select-pattern: '^classDef store '
                  min: 1
                - select-pattern: '^classDef contract '
                  min: 1
                - select-pattern: '^classDef group '
                  min: 1
                - format: canvas
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw each host as a boundary in a Deployment diagram." }
                - format: boundary
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw the inside of each subsystem in the subsystem's own section." }
                - format: boundary-style
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw another system as one external system, with no boundary of its own." }
                - format: ground-end
                  min: 2
                  max: { hard: 2, when-at-limit: "Draw subsystems that belong together as one subsystem, rather than inside a further boundary." }
      - kind: prose
        max: { hard: 1, when-at-limit: "Say what each subsystem does in the caption of the diagram." }
        each:
          text: 'Each subsystem has a section of its own, which shows its parts:'
      - kind: bullet-list
        guidance: The list links to the section of each subsystem, which shows the subsystem's parts.
        each:
          format: nav-link-subsystem
          example:
            - '[Subsystem: Plumber][sokf:architecture#subsystem-plumber]'
            - '[Subsystem: Plumbing library][sokf:architecture#subsystem-plumbing-library]'
            - '[Subsystem: Plumb command][sokf:architecture#subsystem-plumb-command]'

  - heading: { template: 'Subsystem: ${name}', key: name, example: "Subsystem: Plumber" }
    level: 3
    repeat: true
    for-each-key-of: subsystems
    rule-id: subsystem
    guidance: >
      Each subsystem section shows the parts of one subsystem and how a request moves
      between them. Each part appears once in the diagram of the parts, as a block. A
      part of another subsystem appears as a neighbour when the subsystem's parts call
      it, so an exchange between two subsystems always appears in the caller's diagram.
      The called subsystem's diagram draws each external system that only this subsystem
      uses. It draws any other caller only where its reader needs the caller to see which
      part takes each kind of request. One such caller for each kind of request is
      enough, because the Subsystems diagram and the callers' own diagrams show every
      other caller. An external system, a neighbour, or a store that several parts or
      groups of the subsystem use is a node of the subsystem's diagram, because that is
      the level where they meet. The diagram connects each contract of the subsystem to the part that
      provides it. A contract that the subsystem only uses appears in its provider's
      diagram. A contract never takes the place of a caller: where the contracts leave no
      room for the callers, a second diagram draws them.
    examples:
      - |
        ### Subsystem: Plumbing library

        The editor opens a port with the port opener. It sends messages through the sender. It reads messages through the receiver. The sender and the receiver call the codec, which converts a `Plumbmsg` to and from text. The port opener, the sender, and the receiver reach the plumber's file server over 9P.

        ```mermaid
        %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
        flowchart LR
        accTitle: The parts of the plumbing library
          subgraph canvas[" "]
            subgraph libplumb["Plumbing library<br/><i>[Subsystem]</i>"]
              opener["Port opener<br/><i>plumbopen</i>"]:::block
              sender["Sender<br/><i>plumbsend</i>"]:::block
              receiver["Receiver<br/><i>plumbrecv</i>"]:::block
              codec["Codec<br/><i>plumbpack and plumbunpack</i>"]:::block
            end
            editor["Editor<br/><i>acme</i>"]:::external
            fsys["File server<br/><i>plumber</i>"]:::neighbour
            editor -->|"Calls<br/><i>plumbopen</i>"| opener
            editor -->|"Calls<br/><i>plumbsend</i>"| sender
            editor -->|"Calls<br/><i>plumbrecv</i>"| receiver
            sender -->|"Calls<br/><i>plumbpack</i>"| codec
            receiver -->|"Calls<br/><i>plumbunpack</i>"| codec
            opener -->|"Opens over 9P<br/><i>port</i>"| fsys
            sender -->|"Writes over 9P<br/><i>message</i>"| fsys
            receiver -->|"Reads over 9P<br/><i>message</i>"| fsys
          end
          style libplumb fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
          classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
          classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
          classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
          classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
          classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
          classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
          classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
        ```
      - |
        ### Subsystem: Plumb command

        The shell runs the message builder. The message builder sets the fields of a message from the command's arguments. It calls the input reader when the `-i` option takes the data from standard input. It then sends the message through the sender of the plumbing library.

        ```mermaid
        %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
        flowchart LR
        accTitle: The parts of the plumb command
          subgraph canvas[" "]
            subgraph plumbcmd["Plumb command<br/><i>[Subsystem]</i>"]
              builder["Message builder<br/><i>main</i>"]:::block
              input["Input reader<br/><i>gather</i>"]:::block
            end
            shell["Shell<br/><i>rc</i>"]:::external
            sender["Sender<br/><i>plumbing library</i>"]:::neighbour
            shell -->|"Runs<br/><i>plumb</i>"| builder
            builder -->|"Calls<br/><i>gather</i>"| input
            builder -->|"Calls<br/><i>plumbsend</i>"| sender
          end
          style plumbcmd fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
          classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
          classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
          classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
          classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
          classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
          classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
          classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
        ```
    blocks:
      - kind: group
        max: { hard: 2, when-at-limit: "Draw the inside of each group, with the stores that only its parts use, in the group's Group section." }
        guidance: >
          The section has one caption and one diagram. A subsystem whose diagram can't hold
          its parts, what they use, its contracts, and a caller for each kind of request, such
          as a subsystem with one part, has a second caption and a second diagram. The second
          diagram draws the callers and the parts that they call, and the first keeps the
          parts, what they use, and the contracts.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves
              as the diagram's title, because a reader reads it first. It says how the parts
              of the subsystem work together. Each exchange that it describes has a sentence
              of its own, and the caption takes as many paragraphs as its exchanges need. The
              subsystem's design says what each part does inside, and each group's Group
              section draws the group's parts. A `Planned:` paragraph can follow the caption.
              It describes a part that isn't built yet, and links the open issue that owns
              the change. The part's node and section are added, and the paragraph removed,
              when the part is built, before the issue closes.
            sequence:
              - format: caption
                repeat: true
                example:
                  - At startup, the startup module calls the rules reader. The rules reader writes the rule sets in memory. The startup module then starts the file server.
                  - The file server collects each message that a program writes. It calls the matcher for each message. The matcher tests the message against the rule sets. The file server then queues the message on the destination port.
              - pattern: '^Planned: [A-Z].*\[[^\]]+\]\[sokf:([a-z0-9-]+:)?issue-[0-9]+-[a-z0-9-]+\].*\.$'
                optional: true
                example: 'Planned: The matcher reports which pattern failed when a message with the attribute `explain=yes` matches no rule set, as [the issue on the failed pattern][sokf:issue-001-feature-report-failed-pattern] describes.'
          - kind: code
            guidance: >
              The diagram shows only what a reader needs at this level: the subsystem's
              parts and groups, and the stores, neighbours, and external systems that they
              connect to, each as a node of its own. It shows each group with the group's
              inputs and outputs, so that each level fits in one diagram. A store that only
              the parts of one group use appears in that group's diagram instead, and one that
              several groups or parts use stays in this diagram.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                flowchart LR
                accTitle: The parts of the plumber
                  subgraph canvas[" "]
                    subgraph plumber["Plumber<br/><i>[Subsystem]</i>"]
                      startup["Startup<br/><i>plumber.c</i>"]:::block
                      reader["Rules reader<br/><i>rules.c</i>"]:::block
                      matcher["Matcher<br/><i>match.c</i>"]:::block
                      fsys[["File server<br/><i>fsys.c</i>"]]:::group
                      rules[("Rule sets<br/><i>in memory</i>")]:::store
                    end
                    editor["Editor<br/><i>acme</i>"]:::external
                    plumbing{{"Plumbing<br/><i>public</i>"}}:::contract
                    editor -->|"Writes<br/><i>message</i>"| fsys
                    startup -->|"Calls<br/><i>readrules</i>"| reader
                    startup -->|"Starts<br/><i>file server</i>"| fsys
                    reader -->|"Writes<br/><i>rule sets</i>"| rules
                    fsys -->|"Calls<br/><i>matchruleset</i>"| matcher
                    matcher -->|"Reads<br/><i>rule sets</i>"| rules
                    fsys -.- plumbing
                  end
                  style plumber fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
                  classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
                  classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
                  classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
                  classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
                  classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
                  classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
                  classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Give a node its colours through its class." }
                - format: flowchart-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Show how the parts carry out each task in the subsystem's design." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: node
                  min: 2
                  max: { hard: 9, when-at-limit: "Draw related parts together as one group, with a Group section of its own, and move each store that only the parts of one group use to that group's diagram. Draw the callers in a second diagram, with a caption of its own, where the parts, what they use, and the contracts leave no room for them. Never fold several external systems, neighbours, or stores into one node, or leave out a caller or what a part uses to make room for a contract, to stay within the count." }
                - select-pattern: ':::(block|group)$'
                  rule-id: part-or-group
                  min: 1
                  max: { hard: 7, when-at-limit: "Join related groups into one larger group, or divide the subsystem into two subsystems." }
                - select-pattern: ':::block$'
                  rule-id: parts
                  key-of: parts
                  key-repeats: true
                  min: 0
                  pattern: '^\w+\["(?P<name>[^"<]+)<br/>'
                  key: name
                  example: '      startup["Startup<br/><i>plumber.c</i>"]:::block'
                - select-pattern: ':::group$'
                  rule-id: groups
                  key-of: groups
                  min: 0
                  pattern: '^\w+\[\["(?P<name>[^"<]+)<br/>'
                  key: name
                  key-repeats: true
                  example: '      fsys[["File server<br/><i>fsys.c</i>"]]:::group'
                - select-pattern: ':::contract$'
                  rule-id: contracts
                  key-of: contracts
                  min: 0
                  pattern: '\{\{"(?P<name>[^"<]+)<br/>'
                  key: name
                  example: '    plumbing{{"Plumbing<br/><i>public</i>"}}:::contract'
                - format: arrow
                  min: 1
                  max: { hard: 9, when-at-limit: "Join the exchanges between two nodes into one arrow, such as `Sends and reads`, and draw related parts together as one group, with a Group section of its own. Draw the callers in a second diagram, with a caption of its own, where the parts, what they use, and the contracts leave no room for them. Never join the exchanges with different systems or parts into one arrow to stay within the count." }
                - format: provides
                  min: 0
                  max: { hard: 9, when-at-limit: "Give the parts that provide the other contracts a subsystem of their own. Never leave out a caller to make room for a contract: draw the callers in a second diagram, with a caption of its own." }
                - format: class-style
                  min: 7
                  max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
                - select-pattern: '^classDef user '
                  min: 1
                - select-pattern: '^classDef block '
                  min: 1
                - select-pattern: '^classDef neighbour '
                  min: 1
                - select-pattern: '^classDef external '
                  min: 1
                - select-pattern: '^classDef store '
                  min: 1
                - select-pattern: '^classDef contract '
                  min: 1
                - select-pattern: '^classDef group '
                  min: 1
                - format: canvas
                  min: 1
                  max: { hard: 1, when-at-limit: "Show the hosts that the parts run on in the Deployment section." }
                - format: boundary
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw a part of another subsystem as a neighbour, outside the boundary." }
                - format: boundary-style
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw another system as one external system, with no boundary of its own." }
                - format: ground-end
                  min: 2
                  max: { hard: 2, when-at-limit: "Describe what is inside a part in the subsystem's design." }
      - kind: group
        optional: true
        max: { hard: 1, when-at-limit: "Link every group in the one list." }
        blocks:
          - kind: prose
            max: { hard: 1, when-at-limit: "Say what is inside each group in its Group section." }
            each:
              text: 'Each group in the diagram has a Group section of its own:'
          - kind: bullet-list
            guidance: The list links to the Group section of each group in the subsystem's diagram.
            each:
              format: nav-link-group

  - heading: { template: 'Group: ${name}', key: name, example: "Group: File server" }
    level: 4
    repeat: true
    for-each-key-of: groups
    parent: subsystem
    guidance: >
      Each Group section shows what is inside one group of the parent subsystem, and
      is named after the group. A store appears inside the group when only the
      group's parts use it. A store that other parts also use appears in the
      subsystem's diagram.
    blocks:
      - kind: group
        max: { hard: 1, when-at-limit: "Draw some of the group's parts outside the group, in the subsystem's diagram." }
        guidance: The section has one caption and one diagram.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves
              as the diagram's title, because a reader reads it first. It says what each part
              inside the group does. Each exchange that it describes has a sentence of its
              own, and the caption takes as many paragraphs as its exchanges need. The
              group's design holds the detail of each part. A `Planned:` paragraph can follow
              the caption. It describes a part that isn't built yet, and links the open issue
              that owns the change. The part's node is added, and the paragraph removed, when
              the part is built, before the issue closes.
            sequence:
              - format: caption
                repeat: true
                example: The request handler answers each 9P request. It passes each complete message to the dispatcher. The dispatcher appends the message to the queue of the destination port. The request handler takes the message from the queue when a program reads the port.
              - pattern: '^Planned: [A-Z].*\[[^\]]+\]\[sokf:([a-z0-9-]+:)?issue-[0-9]+-[a-z0-9-]+\].*\.$'
                optional: true
                example: 'Planned: The request handler returns an error that names the failed pattern to the program that writes to `send`, as [the issue on the failed pattern][sokf:issue-001-feature-report-failed-pattern] describes.'
          - kind: code
            guidance: >
              The diagram shows only the group's contents, inside a boundary named after
              the group, because the subsystem's diagram already shows the group's
              inputs and outputs. The contents are the group's parts and the stores that
              only those parts use, each as a node of its own.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                flowchart LR
                accTitle: The parts of the file server
                  subgraph canvas[" "]
                    subgraph fsys["File server<br/><i>[Group]</i>"]
                      handler["Request handler<br/><i>fsys.c</i>"]:::block
                      dispatcher["Dispatcher<br/><i>dispose</i>"]:::block
                      queues[("Port queues<br/><i>in memory</i>")]:::store
                    end
                    handler -->|"Passes<br/><i>message</i>"| dispatcher
                    dispatcher -->|"Appends<br/><i>message</i>"| queues
                    handler -->|"Takes<br/><i>message for a read</i>"| queues
                  end
                  style fsys fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
                  classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
                  classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
                  classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
                  classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
                  classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
                  classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
                  classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Give a node its colours through its class." }
                - format: flowchart-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Show how the parts carry out each task in the group's design." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: node
                  min: 2
                  max: { hard: 9, when-at-limit: "Move each store that parts outside the group also use to the subsystem's diagram, or split the group into two groups, each with a Group section of its own. Never fold several stores into one node to stay within the count." }
                - select-pattern: ':::block$'
                  rule-id: group-parts
                  key-of: group-parts
                  min: 1
                  max: { hard: 7, when-at-limit: "Split the group into two groups, each with a Group section of its own." }
                  pattern: '^\w+\["(?P<name>[^"<]+)<br/>'
                  key: name
                  example: '      handler["Request handler<br/><i>fsys.c</i>"]:::block'
                - select-pattern: ':::(user|neighbour|external|contract|group)$'
                  rule-id: group-outside-node
                  min: 0
                  max: { hard: 0, when-at-limit: "Draw what the group connects to in the subsystem's diagram." }
                - format: arrow
                  min: 1
                  max: { hard: 9, when-at-limit: "Join the exchanges between two nodes into one arrow, such as `Sends and reads`, or split the group into two groups, each with a Group section of its own. Never join the exchanges with different parts or stores into one arrow to stay within the count." }
                - select-pattern: ' -\.- '
                  rule-id: group-contract-line
                  min: 0
                  max: { hard: 0, when-at-limit: "Name the group as the provider of its contracts in the Contracts section." }
                - format: class-style
                  min: 7
                  max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
                - select-pattern: '^classDef user '
                  min: 1
                - select-pattern: '^classDef block '
                  min: 1
                - select-pattern: '^classDef neighbour '
                  min: 1
                - select-pattern: '^classDef external '
                  min: 1
                - select-pattern: '^classDef store '
                  min: 1
                - select-pattern: '^classDef contract '
                  min: 1
                - select-pattern: '^classDef group '
                  min: 1
                - format: canvas
                  min: 1
                  max: { hard: 1, when-at-limit: "Show the hosts that the parts run on in the Deployment section." }
                - format: group-boundary
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the inside of a part in a design of its own." }
                - format: boundary-style
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw each other group of the subsystem in its own Group section." }
                - format: ground-end
                  min: 2
                  max: { hard: 2, when-at-limit: "Draw a group within the group as one more group of the subsystem." }

  - heading: Contracts
    level: 2
    guidance: >
      This section lists each contract in the diagrams of the subsystems, with the
      building block that provides it. Each contract appears once, and no other section of
      the architecture lists contracts.
    blocks:
      - kind: bullet-list
        guidance: >
          Each item links to the Contract document, names in parentheses the block or the
          group that the diagram attaches the contract to, and says what the contract
          defines, so that a reader knows where the link leads before following it. The
          Designs section lists a design for each block named here. A contract that isn't
          written yet has a `Planned:` item, whose link has the contract's name as its text
          and leads to the open issue that writes the contract.
        rules:
          - format: contract-link
            key: name
            example: '[Plumbing][sokf:contract-plumbing] (Plumber): the files in `/mnt/plumb`, the format of a message, and what a program that sends or reads a message can rely on.'
            for-each-key-of: contracts
          - select-pattern: '\] \('
            pattern: '\] \((?P<block>[^)]+)\): '
            key: block
            key-repeats: true
            rule-id: providers
            key-of: providers
            min: 1
            example: '[Plumbing][sokf:contract-plumbing] (Plumber): the files in `/mnt/plumb`, the format of a message, and what a program that sends or reads a message can rely on.'

  - heading: Designs
    level: 2
    guidance: >
      This section lists the design of each subsystem, of each group, and of each block
      that provides a contract, because those are the blocks whose inside a reader needs
      explained. A part of a group can have a design of its own when the group's design
      would otherwise grow too long.
    blocks:
      - kind: bullet-list
        guidance: >
          Each item links to the Design document, names in parentheses the subsystem, the
          group, or the block that the design covers, and says what the design explains. A
          part's design names its group, a colon, and the part. A design that isn't
          written yet has a `Planned:` item, whose link leads to the open issue that writes
          the design.
        rules:
          - format: design-link
            key: block
            for-each-key-of: [subsystems, groups, providers]
            example: '[Plumber][sokf:design-plumber] (Plumber): how the plumber reads the rules, matches each message, and delivers it to a port.'
          - format: part-design-link
            key: part
            key-derives-from: { rule: groups, separator: ': ' }
            min: 0
            example: '[Rules reader][sokf:design-rules-reader] (File server: Rules reader): how the file server parses a rules file and reports the line of each error.'

  - heading: Flows
    level: 2
    guidance: >
      This section shows how the subsystems work together in each important request, with
      one subsection for each request.
    blocks: []
  - heading: { template: 'Flow: ${name}', key: name, example: "Flow: Open a file from an error message" }
    level: 3
    repeat: true
    rule-id: flows
    guidance: >
      Each flow follows one important request, from the first call to the response. A
      story realises each flow and cites it by its key, because a flow that no user needs
      is waste. A flow that no story cites yet gets a story of its own in a Stories
      document.
    examples:
      - |
        ### Flow: Show an image

        The user clicks on `fig1.jpg` in the editor when no image viewer is running. The plumber starts the image viewer with a `plumb client` action. It holds the message until the image viewer opens the `image` port.

        ```mermaid
        %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
        sequenceDiagram
        accTitle: Showing an image
          box rgb(246, 248, 250)
          participant editor as Editor
          participant plumber as Plumber
          participant viewer as Image viewer
          end
          editor->>plumber: write(send, message)
          plumber->>viewer: start page
          plumber-->>editor: the write returns
          viewer->>plumber: open(image)
          plumber-->>viewer: read(image) returns the message
        ```
    blocks:
      - kind: group
        guidance: Each diagram in a flow has its own caption before it.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves as
              the diagram's title, because a reader reads it first. It says what happens in
              the request, step by step. Each step has a sentence of its own, and the caption
              takes as many paragraphs as its steps need. The design of the subsystem that
              takes a step holds the step's detail.
            each:
              format: caption
              example: The user clicks on `mem.c:27` in a compiler's error message. The window system sends the text to the plumber. The plumber matches the text against the rules. It delivers the text to the `edit` port. The editor shows line 27 of `mem.c`.
          - kind: code
            guidance: >
              The participants are the subsystems and the external systems that the
              request passes through, each as a participant of its own. Each message holds one
              step of one part or external system, and where two parts of one participant
              act, each step is a message of its own. What happens inside one subsystem
              appears in the Runtime behaviour section of the subsystem's design.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                sequenceDiagram
                accTitle: Opening a file from an error message
                  box rgb(246, 248, 250)
                  participant rio as Window system
                  participant plumber as Plumber
                  participant editor as Editor
                  end
                  editor->>plumber: read(edit), which waits
                  rio->>plumber: write(send, message)
                  plumber->>plumber: matchruleset(message, rule set)
                  plumber-->>editor: read(edit) returns the message
                  plumber-->>rio: the write returns
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Start each further diagram in a fence of its own." }
                - format: sequence-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Give each further request a Flow section of its own." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: sequence-message
                  min: 1
                  max: { hard: 15, when-at-limit: "Split the request into two diagrams, each with its own caption." }
                - format: box
                  min: 1
                  max: { hard: 1, when-at-limit: "Show the modules inside a subsystem in the Runtime behaviour section of the subsystem's design." }
                - format: participant
                  min: 2
                  max: { hard: 9, when-at-limit: "Draw each subsystem as one participant, or split the request into two diagrams, each with its own caption. Never fold several external systems into one participant to stay within the count." }
                - format: ground-end
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw each other path in a diagram of its own, with its own caption." }

  - heading: Deployment
    level: 2
    guidance: >
      This section shows where each part of the system runs and which hosted services the
      system relies on.
    blocks:
      - kind: group
        guidance: >
          Each environment that the system runs in, such as test and production, has one
          caption and one diagram, and the caption names the environment. An environment
          too large for one diagram has a caption and a diagram for each subsystem. Every
          environment follows this rule, including one that runs the same parts as another,
          such as staging.
        blocks:
          - kind: prose
            guidance: >
              The caption comes first, before the diagram that it describes, and serves as
              the diagram's title, because a reader reads it first. It says where the system
              runs and what it relies on there. Each host and each exchange that it
              describes has a sentence of its own, and the caption takes as many paragraphs
              as they need. The Technology section says more about what the system builds
              on.
            each:
              format: caption
              example: On each user's terminal, one plumber runs beside the window system, the editor, the image viewer, and the mail file system. The plumber reads the rules from the file server. The mail file system reads the mailbox from the file server.
          - kind: code
            guidance: >
              The diagram shows one environment, and only what a reader needs at this level:
              the hosts, the blocks that run on each host, and the external systems and
              stores that the blocks rely on, each as a node of its own. The parts, the
              external systems, and the stores appear again in each environment's diagrams,
              because each environment runs them or relies on them. In an environment with a
              diagram for each subsystem, an external system or a store that only one
              subsystem uses appears in that subsystem's diagram alone, and one that several
              subsystems use appears in the diagram of each of them.
            lang: mermaid
            each:
              example: |
                %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
                flowchart LR
                accTitle: Where the plumbing system runs
                  subgraph canvas[" "]
                    subgraph terminal["Terminal<br/><i>[Host]</i>"]
                      plumber["Plumber<br/><i>one process for each user</i>"]:::block
                      rio["Window system<br/><i>rio</i>"]:::external
                      editor["Editor<br/><i>acme</i>"]:::external
                      viewer["Image viewer<br/><i>page</i>"]:::external
                      mailfs["Mail file system<br/><i>upas/fs</i>"]:::external
                    end
                    subgraph server["File server<br/><i>[Host]</i>"]
                      files[("User files<br/><i>lib/plumbing and mail</i>")]:::store
                    end
                    plumber -->|"Reads over 9P<br/><i>rules</i>"| files
                    mailfs -->|"Reads over 9P<br/><i>mailbox</i>"| files
                  end
                  style terminal fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
                  style server fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
                  classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
                  classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
                  classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
                  classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
                  classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
                  classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
                  classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              lines:
                - format: theme
                  min: 1
                  max: { hard: 1, when-at-limit: "Give a node its colours through its class." }
                - format: flowchart-type
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw each further environment in a diagram of its own, after a caption of its own." }
                - format: acc-title
                  min: 1
                  max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
                - format: acc-descr
                  min: 0
                  max: { hard: 1, when-at-limit: "Put the rest in the caption." }
                - format: node
                  min: 2
                  max: { hard: 9, when-at-limit: "Draw the environment in a diagram for each subsystem, each after a caption of its own, with the external systems and stores that the subsystem uses. Take this way out in every environment, a staging or a test environment as well as production. Never fold several external systems or stores into one node to stay within the count." }
                - select-pattern: ':::block$'
                  rule-id: deployment-block
                  min: 1
                  max: { hard: 7, when-at-limit: "Draw the parts that always run together as one block." }
                - format: arrow
                  min: 1
                  max: { hard: 9, when-at-limit: "Join the exchanges between two nodes into one arrow, such as `Sends and reads`, or draw the environment in a diagram for each subsystem, each after a caption of its own. Never join the exchanges with different systems into one arrow to stay within the count." }
                - format: provides
                  min: 0
                  max: { hard: 9, when-at-limit: "Connect the other contracts in the diagrams of their subsystems." }
                - format: class-style
                  min: 7
                  max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
                - select-pattern: '^classDef user '
                  min: 1
                - select-pattern: '^classDef block '
                  min: 1
                - select-pattern: '^classDef neighbour '
                  min: 1
                - select-pattern: '^classDef external '
                  min: 1
                - select-pattern: '^classDef store '
                  min: 1
                - select-pattern: '^classDef contract '
                  min: 1
                - select-pattern: '^classDef group '
                  min: 1
                - format: canvas
                  min: 1
                  max: { hard: 1, when-at-limit: "Draw each host as a boundary inside the canvas." }
                - format: boundary
                  min: 1
                - format: boundary-style
                  min: 1
                - format: ground-end
                  min: 2
                  max: { hard: 8, when-at-limit: "Draw the hosts that run the same parts as one host." }

  - heading: Technology
    level: 2
    guidance: >
      This section lists the technologies that shape the system, such as its languages,
      runtimes, stores, and libraries. The build manifests record the versions and the
      other libraries.
    blocks: []
  - heading: { template: 'Technology: ${:block-name}', example: "Technology: Plumber" }
    level: 3
    repeat: true
    guidance: >
      Each subsection lists the technology choices for one subsystem, in the order of the
      Subsystems section. The subsection _Technology: Build_ lists the choices that belong
      to no single subsystem. The heading begins with _Technology_, so that its anchor
      differs from the anchor of the subsystem's section.
    examples:
      - |
        ### Technology: Build

        - **mk**: The `mk` command builds the plumber from the `mkfile` in its directory. Why: `mk` builds every Plan 9 program, so the plumber needs no build tool of its own.
    blocks:
      - kind: bullet-list
        guidance: >
          Each item begins with the name of the choice in bold and a colon. A sentence with the
          choice as its subject says what the choice does, and the reasons for the choice
          follow `Why:`. Each reason is one sentence, and an item can give several. Where a
          design describes the choice in depth, the item links to the design.
        each:
          template: '**${}**: ${}. Why: ${}.'
          example:
            - '**C**: C is the language of the plumber and of the programs that send it messages. Why: the plumber shares its types, such as `Plumbmsg`, with those programs through `libplumb`.'
            - '**9P**: 9P, the Plan 9 file protocol, carries every read and write of the files in `/mnt/plumb`. Why: every Plan 9 program can already read and write files, on its own machine or across the network ([the design of the plumber][sokf:design-plumber]).'
            - '**regexp**: The `regexp` library matches the patterns of the rules. Why: it finds the longest leftmost match, which the `click` attribute needs.'

  - heading: Legend
    level: 2
    guidance: >
      The schema fixes the legend, so that each class of node has one shape and one colour
      in every diagram. The schema fixes its one sentence too.
    blocks:
      - kind: prose
        max: { hard: 1, when-at-limit: "Explain a diagram in its own caption." }
        each:
          text: Every diagram in this document uses these shapes and colours.
      - kind: code
        lang: mermaid
        each:
          example: |
            %%{init: {"theme": "neutral", "themeVariables": {"darkMode": false, "background": "#f6f8fa", "clusterBkg": "#f6f8fa", "clusterBorder": "#d0d0d0", "edgeLabelBackground": "#f6f8fa", "lineColor": "#555555", "textColor": "#1a1a1a", "primaryTextColor": "#1a1a1a", "actorBkg": "#ffffff", "actorBorder": "#6c8ebf", "actorLineColor": "#767676", "actorTextColor": "#1a1a1a", "signalColor": "#555555", "signalTextColor": "#1a1a1a", "labelBoxBkgColor": "#f5f5f5", "labelBoxBorderColor": "#6c8ebf", "labelTextColor": "#1a1a1a", "classText": "#1a1a1a"}}}%%
            flowchart LR
            accTitle: The classes and the colours of these diagrams
              subgraph canvas[" "]
                subgraph boundary["Boundary<br/><i>[Level noun]</i>"]
                  block["Block"]:::block
                end
                user(["User"]):::user
                neighbour["Neighbour"]:::neighbour
                external["External system"]:::external
                store[("Data store")]:::store
                contract{{"Contract"}}:::contract
                group[["Group"]]:::group
                user -->|"Verb<br/><i>payload</i>"| block
                block -.- contract
              end
              style boundary fill:#f6f8fa,stroke:#999999,stroke-dasharray: 5 5
              classDef user fill:#ffffff,stroke:#666666,color:#1a1a1a
              classDef block fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
              classDef neighbour fill:#eaf2fb,stroke:#a9c4e0,color:#1a1a1a
              classDef external fill:#eeeeee,stroke:#999999,color:#1a1a1a
              classDef store fill:#ffe6cc,stroke:#d79b00,color:#1a1a1a
              classDef contract fill:#e1d5e7,stroke:#9673a6,color:#1a1a1a
              classDef group fill:#dae8fc,stroke:#6c8ebf,color:#1a1a1a
          lines:
            - format: theme
              min: 1
              max: { hard: 1, when-at-limit: "Give each class its colours in its `classDef` line." }
            - format: flowchart-type
              min: 1
              max: { hard: 1, when-at-limit: "Draw each further diagram in the section that it belongs to." }
            - format: acc-title
              min: 1
              max: { hard: 1, when-at-limit: "Describe the diagram in the `accDescr` line." }
            - format: acc-descr
              min: 0
              max: { hard: 1, when-at-limit: "Explain each class in the list after the diagram." }
            - format: legend-node
              min: 7
              max: { hard: 7, when-at-limit: "Show further nodes of a class in the diagrams that the legend explains." }
            - format: arrow
              min: 1
              max: { hard: 1, when-at-limit: "Draw each real exchange in the diagram of its level." }
            - format: provides
              min: 1
              max: { hard: 1, when-at-limit: "Connect each real contract to its provider in its subsystem's diagram." }
            - format: class-style
              min: 7
              max: { hard: 7, when-at-limit: "Say in a node's label what sets it apart from the others of its class." }
            - select-pattern: '^classDef user '
              min: 1
            - select-pattern: '^classDef block '
              min: 1
            - select-pattern: '^classDef neighbour '
              min: 1
            - select-pattern: '^classDef external '
              min: 1
            - select-pattern: '^classDef store '
              min: 1
            - select-pattern: '^classDef contract '
              min: 1
            - select-pattern: '^classDef group '
              min: 1
            - format: canvas
              min: 1
              max: { hard: 1, when-at-limit: "Draw the other examples of boundaries in the diagrams that the legend explains." }
            - format: boundary
              min: 1
              max: { hard: 1, when-at-limit: "Write the level's noun, such as System or Host, in each diagram's own boundary." }
            - format: boundary-style
              min: 1
              max: { hard: 1, when-at-limit: "Draw several hosts, each with a `style` line of its own, in the Deployment diagram." }
            - format: ground-end
              min: 2
              max: { hard: 2, when-at-limit: "Draw a boundary within the boundary as a group node." }
      - kind: bullet-list
        guidance: >
          The list says what each class of node in the diagram stands for, in the order of
          the diagram, and links the glossary term of the class, so that the glossary
          defines each kind of node once.
        rules:
          - template: '**${:phrase}**: ${:article} [${:link-text}][sokf:${:store}glossary-workflow#${:slug}].'
            example:
            - '**Block**: a [building block][sokf:glossary-workflow#building-block].'
            - '**User**: a [user][sokf:glossary-workflow#user].'
            - '**Neighbour**: a [neighbour][sokf:glossary-workflow#neighbour].'
            - '**External system**: an [external system][sokf:glossary-workflow#external-system].'
            - '**Data store**: a [data store][sokf:glossary-workflow#data-store].'
            - '**Contract**: a [contract][sokf:glossary-workflow#contract].'
            - '**Group**: a [group][sokf:glossary-workflow#group].'
          - select-pattern: 'glossary-workflow#building-block\]'
            rule-id: legend-building-block-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a building block." }
          - select-pattern: 'glossary-workflow#user\]'
            rule-id: legend-user-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a user." }
          - select-pattern: 'glossary-workflow#neighbour\]'
            rule-id: legend-neighbour-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a neighbour." }
          - select-pattern: 'glossary-workflow#external-system\]'
            rule-id: legend-external-system-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for an external system." }
          - select-pattern: 'glossary-workflow#data-store\]'
            rule-id: legend-data-store-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a data store." }
          - select-pattern: 'glossary-workflow#contract\]'
            rule-id: legend-contract-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a contract." }
          - select-pattern: 'glossary-workflow#group\]'
            rule-id: legend-group-item
            min: 1
            max: { hard: 1, when-at-limit: "Say more in the glossary entry for a group." }

````
