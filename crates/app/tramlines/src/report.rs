//! The report of `validate`: the files that `--fix` rewrote and the
//! findings, as text, as one JSON object, or as GitHub workflow commands.

use serde_json::{Value, json};
use tramlines_md::{Finding, Severity};

use crate::Stop;

/// What one run of `validate` found and repaired.
#[derive(Debug, Default)]
pub struct Report {
    /// Each finding, with the path of its file as the command line spells it.
    findings: Vec<(String, Finding)>,
    /// The files that `--fix` rewrote.
    fixes: Vec<String>,
}

impl Report {
    /// Add the findings on the file at `path`.
    pub fn add(&mut self, path: &str, findings: impl IntoIterator<Item = Finding>) {
        self.findings
            .extend(findings.into_iter().map(|f| (path.to_string(), f)));
    }

    /// Record that `--fix` rewrote the file at `path`.
    pub fn rewrote(&mut self, path: &str) {
        self.fixes.push(path.to_string());
    }

    /// Sort the findings by path, then line, then message, a finding with
    /// no line first, and the fixes by path.
    // sokf-implements: CLI-4_AC-5
    // sokf-implements: CLI-4_AC-6
    pub fn finish(&mut self) {
        self.findings
            .sort_by(|(a, f), (b, g)| (a, f.line, &f.message).cmp(&(b, g.line, &g.message)));
        self.fixes.sort();
    }

    /// Whether a finding is an error.
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|(_, f)| f.severity == Severity::Error)
    }

    /// The findings of one severity.
    fn of(&self, severity: Severity) -> impl Iterator<Item = &(String, Finding)> {
        self.findings
            .iter()
            .filter(move |(_, f)| f.severity == severity)
    }

    /// The findings that a listing shows: the errors, and the warnings
    /// where `warnings` asks for them.
    fn listed(&self, warnings: bool) -> impl Iterator<Item = &(String, Finding)> {
        self.findings
            .iter()
            .filter(move |(_, f)| warnings || f.severity == Severity::Error)
    }

    /// The count line: `<n> errors, <m> warnings`.
    // sokf-implements: CLI-4_AC-7
    // sokf-implements: CLI-4_AC-8
    fn counts(&self) -> String {
        let plural = |n: usize, what: &str| format!("{n} {what}{}", if n == 1 { "" } else { "s" });
        format!(
            "{}, {}\n",
            plural(self.of(Severity::Error).count(), "error"),
            plural(self.of(Severity::Warning).count(), "warning")
        )
    }

    /// The text report: a `rewrote <path>` line for each fix, a line for
    /// each finding listed, then the count line.
    // sokf-implements: CLI-4_AC-1
    // sokf-implements: CLI-4_AC-9
    // sokf-implements: CLI-8_AC-2
    // sokf-implements: TL_92
    // sokf-implements: CLI-4_AC-10
    // sokf-implements: CLI-4_AC-11
    // sokf-implements: CLI-4_AC-12
    pub fn text(&self, warnings: bool) -> String {
        let mut out = String::new();
        for path in &self.fixes {
            out.push_str(&format!("rewrote {path}\n"));
        }
        for (path, f) in self.listed(warnings) {
            // sokf-implements: CLI-4_AC-3
            let severity = f.severity.as_str();
            // sokf-implements: CLI-4_AC-4
            let at = f.line.map_or_else(String::new, |l| format!(":{l}"));
            out.push_str(&format!(
                "{path}{at}: {severity}[{}]: {}\n",
                f.criterion, f.message
            ));
        }
        out.push_str(&self.counts());
        out
    }

    /// The report as one JSON object: `errors`, `warnings` and `fixes`.
    // sokf-implements: CLI-10_AC-1
    // sokf-implements: CLI-10_AC-2
    pub fn json(&self) -> Value {
        let findings = |severity: Severity| -> Vec<Value> {
            self.of(severity)
                .map(|(path, f)| {
                    let mut one = json!({ "path": path });
                    if let Some(line) = f.line {
                        one["line"] = json!(line);
                    }
                    one["key"] = json!(f.criterion);
                    one["message"] = json!(f.message);
                    one
                })
                .collect()
        };
        let fixes: Vec<Value> = self
            .fixes
            .iter()
            .map(|path| json!({ "op": "rewrote", "path": path }))
            .collect();
        json!({
            "errors": findings(Severity::Error),
            "warnings": findings(Severity::Warning),
            "fixes": fixes,
        })
    }

    /// The report as GitHub workflow commands: an `::error` or
    /// `::warning` command for each finding listed, then the fixes and the
    /// count line as plain lines of the log.
    // sokf-implements: CLI-10_AC-6
    pub fn github(&self, warnings: bool) -> String {
        let mut out = String::new();
        for (path, f) in self.listed(warnings) {
            let mut props = format!("file={}", property(path));
            if let Some(line) = f.line {
                props.push_str(&format!(",line={line}"));
            }
            props.push_str(&format!(",title={}", property(&f.criterion)));
            out.push_str(&format!(
                "::{} {props}::{}\n",
                f.severity.as_str(),
                data(&f.message)
            ));
        }
        for path in &self.fixes {
            out.push_str(&format!("rewrote {path}\n"));
        }
        out.push_str(&self.counts());
        out
    }
}

