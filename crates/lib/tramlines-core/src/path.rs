//! `RepoPath`: a `/`-rooted, forward-slash path from the repository root,
//! the only path type in the engine and its host. Conversion to `Path`
//! happens only in the host's walk and in file writes.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The repository root on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoRoot(PathBuf);

impl RepoRoot {
    /// Wrap a directory.
    pub fn new(path: PathBuf) -> Self {
        RepoRoot(path)
    }

    /// The directory.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// A path relative to the repository root, which starts with `/` and uses
/// forward slashes. The path `/` is the root itself.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
#[schemars(extend("pattern" = "^/"))]
pub struct RepoPath(String);

impl RepoPath {
    /// The repository root.
    pub fn root() -> Self {
        RepoPath("/".into())
    }

    /// Parse a `/`-rooted path. Rejects a relative path, an empty segment, a
    /// `.` or `..` segment, and a trailing slash (except on the root).
    pub fn parse(s: &str) -> Option<Self> {
        if s == "/" {
            return Some(Self::root());
        }
        let rest = s.strip_prefix('/')?;
        if rest.is_empty() {
            return None;
        }
        for segment in rest.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return None;
            }
        }
        Some(RepoPath(s.to_string()))
    }

    /// Build from a filesystem path inside `root`. `path` is absolute or
    /// relative to the current directory; it is normalised lexically, so a
    /// `..` that leaves the root yields `None`.
    pub fn from_fs(root: &RepoRoot, path: &Path) -> Option<Self> {
        let abs = normalize(&std::path::absolute(path).ok()?);
        let root_abs = normalize(root.as_path());
        let rel = abs.strip_prefix(&root_abs).ok()?;
        let parts = rel
            .components()
            .map(|c| c.as_os_str().to_str())
            .collect::<Option<Vec<_>>>()?;
        Self::parse(&format!("/{}", parts.join("/")))
    }

    /// The filesystem path under `root`.
    pub fn to_fs(&self, root: &RepoRoot) -> PathBuf {
        let mut p = root.as_path().to_path_buf();
        for segment in self.segments() {
            p.push(segment);
        }
        p
    }

    /// The containing directory; `None` for the root.
    pub fn parent(&self) -> Option<RepoPath> {
        if self.0 == "/" {
            return None;
        }
        match self.0.rfind('/') {
            Some(0) => Some(Self::root()),
            Some(i) => Some(RepoPath(self.0[..i].to_string())),
            None => None,
        }
    }

    /// Whether this path is `dir` or lies under it.
    pub fn starts_with(&self, dir: &RepoPath) -> bool {
        if dir.0 == "/" {
            return true;
        }
        self.0 == dir.0
            || self
                .0
                .strip_prefix(&dir.0)
                .is_some_and(|r| r.starts_with('/'))
    }

    /// The path without its leading slash, as the report prints it.
    pub fn relative(&self) -> &str {
        self.0.strip_prefix('/').unwrap_or("")
    }

    /// The path as written, `/`-rooted.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path segments.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.relative().split('/').filter(|s| !s.is_empty())
    }

    /// The last segment; empty for the root.
    pub fn file_name(&self) -> &str {
        self.segments().last().unwrap_or("")
    }

    /// The text after the last `.` of the file name, if any.
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name();
        name.rfind('.').filter(|&i| i > 0).map(|i| &name[i + 1..])
    }

    /// `self/name`.
    pub fn join(&self, name: &str) -> Option<RepoPath> {
        if self.0 == "/" {
            Self::parse(&format!("/{name}"))
        } else {
            Self::parse(&format!("{}/{name}", self.0))
        }
    }

    /// Resolve `target` from this file's directory, as standard markdown
    /// resolves a relative link. `..` segments are applied; leaving the root
    /// yields `None`.
    pub fn resolve_relative(&self, target: &str) -> Option<RepoPath> {
        let dir = self.parent()?;
        let mut parts: Vec<&str> = dir.segments().collect();
        for segment in target.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    parts.pop()?;
                }
                s => parts.push(s),
            }
        }
        if parts.is_empty() {
            return Some(Self::root());
        }
        Self::parse(&format!("/{}", parts.join("/")))
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Lexical normalisation: drop `.` and apply `..`, without touching the disk.
fn normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RepoPath {
        RepoPath::parse(s).unwrap()
    }

    #[test]
    fn parses_rooted_paths_only() {
        assert_eq!(p("/a/b.md").as_str(), "/a/b.md");
        assert_eq!(p("/").as_str(), "/");
        for bad in ["a/b", "", "/a/", "/a//b", "/a/../b", "/./a", "/.."] {
            assert!(RepoPath::parse(bad).is_none(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn parent_and_starts_with() {
        assert_eq!(p("/a/b/c.md").parent(), Some(p("/a/b")));
        assert_eq!(p("/a").parent(), Some(p("/")));
        assert_eq!(p("/").parent(), None);
        assert!(p("/a/b/c.md").starts_with(&p("/a/b")));
        assert!(p("/a/b").starts_with(&p("/a/b")));
        assert!(!p("/a/bc").starts_with(&p("/a/b")));
        assert!(p("/x").starts_with(&p("/")));
    }

    #[test]
    fn relative_names_and_extension() {
        assert_eq!(p("/a/b.md").relative(), "a/b.md");
        assert_eq!(p("/").relative(), "");
        assert_eq!(p("/a/b.md").file_name(), "b.md");
        assert_eq!(p("/a/b.md").extension(), Some("md"));
        assert_eq!(p("/a/Makefile").extension(), None);
        assert_eq!(p("/a/.hidden").extension(), None);
        assert_eq!(p("/a").join("b"), Some(p("/a/b")));
        assert_eq!(p("/").join("b"), Some(p("/b")));
        assert_eq!(p("/a/b.md").to_string(), "/a/b.md");
    }

    #[test]
    fn resolves_relative_links_from_the_directory() {
        assert_eq!(p("/k/a.md").resolve_relative("b.md"), Some(p("/k/b.md")));
        assert_eq!(
            p("/k/sub/a.md").resolve_relative("../b.md"),
            Some(p("/k/b.md"))
        );
        assert_eq!(
            p("/k/a.md").resolve_relative("./c/d.md"),
            Some(p("/k/c/d.md"))
        );
        assert_eq!(p("/k/a.md").resolve_relative("../../x"), None);
        assert_eq!(p("/a.md").resolve_relative(".."), None);
        assert_eq!(p("/k/a.md").resolve_relative(".."), Some(p("/")));
    }

    #[test]
    fn converts_to_and_from_the_filesystem() {
        let dir = std::env::temp_dir().join(format!("tramlines-path-{}", std::process::id()));
        let root = RepoRoot::new(dir.clone());
        let fs = p("/a/b.md").to_fs(&root);
        assert_eq!(fs, dir.join("a").join("b.md"));
        assert_eq!(RepoPath::from_fs(&root, &fs), Some(p("/a/b.md")));
        assert_eq!(RepoPath::from_fs(&root, &dir), Some(p("/")));
        assert_eq!(RepoPath::from_fs(&root, &dir.join("a/../c")), Some(p("/c")));
        assert_eq!(RepoPath::from_fs(&root, &dir.join("..")), None);
    }

    #[test]
    fn serializes_as_a_string() {
        let json = serde_json::to_string(&p("/a.md")).unwrap();
        assert_eq!(json, "\"/a.md\"");
        let back: RepoPath = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p("/a.md"));
    }
}
