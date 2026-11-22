# Contributing to tramlines

This repository is a read-only mirror of tramlines. Each release reaches it as one commit, which an export from the project's own repository creates, and nobody commits to it directly. This guide explains how to report a bug or ask for a feature, and how to build tramlines from this repository.

- [Report a bug or ask for a feature](#report-a-bug-or-ask-for-a-feature)
- [Build tramlines](#build-tramlines)

## Report a bug or ask for a feature

Open an issue, with the bug report or the feature request template. A pull request can't be merged here, because the next release replaces the whole tree. When you have a change in mind, describe it in an issue, and include a patch there if you have one.

## Build tramlines

The project pins its toolchains in `rust-toolchain.toml`, in the files of `.config/mise/conf.d/`, and in `.mise.toml`, which [mise](https://mise.jdx.dev/) installs:

```sh
mise install
cargo build --locked -p tramlines
```

The specifications and the grammars are in `knowledge/spec/`, and the build embeds the grammars into the crates. The documentation site's pages are in `docs/`, and `mise run docs:build` builds the site with mdBook into `target/book`.

To build the npm packages as the release does, install the JavaScript workspace and run the build task of a platform, such as Linux on x64:

```sh
npm install
mise run build:release:ubuntu:linux-x64
```

`mise tasks ls --hidden` lists a task for each of the five platforms, and the middle part of each name says which operating system it runs on. Each of these tasks builds the binary, puts it in its platform package, and smoke-tests it when it can run on the machine. One more task, `build:release:ubuntu:assets`, builds no binary: it puts the JSON Schema of a schema in `dist/assets/`, which the GitHub release attaches.

`.github/workflows/publish.yml` is the release: with `build.yml`, it builds the five binaries, smoke-tests them, publishes the crates to crates.io and the packages to npm, creates the GitHub release with the JSON Schema of a schema and the notes that the tag's message carries, and deploys the documentation site through `docs.yml`. The GitHub releases hold the notes of every release: they are the project's changelog.
