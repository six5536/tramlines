//! A compiled schema held to itself: the example of each rule, the
//! scaffold of each state combination, the `only` markers of each
//! scaffold, and the complete examples.

use super::{CheckOptions, Gate, Host, check_against, example, short};
use crate::{
    Finding, Locator, RepoPath, Severity,
    derive::derive,
    document::Document,
    grammar::{Block, Only, RuleCommon, Schema, States, squash},
    markdown::Dialect,
};

/// Every finding a compiled schema raises on itself, reported on `path`;
/// `id` names the schema in the findings of its scaffolds and examples.
/// A scaffold that fails its schema fails the schema; so does an example.
// sokf-implements: TL_64
// sokf-implements: TL_69
// sokf-implements: TLMD_114
// sokf-implements: TLMD_116
// sokf-implements: TLMD_124
// sokf-implements: TLMD_164
// sokf:begin check-schema
pub fn check_schema(schema: &Schema, host: &dyn Host, path: &RepoPath, id: &str) -> Vec<Finding> {
    // sokf:end check-schema
    // sokf-implements: MD-5_AC-1
    let dialect = host.dialect();
    // sokf-implements: MD-6_AC-1
    // sokf-implements: MD-6_AC-7
    let (mut findings, failed) = example::check_examples(schema, path, id);
    // sokf-implements: MD-6_AC-2
    let scaffolds = match derive(schema, host) {
        Ok(derived) => {
            // An optional part the derivation cannot write is left out, and
            // the author is told which rule kept it out.
            // sokf-implements: MD-6_AC-4
            for r in &derived.omitted {
                findings.push(Finding::warning(
                    path.clone(),
                    Some(r.line),
                    "TLMD_124",
                    format!("an optional part is left out of the scaffold: {}", r.what),
                ));
            }
            derived.scaffolds
        }
        Err(refusals) => {
            findings.extend(refusals.into_iter().map(|r| {
                Finding::error(
                    path.clone(),
                    Some(r.line),
                    r.criterion,
                    format!("the schema derives no scaffold: {}", r.what),
                )
            }));
            Vec::new()
        }
    };
    // A marker that gates nothing names the rule it was written for where
    // one rule alone carries its `only`.
    let gated = gated_rules(schema);
    for scaffold in &scaffolds {
        let label = match scaffold.lifecycle.as_ref() {
            Some(only) if only.is_gated() => {
                format!("the scaffold derived for `{}`", only.render())
            }
            _ => "the scaffold".to_string(),
        };
        let states = scaffold_states(scaffold.lifecycle.as_ref(), schema);
        let parsed = Document::parse(path.clone(), &scaffold.document, &dialect);
        // sokf-implements: MD-6_AC-9
        let out = check_against(
            parsed.view(),
            schema,
            &states,
            host,
            CheckOptions { scaffold: true },
            Locator::schema(id),
        );
        // The scaffold writes each example, so a line that holds one that
        // failed, and a failure of the rule whose example failed, are the
        // fault TL_64 or TL_65 reports.
        let lines: Vec<&str> = scaffold.document.lines().collect();
        let line_of = |f: &Finding| f.line.and_then(|l| lines.get(l.checked_sub(1)?).copied());
        // A finding with no line of its own, such as a section missing
        // because its heading example failed, is matched by its rule.
        let holds_failed = |f: &Finding| {
            line_of(f).is_some_and(|l| failed.texts.iter().any(|e| writes(l, e)))
                || (f.line.is_none() && f.rule_line.is_some_and(|l| failed.rules.contains(&l)))
        };
        // sokf-implements: MD-6_AC-5
        // sokf-implements: MD-6_AC-6
        for f in out
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error && !holds_failed(f))
        {
            // The finding sits on the rule that the scaffold fails, and
            // quotes the scaffold's line, which the reader cannot see.
            let at = line_of(f)
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| format!(" at `{}`", short(l)))
                .unwrap_or_default();
            findings.push(Finding::error(
                path.clone(),
                Some(f.rule_line.unwrap_or(scaffold.line)),
                "TLMD_164",
                format!("{label} fails{at}: [{}] {}", f.criterion, f.message),
            ));
        }
        // sokf-implements: MD-6_AC-3
        // sokf-implements: MD-6_AC-9
        for (rule, problem) in check_markers(&dialect, &scaffold.document, &out.gates, &gated) {
            findings.push(Finding::error(
                path.clone(),
                Some(rule.filter(|l| *l > 0).unwrap_or(scaffold.line)),
                "TLMD_122",
                format!("{label}: {problem}"),
            ));
        }
    }
    // An example is a governed document: its own states decide, and a
    // marker in it is an error.
    for (n, example) in schema.core.examples.iter().enumerate() {
        let parsed = Document::parse(path.clone(), &example.document, &dialect);
        let out = parsed.check(schema, host, Locator::schema(id));
        for f in out
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
        {
            findings.push(Finding::error(
                path.clone(),
                Some(example.line),
                "TL_69",
                format!("example {} fails: {}", n + 1, inner(f)),
            ));
        }
    }
    findings
}

