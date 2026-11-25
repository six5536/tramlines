//! The frontmatter a schema implies: the host's leading keys, the declared
//! keys, the lifecycle keys and the host's trailing lines, each key with
//! the guidance its rule carries on a comment line before it.

use super::{Deriver, guide};
use crate::{
    Yaml,
    grammar::{FmFormat, FrontmatterConstraint, squash},
    markdown::Dialect,
    text, yaml,
};

/// A scalar written plain where it reads back as itself, else quoted.
fn scalar(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || " ._/-".contains(c))
        && !value.starts_with(' ')
        && !value.ends_with(' ');
    if plain {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

impl Deriver<'_> {
    /// The frontmatter block, fence to fence: the host's leading keys, the
    /// declared keys, the lifecycle keys, then the host's trailing lines.
    /// Nothing where the host's documents carry no frontmatter, or where
    /// there is nothing to write.
    // sokf-implements: TLMD_114
    // sokf-implements: MD-7_AC-3
    // sokf-implements: TLMD_120
    // sokf-implements: TLMD_132
    pub(super) fn frontmatter(&mut self) -> Vec<String> {
        // sokf-implements: MD-5_AC-8
        let Some(front) = self.host.scaffold_front(self.states) else {
            return Vec::new();
        };
        let mut out = vec!["---".to_string()];
        let mut written: Vec<String> = Vec::new();
        // sokf-implements: MD-5_AC-6
        for (key, seed) in &front.leading {
            // sokf-implements: MD-5_AC-7
            let constraint = self.schema.core.constraint(key);
            let given = self.given_value(key, constraint);
            let render = if given.is_some() {
                given_scalar
            } else {
                scalar
            };
            let Some(value) = self.leading_choice(key, constraint, given, seed.as_ref()) else {
                // sokf-implements: MD-17_AC-3
                if self.skeleton() && constraint.is_some_and(|c| !c.optional) {
                    out.push(format!("{key}:"));
                    written.push(key.clone());
                }
                continue;
            };
            self.guidance_of(key, constraint, &mut out);
            let line = format!("{key}: {}", render(&value));
            self.record_key(key, &line);
            out.push(format!("{line}{}", self.gate_of(constraint)));
            self.values.insert(key.clone(), value);
            written.push(key.clone());
        }
        let schema = self.schema;
        for (key, constraint) in &schema.core.frontmatter {
            // A value that the host gives is the document's, so it is
            // written whatever the gate of its key, as `tramlines new
            // --title` wrote it.
            let given = self.given_value(key, Some(constraint));
            if written.contains(key)
                || (given.is_none() && !constraint.common.only.admits(self.states))
            {
                continue;
            }
            let value = match &given {
                Some(v) => Some(v.clone()),
                None if self.skeleton() => (!constraint.optional)
                    .then(|| self.prescribed(constraint))
                    .flatten(),
                None => self.value_of(key, constraint),
            };
            let Some(value) = value else {
                // sokf-implements: MD-17_AC-3
                if self.skeleton() && !constraint.optional {
                    out.push(format!("{key}:"));
                    written.push(key.clone());
                }
                continue;
            };
            written.push(key.clone());
            self.guidance_of(key, Some(constraint), &mut out);
            let render = if given.is_some() {
                given_scalar
            } else {
                scalar
            };
            let rendered = if constraint.list {
                let one = render(&value);
                let n = if given.is_some() {
                    1
                } else {
                    constraint.min.unwrap_or(1).max(1)
                };
                format!("[{}]", vec![one; n].join(", "))
            } else {
                render(&value)
            };
            let line = format!("{key}: {rendered}");
            self.record_key(key, &line);
            out.push(format!("{line}{}", self.gate_of(Some(constraint))));
            self.values.insert(key.clone(), value);
        }
        for dim in &self.schema.core.lifecycle {
            let state = dim.state_in(self.states).cloned().unwrap_or_default();
            if let Some(g) = &dim.guidance
                && !self.plain()
            {
                out.push(guidance_comment(&self.dialect, g));
            }
            let line = format!("{}: {}", dim.key, scalar(&state));
            self.record_key(&dim.key, &line);
            out.push(line);
            self.values.insert(dim.key.clone(), state);
        }
        // sokf-implements: MD-5_AC-6
        out.extend(front.trailing);
        if out.len() == 1 {
            return Vec::new();
        }
        out.push("---".to_string());
        out
    }

    /// The value that a leading key of the host takes: the value that the
    /// host gives for the document, else, in a scaffold, the value that the
    /// schema fixes or the host's seed, and, in a skeleton, for a key that
    /// isn't optional, the value that the schema prescribes or the seed.
    fn leading_choice(
        &mut self,
        key: &str,
        constraint: Option<&FrontmatterConstraint>,
        given: Option<String>,
        seed: Option<&String>,
    ) -> Option<String> {
        if given.is_some() {
            return given;
        }
        if !self.skeleton() {
            return self
                .leading_value(key, constraint)
                .or_else(|| seed.cloned());
        }
        // A skeleton writes an optional key only where the host gives the
        // document its value.
        if constraint.is_some_and(|c| c.optional) {
            return None;
        }
        constraint
            .and_then(|c| self.prescribed(c))
            .or_else(|| seed.cloned())
    }

    /// The value of `key` that the host gives for the document, or the
    /// `const` of the key where the schema fixes it, which no document can
    /// change, or `None` where the host gives none.
    // sokf-implements: MD-7_AC-18
    // sokf-implements: MD-17_AC-3
    // sokf-implements: TLMD_301
    // sokf-implements: TLMD_304
    fn given_value(&self, key: &str, constraint: Option<&FrontmatterConstraint>) -> Option<String> {
        let given = self.given.get(key).filter(|v| !v.is_empty())?;
        // A `const` wins over the host's value, which no document changes.
        Some(
            constraint
                .and_then(|c| c.const_.clone())
                .unwrap_or_else(|| given.clone()),
        )
    }

    /// The value that the schema prescribes for a key of a skeleton, which
    /// no example states: its `const`, or the `id` that its template renders
    /// from the states alone, with no `slug` hole and no `seq` hole.
    // sokf-implements: MD-17_AC-3
    // sokf-implements: TLMD_304
    fn prescribed(&self, c: &FrontmatterConstraint) -> Option<String> {
        if let Some(v) = &c.const_ {
            return Some(v.clone());
        }
        let t = c.id_template.as_ref()?;
        if t.has_slug() || t.seq_width().is_some() {
            return None;
        }
        t.render_with(1, None, self.states).ok()
    }

    /// Record `line`, the line that the derivation writes for `key`, where it
    /// records examples.
    fn record_key(&mut self, key: &str, line: &str) {
        if let Some(ex) = &mut self.examples {
            ex.keys
                .entry(key.to_string())
                .or_insert_with(|| line.to_string());
        }
    }

    /// The `guidance` comment line a declared key's guidance takes, on
    /// the line before the key, so it never shares a line with a trailing
    /// `only` marker.
    // sokf-implements: TLMD_120
    // sokf-implements: TLMD_232
    // sokf-implements: MD-7_AC-14
    fn guidance_of(
        &self,
        key: &str,
        constraint: Option<&FrontmatterConstraint>,
        out: &mut Vec<String>,
    ) {
        if self.plain() {
            return;
        }
        let language = self.schema.core.language.as_deref();
        if let Some(g) = constraint.and_then(|c| guide::key_text(self.host, language, key, c)) {
            out.push(guidance_comment(&self.dialect, &g));
        }
    }

    /// The `only` comment a gated frontmatter key carries, if any.
    // sokf-implements: TLMD_121
    fn gate_of(&self, constraint: Option<&FrontmatterConstraint>) -> String {
        constraint
            .and_then(|c| self.gate_text(&c.common))
            .map(|gate| format!(" # {gate}"))
            .unwrap_or_default()
    }

    /// The value a leading key takes where the schema fixes one, as
    /// `FrontmatterConstraint::fixed_value` says. A key narrowed by a shape
    /// and offering no example is a refusal, as any other narrowed key is.
    fn leading_value(
        &mut self,
        key: &str,
        constraint: Option<&FrontmatterConstraint>,
    ) -> Option<String> {
        let c = constraint?;
        if let Some(v) = c.fixed_value(self.states) {
            return Some(v);
        }
        if c.shape.is_some() {
            let line = c.common.line;
            self.refuse(line, format!("`{key}` declares a shape and no `example`"));
        }
        None
    }

    /// The value one declared key takes, or `None` where the key is
    /// optional and nothing states what to write.
    fn value_of(&mut self, key: &str, c: &FrontmatterConstraint) -> Option<String> {
        // The `id` a scaffold writes holds the states of that scaffold, so
        // it comes from the template, as a host's leading `id` does.
        // sokf-implements: TL_59
        if let Some(v) = c.scaffold_value(self.states) {
            return Some(v);
        }
        // Nothing else fixes the value, so the scaffold writes none of its
        // own: an optional key is left out, a required one is refused.
        // sokf-implements: TLMD_115
        if c.optional {
            return None;
        }
        let what = match (&c.format, c.shape.is_some()) {
            (Some(FmFormat::Host(name)), _) => {
                format!(
                    "`{key}` is {} `{name}`, and no `example` says which one",
                    text::article(name)
                )
            }
            (Some(FmFormat::Url), _) => {
                format!("`{key}` names a URL, and no `example` says which one")
            }
            (_, true) => format!("`{key}` declares a shape and no `example`"),
            _ => format!("`{key}` is required and has no `example`; write the value it demands"),
        };
        self.refuse(c.common.line, what);
        None
    }
}

/// A value that the host gives for the document, written plain only where
/// YAML reads it back as the same text, so that a title such as `2026`,
/// `true` or `null` stays a text and never reads as no value.
fn given_scalar(value: &str) -> String {
    let text = yaml::parse(&format!("k: {value}\n"), 1)
        .ok()
        .and_then(|y| y.get("k").map(|v| v.value.clone()));
    if !value.contains('#') && text == Some(Yaml::Str(value.to_string())) {
        scalar(value)
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// One `guidance` YAML comment line, the text joined onto that line.
fn guidance_comment(dialect: &Dialect, text: &str) -> String {
    format!("# {} {}", dialect.spell("guidance"), squash(text))
}
