//! Compiles each pattern once per run.

use std::{cell::RefCell, collections::HashMap};

use regex::Regex;

/// A per-run cache of compiled patterns. A pattern that does not compile is
/// remembered too, so its message is stable.
#[derive(Debug, Default)]
pub struct RegexCache {
    map: RefCell<HashMap<String, Result<Regex, String>>>,
}

impl RegexCache {
    /// A fresh cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// The compiled pattern, or the compiler's message on one line: the
    /// last line of its report, which says what is wrong, since a finding
    /// is one line.
    // sokf-implements: TL_16
    pub fn get(&self, pattern: &str) -> Result<Regex, String> {
        if let Some(hit) = self.map.borrow().get(pattern) {
            return hit.clone();
        }
        let compiled = Regex::new(pattern).map_err(|e| reason(&e));
        self.map
            .borrow_mut()
            .insert(pattern.to_string(), compiled.clone());
        compiled
    }
}

/// The compiler's message for `e` on one line: the last line of its
/// report, which says what is wrong, since a finding is one line.
pub(crate) fn reason(e: &regex::Error) -> String {
    let report = e.to_string();
    let last = report.lines().rev().find(|l| !l.trim().is_empty());
    last.unwrap_or(&report)
        .trim()
        .trim_start_matches("error: ")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_once_and_remembers_failures() {
        let cache = RegexCache::new();
        let a = cache.get("^a+$").unwrap();
        assert!(a.is_match("aaa"));
        assert!(cache.get("^a+$").is_ok());
        let e = cache.get("(").unwrap_err();
        assert_eq!(cache.get("(").unwrap_err(), e);
        assert!(!e.contains('\n'), "{e}");
        assert_eq!(cache.map.borrow().len(), 2);
    }
}
