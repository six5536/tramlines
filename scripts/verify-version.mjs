#!/usr/bin/env node
// Assert that one version is used consistently everywhere: the Cargo workspace,
// the internal crate pins, every package.json under packages/, the
// launcher's optionalDependencies, and both lockfiles.
//
// Usage: node scripts/verify-version.mjs [expected-version]
//
// With no argument it only checks internal consistency. With one, it also
// checks everything matches that version — this is how the release workflow
// verifies a tag against the tree.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import {
  LIBRARY_CRATES,
  WORKSPACE_VERSION,
  isPlatformPackage,
  packageJsons,
  pin,
  root,
} from "./versions.mjs";

const expected = process.argv[2]?.replace(/^v/, "");

/** Every place a version is written, as { where, version } records. */
const found = [];
const problems = [];

const record = (where, version) => {
  if (version === undefined) {
    problems.push(`could not read a version from ${where}`);
    return;
  }
  found.push({ where, version });
};

// --- Cargo.toml: workspace version + the internal crate pins ----------------
const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
record("Cargo.toml [workspace.package] version", cargo.match(WORKSPACE_VERSION)?.[1]);
for (const crate of LIBRARY_CRATES) {
  record(`Cargo.toml ${crate} dependency pin`, cargo.match(pin(crate))?.[2]);
}

// --- packages/*/package.json + the launcher's optionalDependencies -----------
for (const { name, json } of packageJsons()) {
  record(`packages/${name}/package.json version`, json.version);
  for (const [dep, range] of Object.entries(json.optionalDependencies ?? {})) {
    if (isPlatformPackage(dep)) {
      record(`packages/${name}/package.json optionalDependencies["${dep}"]`, range);
    }
  }
}

// --- Cargo.lock -------------------------------------------------------------
const cargoLock = readFileSync(join(root, "Cargo.lock"), "utf8");
for (const crate of ["tramlines", ...LIBRARY_CRATES]) {
  const re = new RegExp(`name = "${crate}"\\nversion = "([^"]*)"`);
  record(`Cargo.lock ${crate}`, cargoLock.match(re)?.[1]);
}

// --- package-lock.json ------------------------------------------------------
const npmLock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8"));
for (const [key, entry] of Object.entries(npmLock.packages ?? {})) {
  if (key.startsWith("packages/") && entry.version) {
    record(`package-lock.json ${key}`, entry.version);
  }
}

// --- Verdict ----------------------------------------------------------------
const versions = [...new Set(found.map((f) => f.version))];
const target = expected ?? versions[0];

if (versions.length !== 1) {
  problems.push(`inconsistent versions across the tree: ${versions.join(", ")}`);
}
for (const { where, version } of found) {
  if (version !== target) {
    problems.push(`${where}: found ${version}, expected ${target}`);
  }
}

if (problems.length > 0) {
  console.error("version check failed:");
  for (const p of problems) console.error(`  - ${p}`);
  console.error("\nRun `npm run set-version <version>` to fix.");
  process.exit(1);
}

console.log(`version ${target} is consistent across ${found.length} locations`);