/// The message of a workflow command, escaped as GitHub reads it.
fn data(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// A property of a workflow command, escaped as GitHub reads it.
fn property(text: &str) -> String {
    data(text).replace(':', "%3A").replace(',', "%2C")
}

/// The JMESPath expression `expr`, compiled.
// sokf-implements: CLI-10_AC-5
pub fn compile(expr: &str) -> Result<jmespath::Expression<'static>, Stop> {
    jmespath::compile(expr).map_err(query_failed)
}

/// The stop of a query that does not compile or run.
fn query_failed(e: impl std::fmt::Display) -> Stop {
    Stop::plain(format!("query: {}", e.to_string().trim_end()))
}

/// The result of the compiled expression `expr` over `value`, as one line
/// of JSON.
pub fn query(expr: &jmespath::Expression<'_>, value: &Value) -> Result<String, Stop> {
    let result = expr.search(value).map_err(query_failed)?;
    let mut line = serde_json::to_string(&*result).map_err(|e| Stop::plain(e.to_string()))?;
    line.push('\n');
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tramlines_md::RepoPath;

    fn finding(line: Option<usize>, severity: Severity, message: &str) -> Finding {
        Finding {
            path: RepoPath::root(),
            line,
            severity,
            criterion: "TL_74".into(),
            message: message.into(),
            rule_line: None,
        }
    }

    fn report() -> Report {
        let mut r = Report::default();
        r.add(
            "b.md",
            [
                finding(Some(3), Severity::Warning, "a warning"),
                finding(Some(2), Severity::Error, "an error"),
            ],
        );
        r.add("a.md", [finding(None, Severity::Error, "no line")]);
        r.rewrote("b.md");
        r.finish();
        r
    }

    #[test]
    fn the_text_lists_the_fixes_the_errors_and_the_counts() {
        assert_eq!(
            report().text(false),
            "rewrote b.md\na.md: error[TL_74]: no line\nb.md:2: error[TL_74]: an error\n2 errors, 1 warning\n"
        );
    }

    #[test]
    fn warnings_are_listed_on_request() {
        assert!(
            report()
                .text(true)
                .contains("b.md:3: warning[TL_74]: a warning\n")
        );
    }

    #[test]
    fn the_json_holds_every_finding_and_fix() {
        assert_eq!(
            report().json(),
            json!({
                "errors": [
                    { "path": "a.md", "key": "TL_74", "message": "no line" },
                    { "path": "b.md", "line": 2, "key": "TL_74", "message": "an error" },
                ],
                "warnings": [
                    { "path": "b.md", "line": 3, "key": "TL_74", "message": "a warning" },
                ],
                "fixes": [{ "op": "rewrote", "path": "b.md" }],
            })
        );
    }

    #[test]
    fn workflow_commands_escape_their_text() {
        let mut r = Report::default();
        r.add(
            "a,b.md",
            [finding(Some(1), Severity::Error, "100% wrong\nhere")],
        );
        r.finish();
        assert_eq!(
            r.github(false),
            "::error file=a%2Cb.md,line=1,title=TL_74::100%25 wrong%0Ahere\n1 error, 0 warnings\n"
        );
    }

    #[test]
    fn a_query_projects_the_json() {
        let expr = compile("errors[].path").ok().unwrap();
        assert_eq!(
            query(&expr, &report().json()).ok(),
            Some("[\"a.md\",\"b.md\"]\n".to_string())
        );
        assert!(compile("errors[?").is_err());
    }
}
