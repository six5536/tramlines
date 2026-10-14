//! The frontmatter a schema implies: the host's leading keys, the declared
//! keys, the lifecycle keys and the host's trailing lines, each key with
//! the guidance its rule carries on a comment line before it.

use super::Deriver;
use crate::{
    grammar::{FmFormat, FrontmatterConstraint, squash},
    markdown::Dialect,
    text,
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
            let Some(value) = self.leading_value(key, constraint).or_else(|| seed.clone()) else {
                continue;
            };
            self.guidance_of(constraint, &mut out);
            out.push(format!(
                "{key}: {}{}",
                scalar(&value),
                self.gate_of(constraint)
            ));
            self.values.insert(key.clone(), value);
            written.push(key.clone());
        }
        let schema = self.schema;
        for (key, constraint) in &schema.core.frontmatter {
            if written.contains(key) || !constraint.common.only.admits(self.states) {
                continue;
            }
            let Some(value) = self.value_of(key, constraint) else {
                continue;
            };
            written.push(key.clone());
            self.guidance_of(Some(constraint), &mut out);
            let rendered = if constraint.list {
                let one = scalar(&value);
                let items = vec![one; constraint.min.unwrap_or(1).max(1)];
                format!("[{}]", items.join(", "))
            } else {
                scalar(&value)
            };
            out.push(format!(
                "{key}: {rendered}{}",
                self.gate_of(Some(constraint))
            ));
            self.values.insert(key.clone(), value);
        }
        for dim in &self.schema.core.lifecycle {
            let state = self
                .states
                .0
                .get(&dim.key)
                .and_then(|s| s.first())
                .cloned()
                .or_else(|| dim.states.first().cloned())
                .unwrap_or_default();
            if let Some(g) = &dim.guidance {
                out.push(guidance_comment(&self.dialect, g));
            }
            out.push(format!("{}: {}", dim.key, scalar(&state)));
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

    /// The `guidance` comment line a declared key's guidance takes, on
    /// the line before the key, so it never shares a line with a trailing
    /// `only` marker.
    // sokf-implements: TLMD_120
    fn guidance_of(&self, constraint: Option<&FrontmatterConstraint>, out: &mut Vec<String>) {
        if let Some(g) = constraint.and_then(|c| c.guidance.as_ref()) {
            out.push(guidance_comment(&self.dialect, g));
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

    /// The value a leading key takes where the schema fixes one: its
    /// `const`, the `id` template rendered with its sequence at one and the
    /// slug `example`, or its `example`. A key narrowed by a shape and
    /// offering no example is a refusal, as any other narrowed key is.
    fn leading_value(
        &mut self,
        key: &str,
        constraint: Option<&FrontmatterConstraint>,
    ) -> Option<String> {
        let c = constraint?;
        if let Some(v) = &c.const_ {
            return Some(v.clone());
        }
        if let Some(t) = &c.id_template {
            return t
                .render_with(1, t.has_slug().then_some("example"), self.states)
                .ok();
        }
        if let Some(v) = &c.example {
            return Some(v.clone());
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
        if let Some(v) = &c.const_ {
            return Some(v.clone());
        }
        // The `id` a scaffold writes holds the states of that scaffold, so
        // it comes from the template, as a host's leading `id` does.
        // sokf-implements: TL_59
        if let Some(t) = &c.id_template
            && let Ok(v) = t.render_with(1, t.has_slug().then_some("example"), self.states)
        {
            return Some(v);
        }
        if let Some(v) = &c.example {
            return Some(v.clone());
        }
        if let Some(v) = c.enum_.as_ref().and_then(|e| e.first()) {
            return Some(v.clone());
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

/// One `guidance` YAML comment line, the text joined onto that line.
fn guidance_comment(dialect: &Dialect, text: &str) -> String {
    format!("# {} {}", dialect.spell("guidance"), squash(text))
}
