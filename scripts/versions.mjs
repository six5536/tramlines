// Where the project writes its version, shared by set-version, verify-version
// and release: one version in lockstep across the Cargo workspace, the
// internal crate pins, and every package.json under packages/.

import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

/** The repository root. */
export const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** A version the project accepts: semver, with an optional pre-release. */
export const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

/** The `[workspace.package]` version line of Cargo.toml; group 1 is the version. */
export const WORKSPACE_VERSION = /^version = "([^"]*)"$/m;

/** The library crates that the workspace pins by path and version. */
export const LIBRARY_CRATES = ["tramlines-core", "tramlines-md"];

/** The pin of `crate` in Cargo.toml; group 2 is the version. */
export const pin = (crate) =>
  new RegExp(`(${crate} = \\{ path = "crates/lib/${crate}", version = ")([^"]*)(" \\})`);

/** Whether an optional dependency is one of the platform packages. */
export const isPlatformPackage = (dep) => dep.startsWith("@six5536/tramlines-");

/** Each packages/<name>/package.json that parses, as { name, path, json }. */
export function packageJsons() {
  const out = [];
  for (const name of readdirSync(join(root, "packages"))) {
    const path = join(root, "packages", name, "package.json");
    try {
      out.push({ name, path, json: JSON.parse(readFileSync(path, "utf8")) });
    } catch {
      // A directory with no package.json is no package.
    }
  }
  return out;
}