/// A finding of a scaffold or an example, as the finding on the schema
/// quotes it: `line N: [KEY] message`.
fn inner(f: &Finding) -> String {
    let at = f.line.map(|l| format!("line {l}: ")).unwrap_or_default();
    format!("{at}[{}] {}", f.criterion, f.message)
}

/// The line of each `only` that one rule of the schema alone carries.
fn gated_rules(schema: &Schema) -> Vec<(Only, usize)> {
    let mut lines: Vec<(Only, usize)> = Vec::new();
    let mut add = |c: &RuleCommon| {
        if c.only.is_gated() {
            lines.push((c.only.clone(), c.line));
        }
    };
    fn blocks(rules: Option<&[Block]>, add: &mut impl FnMut(&RuleCommon)) {
        for b in rules.unwrap_or(&[]) {
            add(&b.common);
            blocks(b.blocks.as_deref(), add);
            for u in b.units() {
                add(&u.common);
                for line in u.lines.iter().flatten() {
                    add(&line.common);
                }
                blocks(u.blocks.as_deref(), add);
            }
        }
    }
    for (_, c) in &schema.core.frontmatter {
        add(&c.common);
    }
    if let Some(p) = &schema.preamble {
        add(&p.common);
        blocks(p.blocks.as_deref(), &mut add);
    }
    for s in &schema.sections {
        add(&s.common);
        blocks(s.blocks.as_deref(), &mut add);
    }
    lines
        .iter()
        .filter(|(only, _)| lines.iter().filter(|(o, _)| o == only).count() == 1)
        .cloned()
        .collect()
}

/// Whether the scaffold line `line` is the example `example` as the
/// scaffold writes it: the text alone, or after the marker of its part, a
/// heading's hashes, an item's bullet or number, a quote's `>`, or a
/// frontmatter key.
fn writes(line: &str, example: &str) -> bool {
    let line = squash(line);
    if line == example {
        return true;
    }
    let Some(lead) = line.strip_suffix(example).and_then(|l| l.strip_suffix(' ')) else {
        return false;
    };
    let marker = |m: &str| {
        m.chars().all(|c| c == '#')
            || matches!(m, "-" | "*" | "+" | ">")
            || m.strip_suffix(['.', ')'])
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            || m.strip_suffix(':')
                .is_some_and(|k| !k.is_empty() && !k.contains(' '))
    };
    !lead.is_empty() && lead.split(' ').all(marker)
}

/// The states a scaffold stands for: its `lifecycle`, absent meaning every
/// state of every dimension.
fn scaffold_states(lifecycle: Option<&Only>, schema: &Schema) -> States {
    let mut states = States::default();
    for dim in &schema.core.lifecycle {
        let admitted = lifecycle
            .and_then(|o| o.0.get(&dim.key).cloned())
            .unwrap_or_else(|| dim.states.clone());
        states.0.insert(dim.key.clone(), admitted);
    }
    states
}

/// The line a marker marks, one-based: an HTML comment marks the next line
/// with content; a YAML comment marks its own.
fn marked_line(dialect: &Dialect, lines: &[&str], i: usize) -> Option<usize> {
    if lines[i].trim_start().starts_with("<!--") {
        lines[i + 1..]
            .iter()
            .position(|l| {
                !l.trim().is_empty()
                    && !dialect.is_only_marker(l)
                    && !dialect.is_optional_mark(l)
                    && !dialect.is_guidance_marker(l)
            })
            .map(|j| i + 1 + j + 1)
    } else {
        Some(i + 1)
    }
}

