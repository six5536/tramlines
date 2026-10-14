#!/usr/bin/env node
// Behavioural smoke test for a compiled tramlines binary: run the real thing
// and assert what a user sees — the version line, a document that keeps its
// schema, one that breaks it, and the exit codes 0, 1 and 2. Release CI runs
// it against each built artifact its runner can execute; locally run
// `npm run smoke` after `cargo build --release -p tramlines`.
//
// This deliberately mirrors crates/app/tramlines/tests/it/cli.rs: those tests
// prove the code is right, this proves the *shipped artifact* is. Extend
// both together.

import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const { binaryName } = createRequire(import.meta.url)("../packages/tramlines/lib/binary.js");

const bin = resolve(process.argv[2] ?? join("target", "release", binaryName(process.platform)));

function fail(message) {
  console.error(`release-smoke: ${message}`);
  process.exit(1);
}

if (!existsSync(bin)) {
  fail(`no binary at ${bin} — build first: cargo build --release -p tramlines`);
}

/** Run the binary in `cwd`, asserting the exit status; returns the spawn result. */
function run(args, expectStatus, cwd) {
  const r = spawnSync(bin, args, { encoding: "utf8", cwd });
  if (r.error) {
    fail(`failed to run ${bin}: ${r.error.message}`);
  }
  if (r.status !== expectStatus) {
    console.error(r.stdout);
    console.error(r.stderr);
    fail(`\`tramlines ${args.join(" ")}\` exited ${r.status}, expected ${expectStatus}`);
  }
  return r;
}

// --- The version line, which the publish step also checks against the tag ---
const version = run(["--version"], 0).stdout.trim();
if (!/^tramlines \d+\.\d+\.\d+/.test(version)) {
  fail(`unexpected --version output: ${version}`);
}

// --- A schema and two documents: one keeps it (exit 0), one breaks it (exit 1)
const dir = mkdtempSync(join(tmpdir(), "tramlines-smoke-"));
try {
  writeFileSync(
    join(dir, "list.yaml"),
    "line-limit: 20\nfrontmatter:\n  title:\n    example: Groceries\nsections:\n  - heading: { from: title }\n    level: 1\n    blocks:\n      - kind: bullet-list\n        example: Milk\n",
  );
  writeFileSync(join(dir, "good.md"), "---\ntitle: Groceries\n---\n\n# Groceries\n\n- Milk\n");
  writeFileSync(join(dir, "bad.md"), "---\ntitle: Groceries\n---\n\n# Shopping\n\n- Milk\n");

  const good = run(["validate", "--schema", "list.yaml", "good.md"], 0, dir).stdout;
  if (good !== "0 errors, 0 warnings\n") {
    fail(`a document that keeps its schema reported: ${good}`);
  }
  const bad = run(["validate", "--schema", "list.yaml", "bad.md"], 1, dir).stdout;
  if (!bad.startsWith("bad.md:5: error[TLMD_165]: heading `Shopping` must equal the frontmatter `title`")) {
    fail(`a document that breaks its schema reported: ${bad}`);
  }

  const json = JSON.parse(run(["validate", "--schema", "list.yaml", "bad.md", "--json"], 1, dir).stdout);
  if (json.errors.length !== 1 || json.errors[0].key !== "TLMD_165") {
    fail(`--json reported: ${JSON.stringify(json)}`);
  }

  // --- Exit 2: a usage error, and a document no format matches ---------------
  run(["validate"], 2, dir);
  writeFileSync(join(dir, "notes.txt"), "text\n");
  run(["validate", "--schema", "list.yaml", "notes.txt"], 2, dir);
} finally {
  rmSync(dir, { recursive: true, force: true });
}

console.log(`release-smoke: ${version} ok`);
