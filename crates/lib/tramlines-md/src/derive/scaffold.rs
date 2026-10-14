//! The scaffold a schema implies, which is the only template of a
//! document it has.
//!
//! The scaffold is checked, never trusted: it goes through the
//! path that checks an authored one, so a generator that writes what the
//! schema refuses fails the schema in the store that owns the schema.
//! The failure mode is a false alarm and never a silent hole.
//!
//! No text is invented. A part that no `text`, no `example` and no bound
//! format supplies is a refusal naming the rule that demands it: text that
//! satisfies the checker and teaches nothing is worse than an error, and
//! the agent that reads a scaffold imitates what it is given.

use std::collections::BTreeMap;

use super::{Deriver, deriver::push_unique};
use crate::{
    RepoPath,
    check::Host,
    grammar::{Only, Scaffold, Schema, States},
    markdown::Dialect,
};

// sokf:begin derive
/// A rule the derivation cannot satisfy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The rule's line in the schema.
    pub line: usize,
    /// The key of the criterion that refuses the derivation: `TLMD_116`
    /// for a part with no text, `TLMD_154` for an example that does not
    /// carry the key spliced into it, `TLMD_155` for a frontmatter key that
    /// the scaffold does not write, `TL_97` for a format that resolves to
    /// nothing, and `TLMD_149` for a group whose repetitions read as one.
    pub criterion: &'static str,
    /// What the rule asked for that the derivation cannot write.
    pub what: String,
}
// sokf:end derive

/// What to tell an author whose schema derives no scaffold, one refusal
/// per line at the line of the rule that raised it. A joined sentence is
/// unreadable: each refusal names a rule of its own, and several of them
/// carry a clause apiece.
pub fn no_template(type_name: &str, schema: Option<&RepoPath>, refusals: &[Refusal]) -> String {
    let mut out = format!("the schema of `{type_name}` derives no scaffold:");
    for r in refusals {
        match schema {
            Some(path) => out.push_str(&format!("\n  {path}:{}: {}", r.line, r.what)),
            None => out.push_str(&format!("\n  line {}: {}", r.line, r.what)),
        }
    }
    out.push_str("\nwrite the `example` each rule asks for");
    out
}

// sokf:begin derive
/// What a schema derives: a scaffold per state combination, and the
/// optional parts no rule gave the derivation anything to write.
pub struct Derived {
    /// The scaffolds, in lifecycle order.
    pub scaffolds: Vec<Scaffold>,
    /// The optional parts left out, and the rule that kept each out.
    pub omitted: Vec<Refusal>,
}
// sokf:end derive

/// One scaffold per state combination, in lifecycle order, each standing
/// for exactly one state of every dimension.
// sokf-implements: TLMD_114
// sokf:begin derive
pub fn derive(schema: &Schema, host: &dyn Host) -> std::result::Result<Derived, Vec<Refusal>> {
    // sokf:end derive
    let mut derived = Derived {
        scaffolds: Vec::new(),
        omitted: Vec::new(),
    };
    let mut refusals = Vec::new();
    // One rule speaks once, however many states it stands in.
    // sokf-implements: MD-7_AC-8
    // sokf-implements: MD-7_AC-1
    for states in combinations(schema) {
        let (result, omitted) = one(schema, &states, host);
        push_unique(&mut derived.omitted, omitted);
        match result {
            // sokf-implements: MD-7_AC-9
            Ok(document) => derived.scaffolds.push(Scaffold {
                document,
                lifecycle: (!states.0.is_empty()).then(|| Only(states.0.clone())),
                line: schema.core.line,
            }),
            Err(r) => push_unique(&mut refusals, r),
        }
    }
    if refusals.is_empty() {
        Ok(derived)
    } else {
        Err(refusals)
    }
}

/// The document at one state combination.
// sokf:begin derive
pub fn derive_one(
    schema: &Schema,
    states: &States,
    host: &dyn Host,
) -> std::result::Result<String, Vec<Refusal>> {
    // sokf:end derive
    one(schema, states, host).0
}

/// The document at one state combination, and the optional parts left out.
fn one(
    schema: &Schema,
    states: &States,
    host: &dyn Host,
) -> (std::result::Result<String, Vec<Refusal>>, Vec<Refusal>) {
    // sokf-implements: MD-5_AC-1
    let dialect = host.dialect();
    let mut d = Deriver {
        schema,
        host,
        dialect,
        states,
        keys: BTreeMap::new(),
        kinds: Vec::new(),
        values: BTreeMap::new(),
        section_rule: None,
        refusals: Vec::new(),
        omitted: Vec::new(),
        in_optional: false,
        scope: None,
    };
    let mut lines = d.frontmatter();
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.extend(d.body());
    if !d.refusals.is_empty() {
        return (Err(d.refusals), d.omitted);
    }
    merge_optional(&mut lines, &dialect);
    let mut text = lines.join("\n");
    // sokf-implements: MD-7_AC-10
    while text.ends_with("\n\n") {
        text.pop();
    }
    if !text.ends_with('\n') {
        text.push('\n');
    }
    (Ok(text), d.omitted)
}

/// Two optional regions that meet, with only blank lines between them,
/// become one, unless the second opens a section. A close followed by an
/// open marks one decision twice, but each optional section is a decision of
/// its own.
// sokf-implements: MD-7_AC-4
// sokf-implements: TLMD_125
fn merge_optional(lines: &mut Vec<String>, dialect: &Dialect) {
    let mut i = 0;
    while i < lines.len() {
        if !dialect.is_optional_close(&lines[i]) {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < lines.len() && lines[j].trim().is_empty() {
            j += 1;
        }
        if j < lines.len()
            && dialect.is_optional_marker(&lines[j])
            && !opens_section(&lines[j + 1..], dialect)
        {
            lines.splice(i..=j, [String::new()]);
            // Leave one blank line between the two parts.
            while i > 0 && lines[i - 1].trim().is_empty() {
                lines.remove(i - 1);
                i -= 1;
            }
            while i + 1 < lines.len() && lines[i + 1].trim().is_empty() {
                lines.remove(i + 1);
            }
            continue;
        }
        i += 1;
    }
}

/// Whether a region whose lines follow its opening marker opens a section:
/// whether its first line, past blank lines and guidance, is a heading.
fn opens_section(rest: &[String], dialect: &Dialect) -> bool {
    rest.iter()
        .flat_map(|chunk| chunk.lines())
        .find(|l| !l.trim().is_empty() && !dialect.is_guidance_marker(l))
        .is_some_and(|l| l.trim_start().starts_with('#'))
}

/// One state of every dimension, every way round. A schema with no
/// lifecycle has one combination, which names nothing.
fn combinations(schema: &Schema) -> Vec<States> {
    let mut out = vec![States::default()];
    for dim in &schema.core.lifecycle {
        let mut next = Vec::new();
        for base in &out {
            for state in &dim.states {
                let mut one = base.clone();
                one.0.insert(dim.key.clone(), vec![state.clone()]);
                next.push(one);
            }
        }
        out = next;
    }
    out
}
/// The words of a slug, space-joined, the first letter upper-cased.
pub fn derived_title(slug: &str) -> String {
    let words = slug.split('-').filter(|w| !w.is_empty());
    let joined = words.collect::<Vec<_>>().join(" ");
    let mut chars = joined.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
