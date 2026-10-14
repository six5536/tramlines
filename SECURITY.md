# Security policy

## Supported versions

tramlines is pre-1.0. Security fixes target only the **latest release** and the `main`
branch. Older versions receive no patches. To pick up a fix, upgrade to the
latest release.

## Reporting a vulnerability

Report a security issue **privately**, through GitHub's private vulnerability
reporting:

1. Open the [Security tab](https://github.com/six5536/tramlines/security) of the
   repository.
2. To start a private advisory, click **Report a vulnerability**.

Don't open a public issue for a suspected vulnerability. The maintainer
acknowledges the report and coordinates a fix and a disclosure date with you.

## Scope

tramlines reads the schema and the documents that its caller names. It writes
only the documents that its caller names, and only with `--fix`. It opens no
network connection, and it runs no code from a schema. A schema's regular
expressions run in the Rust `regex` dialect, which has no backtracking. A schema
therefore can't make a check take exponential time.

The following behaviours are in scope: anything that makes tramlines read or
write a file that it wasn't given, or run without bound on a schema or a
document.
