#!/usr/bin/env node
// Behavioural smoke test for a compiled tramlines binary: run the real thing
// and assert what a user sees — the version line, a document that keeps its
// schema, one that breaks it, and the exit codes 0, 1 and 2. mise run
// test:smoke runs it after the launcher's smoke test, which builds the host's
// release binary, and the export's verify runs it on the export's own build.
//
// This deliberately mirrors crates/app/tramlines/tests/it/cli.rs: those tests
// prove the code is right, this proves the *shipped artifact* is. Extend
// both together.

import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

// A release build stages the binary in the host's platform package, so the
// default is that artifact, and otherwise the local release build.
const exe = process.platform === "win32" ? "tramlines.exe" : "tramlines";
const staged = join("packages", `tramlines-${process.platform}-${process.arch}`, "bin", exe);
const bin = resolve(
  process.argv[2] ?? (existsSync(staged) ? staged : join("target", "release", exe)),
);

/** isRecord reports whether a parsed JSON value is an object whose fields can be read. */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function fail(message: string): never {
  console.error(`release-smoke: ${message}`);
  process.exit(1);
}

if (!existsSync(bin)) {
  fail(`no binary at ${bin} — build first: cargo build --release -p tramlines`);
}

/** Run the binary in `cwd`, asserting the exit status; returns the spawn result. */
function run(args: string[], expectStatus: number, cwd?: string) {
  const r = spawnSync(bin, args, { encoding: "utf8", cwd });
  if (r.error) {
    fail(`failed to run ${bin}: ${r.error.message}`);
  }
  if (r.status !== expectStatus) {
    console.error(r.stdout);
    console.error(r.stderr);
    fail(`\`tramlines ${args.join(" ")}\` exited ${String(r.status)}, expected ${expectStatus}`);
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
// fail() exits at once, so the exit, of a pass or a failure, removes the folder.
process.on("exit", () => {
  rmSync(dir, { recursive: true, force: true });
});
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
if (
  !bad.startsWith(
    "bad.md:5: error[TLMD_165]: heading `Shopping` must equal the frontmatter `title`",
  )
) {
  fail(`a document that breaks its schema reported: ${bad}`);
}

const json: unknown = JSON.parse(
  run(["validate", "--schema", "list.yaml", "bad.md", "--json"], 1, dir).stdout,
);
const errors = isRecord(json) && Array.isArray(json.errors) ? (json.errors as unknown[]) : [];
const first = errors.at(0);
if (errors.length !== 1 || !isRecord(first) || first.key !== "TLMD_165") {
  fail(`--json reported: ${JSON.stringify(json)}`);
}

// --- Exit 2: a usage error, and a document no format matches ---------------
run(["validate"], 2, dir);
writeFileSync(join(dir, "notes.txt"), "text\n");
run(["validate", "--schema", "list.yaml", "notes.txt"], 2, dir);

console.log(`release-smoke: ${version} ok`);
