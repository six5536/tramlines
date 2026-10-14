//! `Finding`, `Severity` and `Locator`: plain data that passes through
//! the cache.

use serde::{Deserialize, Serialize};

use crate::RepoPath;

// sokf-implements: CORE-8_AC-8
// sokf:begin finding
/// How a finding is reported.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Severity {
    /// A failure. Exit 1.
    #[default]
    Error,
    /// What the validator cannot settle alone.
    Warning,
}
// sokf:end finding

// sokf:begin finding
impl Severity {
    /// How a finding line names the severity: `error` or `warning`.
    pub fn as_str(self) -> &'static str {
        // sokf:end finding
        // sokf-implements: CORE-8_AC-10
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

// sokf:begin finding
/// Where in a schema the rule that failed sits: the `id` of the schema,
/// and the rule's `rule-id`, else its section and block position. A
/// finding that a schema rule raises ends with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locator {
    /// The `id` of the schema.
    pub schema: String,
    /// Where the rule sits in the schema, when the check knows.
    pub rule: Option<String>,
}
// sokf:end finding

impl Locator {
    /// The schema `id`, with no rule named.
    pub fn schema(id: impl Into<String>) -> Locator {
        Locator {
            schema: id.into(),
            rule: None,
        }
    }

    /// The same schema, at the rule `at`.
    pub fn at(&self, at: impl Into<String>) -> Locator {
        Locator {
            schema: self.schema.clone(),
            rule: Some(at.into()),
        }
    }

    fn suffix(&self) -> String {
        match &self.rule {
            // sokf-implements: CORE-8_AC-2
            None => format!("(schema {})", self.schema),
            // The reader the finding sends to the schema lands in a
            // schema of hundreds of lines; the locator says where.
            // sokf-implements: CORE-8_AC-3
            // sokf-implements: TL_73
            Some(at) => format!("(schema {}, {at})", self.schema),
        }
    }
}

// Each finding names the criterion of its one fault, which the
// specification keeps to one fault with one fix.
// sokf-implements: TL_100
// sokf:begin finding
/// One line of the report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Finding {
    /// The file the finding is on.
    pub path: RepoPath,
    /// The line, when one applies.
    pub line: Option<usize>,
    /// Error or warning.
    pub severity: Severity,
    /// The key of the criterion that the finding breaks, as `TLMD_57`.
    pub criterion: String,
    /// The message, ending with the location of its rule in the schema
    /// when a schema rule raised it.
    pub message: String,
    /// The line in the schema of the rule that raised it, where the check
    /// knows one: what a finding on the schema itself sits on when the
    /// schema's own scaffold fails the rule. The report does not print it.
    #[serde(skip)]
    pub rule_line: Option<usize>,
}
// sokf:end finding

impl Finding {
    /// Build a finding that breaks `criterion`.
    // sokf-implements: TL_72
    // sokf-implements: CORE-8_AC-1
    // sokf:begin finding
    pub fn new(
        path: RepoPath,
        line: Option<usize>,
        severity: Severity,
        criterion: &str,
        message: impl Into<String>,
    ) -> Self {
        // sokf:end finding
        // sokf-implements: CORE-8_AC-9
        debug_assert!(
            is_criterion(criterion),
            "`{criterion}` is not a criterion key"
        );
        Finding {
            path,
            line,
            severity,
            criterion: criterion.to_owned(),
            message: message.into(),
            rule_line: None,
        }
    }

    /// An error finding.
    // sokf-implements: CORE-8_AC-7
    pub fn error(
        path: RepoPath,
        line: Option<usize>,
        criterion: &str,
        message: impl Into<String>,
    ) -> Self {
        Self::new(path, line, Severity::Error, criterion, message)
    }

    /// A warning finding.
    // sokf-implements: CORE-8_AC-7
    pub fn warning(
        path: RepoPath,
        line: Option<usize>,
        criterion: &str,
        message: impl Into<String>,
    ) -> Self {
        Self::new(path, line, Severity::Warning, criterion, message)
    }

    /// The finding, raised by the schema rule on line `line` of the
    /// schema; a rule with no line of its own, at `0`, names none.
    #[must_use]
    pub fn of_rule(mut self, line: usize) -> Self {
        self.rule_line = (line > 0).then_some(line);
        self
    }

