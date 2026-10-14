//! Embeds the core grammar.
//!
//! In the workspace the files live at the repository root, outside this
//! crate, so they are copied into `OUT_DIR` at build time. The same paths
//! under the crate's own directory are the fallback for a packaged crate:
//! there each is a symlink to the canonical file, which `cargo package`
//! copies into the package. The workspace copy wins when both exist, so the
//! canonical files stay the single source.

use std::{env, fs, path::PathBuf};

/// The embedded files, each relative to the repository root.
const FILES: &[&str] = &["knowledge/spec/tramlines.grammar.yaml"];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let workspace = manifest.join("../../..");
    for rel in FILES {
        let source = [workspace.join(rel), manifest.join(rel)]
            .into_iter()
            .find(|p| p.is_file())
            .unwrap_or_else(|| {
                panic!(
                    "{rel} not found under {} or {}",
                    workspace.display(),
                    manifest.display()
                )
            });
        println!("cargo:rerun-if-changed={}", source.display());
        let text = fs::read_to_string(&source).expect("read embedded file");
        let target = out.join(rel);
        fs::create_dir_all(target.parent().expect("parent")).expect("create OUT_DIR subdirectory");
        fs::write(target, text).expect("write embedded file");
    }
}