/// The `only` markers of a scaffold and the gates of the rules that matched
/// its parts, each read against the other. A marker passes where a gate on
/// the line it marks agrees with it, and a gate passes where a marker on
/// its line agrees with it. Each problem names the part by its text, and
/// the line of the rule in the schema where one is known.
// sokf-implements: TLMD_122
fn check_markers(
    dialect: &Dialect,
    text: &str,
    gates: &[Gate],
    gated: &[(Only, usize)],
) -> Vec<(Option<usize>, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let part = |line: usize| {
        let text = lines.get(line.wrapping_sub(1)).map_or("", |l| l.trim());
        format!("`{}`", short(text))
    };
    let only = dialect.spell("only");
    let mut out = Vec::new();
    let mut marked: Vec<(usize, Only)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !dialect.is_only_marker(line) {
            continue;
        }
        let target = marked_line(dialect, &lines, i);
        let at: Vec<&Gate> = target
            .map(|p| gates.iter().filter(|g| g.line == p).collect())
            .unwrap_or_default();
        // A YAML comment marks its own line; an HTML one the part after it.
        let marker = match target {
            Some(p) if p != i + 1 => format!("the `{only}` marker before {}", part(p)),
            _ => format!("the `{only}` marker on {}", part(i + 1)),
        };
        // A marker the engine did not write, as one inside an example, may
        // not read.
        let Some(m) = dialect.only_marker(line) else {
            out.push((
                at.first().map(|g| g.rule),
                format!(
                    "the `{only}` marker `{}` is malformed; write `{only} <key>=<state>[,<state>]…`",
                    short(line.trim())
                ),
            ));
            continue;
        };
        if let Some(p) = target {
            marked.push((p, Only(m.clone())));
        }
        let problem = if at.is_empty() {
            "gates nothing: the part it marks is not matched by a gated rule".to_string()
        } else if at.iter().any(|g| g.only.0 == m) {
            continue;
        } else {
            format!(
                "disagrees with its rule, whose `only` is `{}`",
                at[0].only.render()
            )
        };
        let rule = at
            .first()
            .map(|g| g.rule)
            .or_else(|| gated.iter().find(|(o, _)| o.0 == m).map(|(_, l)| *l));
        out.push((rule, format!("{marker} {problem}")));
    }
    let mut said: Vec<(usize, &Only)> = Vec::new();
    for gate in gates {
        if marked
            .iter()
            .any(|(p, m)| *p == gate.line && *m == gate.only)
            || said
                .iter()
                .any(|(l, o)| *l == gate.line && **o == gate.only)
        {
            continue;
        }
        said.push((gate.line, &gate.only));
        out.push((
            Some(gate.rule),
            format!(
                "the part {} is matched by a rule whose `only` is `{}` and carries no `{only}` marker",
                part(gate.line),
                gate.only.render()
            ),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::prelude::*;

    use super::*;

    type Mark = Option<&'static str>;

    fn only(state: &str) -> Only {
        Only(BTreeMap::from([(
            "state".to_string(),
            vec![state.to_string()],
        )]))
    }

    /// A text of sections, each gated or not and marked or not, with the
    /// gates the rules that matched them would carry and what the check
    /// must say about each.
    fn text_and_expected(sections: &[(Mark, Mark)]) -> (String, Vec<Gate>, Vec<String>) {
        let mut lines: Vec<String> = vec!["# Example".into(), String::new()];
        let mut gates = Vec::new();
        let mut expected = Vec::new();
        let mut unmarked = Vec::new();
        for (i, (gate, mark)) in sections.iter().enumerate() {
            if let Some(m) = mark {
                lines.push(format!("<!-- tramlines:only state={m} -->"));
                let heading = format!("`## S{i}`");
                match gate {
                    None => expected.push(format!(
                        "the `tramlines:only` marker before {heading} gates nothing: the part it marks is not matched by a gated rule"
                    )),
                    Some(g) if g != m => expected.push(format!(
                        "the `tramlines:only` marker before {heading} disagrees with its rule, whose `only` is `state={g}`"
                    )),
                    Some(_) => {}
                }
            }
            lines.push(format!("## S{i}"));
            if let Some(g) = gate {
                gates.push(Gate {
                    line: lines.len(),
                    only: only(g),
                    rule: i + 1,
                });
                if mark.as_ref() != Some(g) {
                    unmarked.push(format!(
                        "the part `## S{i}` is matched by a rule whose `only` is `state={g}` and carries no `tramlines:only` marker"
                    ));
                }
            }
            lines.push(String::new());
        }
        expected.extend(unmarked);
        (lines.join("\n"), gates, expected)
    }

    fn arb_section() -> impl Strategy<Value = (Mark, Mark)> {
        let state = prop::sample::select(vec![None, Some("a"), Some("b")]);
        (state.clone(), state)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Marker and gate are read against each other, both ways: the check
        /// reports exactly the markers no gate on their line agrees with and
        /// the gates no marker on their line agrees with.
        // sokf-tests: TLMD_122
        #[test]
        fn a_marker_and_its_gate_are_each_read_against_the_other(
            sections in prop::collection::vec(arb_section(), 1..5)
        ) {
            let (text, gates, mut expected) = text_and_expected(&sections);
            let mut reported: Vec<String> = check_markers(&Dialect::default(), &text, &gates, &[])
                .into_iter()
                .map(|(_, problem)| problem)
                .collect();
            reported.sort();
            expected.sort();
            prop_assert_eq!(reported, expected, "{}", text);
        }
    }
}