    /// The finding, with the location of the schema rule that raised it
    /// appended to its message.
    #[must_use]
    pub fn at(mut self, locator: &Locator) -> Self {
        self.message = format!("{} {}", self.message, locator.suffix());
        self
    }
}

/// Whether `key` has the form of a criterion key: upper-case words joined
/// by `_`, an optional `-n` part, then `_n`, `_AC-n` or `_P-n`, as
/// `TLMD_57`, `CLI-4_AC-5` or `SOKF_FORMAT_53`.
fn is_criterion(key: &str) -> bool {
    let Some((head, tail)) = key.rsplit_once('_') else {
        return false;
    };
    let number = tail
        .strip_prefix("AC-")
        .or_else(|| tail.strip_prefix("P-"))
        .unwrap_or(tail);
    let (words, part) = head.split_once('-').unwrap_or((head, "1"));
    !number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit())
        && !part.is_empty()
        && part.bytes().all(|b| b.is_ascii_digit())
        && words
            .split('_')
            .all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_uppercase()))
}

/// A failure found before the file it is on is known: the criterion it
/// breaks, and what failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The key of the criterion, as `TL_25`.
    pub criterion: &'static str,
    /// What failed.
    pub message: String,
}

impl Problem {
    /// A problem that breaks `criterion`.
    pub fn new(criterion: &'static str, message: impl Into<String>) -> Self {
        Problem {
            criterion,
            message: message.into(),
        }
    }

    /// The problem, with `prefix` before its message.
    #[must_use]
    pub fn prefixed(mut self, prefix: &str) -> Self {
        self.message = format!("{prefix}{}", self.message);
        self
    }

    /// The problem as an error finding on `path` at `line`.
    pub fn error(self, path: RepoPath, line: Option<usize>) -> Finding {
        Finding::error(path, line, self.criterion, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RepoPath {
        RepoPath::parse(s).unwrap()
    }

    // sokf-tests: TL_72
    // sokf-tests: TL_73
    #[test]
    fn a_finding_carries_its_criterion_and_its_rule() {
        let f = Finding::error(p("/a.md"), Some(3), "TLMD_109", "bad");
        assert_eq!(
            (f.criterion.as_str(), f.message.as_str()),
            ("TLMD_109", "bad")
        );
        assert_eq!(f.severity, Severity::Error);
        let readme = Locator::schema("readme");
        let f = Finding::warning(p("/a.md"), None, "TL_74", "bad").at(&readme);
        assert_eq!(f.message, "bad (schema readme)");
        assert_eq!(f.severity, Severity::Warning);
        let f = Finding::error(p("/a.md"), None, "TL_74", "bad").at(&readme.at("section \"A\""));
        assert_eq!(f.message, "bad (schema readme, section \"A\")");
    }

    #[test]
    fn a_criterion_key_has_its_form() {
        for key in [
            "TL_1",
            "TLMD_57",
            "CLI-4_AC-5",
            "CORE_P-2",
            "SOKF_FORMAT_53",
        ] {
            assert!(is_criterion(key), "{key}");
        }
        for key in ["unit", "TL", "TL_", "tl_1", "CLI-4", "CLI-x_AC-1", "_1"] {
            assert!(!is_criterion(key), "{key}");
        }
    }

    proptest::proptest! {
        // sokf-tests: CORE_P-5
        #[test]
        fn a_locator_ends_the_message(message in "[ -~]{0,40}", schema in "[a-z]{1,8}") {
            let f = Finding::error(p("/a.md"), None, "TL_74", message.clone())
                .at(&Locator::schema(schema.clone()));
            proptest::prop_assert_eq!(f.criterion.as_str(), "TL_74");
            proptest::prop_assert!(f.message.starts_with(&message));
            let suffix = format!("(schema {schema})");
            proptest::prop_assert!(f.message.ends_with(&suffix));
        }
    }

    // sokf-tests: CORE_P-1
    // sokf-tests: CORE-10_AC-2
    #[test]
    fn findings_round_trip_through_serde() {
        let f = Finding::error(p("/a.md"), Some(3), "TL_53", "bad");
        let json = serde_json::to_string(&f).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }
}
