//! Where a derived unit's text comes from, and the splices that make one
//! key agree with another.

use std::collections::BTreeMap;

use regex::Regex;

use super::{Deriver, deriver::push_unique};
use crate::{
    check,
    grammar::{Block, DerivationSource, KeyDerivation, Shape, Text, Unit, squash},
    text,
};

/// How a key is read out of a text: the regex, and the name of its group.
#[derive(Debug, Clone)]
pub(super) struct KeyReader {
    /// The regex of the shape that holds the key.
    pub regex: Regex,
    /// The group's name.
    pub name: String,
}

impl KeyReader {
    /// The key of `shape` named `name`, where the shape has a regex.
    pub(super) fn of(shape: &Shape, name: &str) -> Option<KeyReader> {
        Some(KeyReader {
            regex: shape.regex()?.clone(),
            name: name.to_string(),
        })
    }
}

impl Deriver<'_> {
    /// The texts that `block` offers for its units under `rule`, and whether
    /// the schema writes them as a list: the rule's `text`, then its
    /// `example`, then the bound format's, then the block's. A part none of
    /// these supplies is a refusal, constrained or not: the scaffold writes
    /// no text of its own, because text that validates and was never meant
    /// reads as finished to the model that fills it.
    // sokf-implements: MD-7_AC-2
    // sokf-implements: TLMD_117
    // sokf-implements: TLMD_116
    // sokf-implements: TLMD_115
    pub(super) fn unit_texts(&mut self, block: &Block, rule: Option<&Unit>) -> (Vec<String>, bool) {
        let line = rule.map_or(block.common.line, |r| r.common.line);
        if let Some(r) = rule {
            match &r.shape {
                Some(Shape::Text(Text::Literal(t))) => return (vec![t.clone()], false),
                Some(Shape::Text(Text::From(key))) => {
                    // sokf-implements: TLMD_155
                    let value = self.values.get(key).cloned().unwrap_or_else(|| {
                        self.refuse_on(
                            line,
                            "TLMD_155",
                            format!("a unit rule takes its text from `{key}`, which no frontmatter writes"),
                        );
                        String::new()
                    });
                    return (vec![value], false);
                }
                _ => {}
            }
            if !r.examples.is_empty() {
                return (r.examples.clone(), r.example_list);
            }
            if let Some(name) = &r.format {
                if let Some(format) = self.schema.core.formats.get(name) {
                    return (vec![format.first_example().to_string()], false);
                }
                // Compiling refuses a format that no layer defines first, so
                // only a schema built by hand reaches this.
                self.refuse_on(
                    line,
                    "TL_97",
                    format!("a unit rule binds `{name}`, which resolves to no format"),
                );
                return (vec![String::new()], false);
            }
        }
        if !block.examples.is_empty() {
            return (block.examples.clone(), block.example_list);
        }
        let constrained = rule.is_some_and(|r| r.has_shape() || r.key.is_some());
        let what = if constrained {
            "a unit rule declares a shape and no `example`; give the rule an `example`".to_string()
        } else {
            format!(
                "{} `{}` block has no `example`, and the scaffold writes no text of its own; give the block an `example`",
                text::article(block.kind.name_in(&self.dialect)),
                block.kind.name_in(&self.dialect)
            )
        };
        self.refuse(line, what);
        (vec![String::new()], false)
    }

    /// The text of the unit at position `i` among the units that `rule`
    /// writes: the `i`th text that `unit_texts` returns, or the last where
    /// the list is shorter, so that a `min` above the length of the list
    /// repeats the last text.
    pub(super) fn unit_text_at(&mut self, block: &Block, rule: Option<&Unit>, i: usize) -> String {
        let (mut texts, _) = self.unit_texts(block, rule);
        let at = i.min(texts.len() - 1);
        texts.swap_remove(at)
    }

    /// How `rule` reads a key out of `text`: its own shape and `key`, or
    /// the shape of its format that `text` takes and the format's key.
    pub(super) fn key_reader(&self, rule: &Unit, text: &str) -> Option<KeyReader> {
        if let Some(shape) = &rule.shape {
            return KeyReader::of(shape, rule.key.as_deref()?);
        }
        let format = self.schema.core.formats.get(rule.format.as_ref()?)?;
        let name = rule.key.as_ref().or(format.key.as_ref())?;
        let (shape, _) = format.shape_for(&squash(text))?;
        KeyReader::of(shape, name)
    }

    /// The key `rule` would capture from `text`.
    pub(super) fn captured(&self, rule: &Unit, text: &str) -> Option<String> {
        capture_of(&self.key_reader(rule, text)?, text)
    }

    /// Record the key `rule` captures from `text`, against the rule's
    /// `rule-id`, else the block's.
    pub(super) fn record_capture(&mut self, block: &Block, rule: &Unit, text: &str) {
        let key = self.captured(rule, text);
        self.record(rule.rule_id.as_ref().or(block.rule_id.as_ref()), key);
    }

    /// A key whose rule derives it takes a prefix the rule it derives from
    /// captured: the rule's own example carries a prefix of its author's
    /// choosing, and the derived document has only the keys the derivation
    /// itself wrote.
    // sokf-implements: TLMD_114
    pub(super) fn derived_key(&mut self, rule: &Unit, text: String) -> String {
        let Some(kd) = &rule.key_derives_from else {
            return text;
        };
        let Some(reader) = self.key_reader(rule, &text) else {
            return text;
        };
        self.derive_with(kd, &reader, text, rule.common.line)
    }

    /// `text`, respelled so the key `reader` reads from it takes a prefix
    /// `kd` allows: the first key its rule captured, or the value written
    /// for its frontmatter key.
    pub(super) fn derive_with(
        &mut self,
        kd: &KeyDerivation,
        reader: &KeyReader,
        text: String,
        line: usize,
    ) -> String {
        let Some(key) = capture_of(reader, &text) else {
            return text;
        };
        let split = key.split_once(&kd.separator);
        let parents = match &kd.source {
            DerivationSource::Rule(r) => self.keys_of(r),
            DerivationSource::From(k) => self.values.get(k).cloned().into_iter().collect(),
        };
        if split.is_some_and(|(p, _)| parents.iter().any(|q| q == p)) {
            return text;
        }
        let Some(parent) = parents.first() else {
            return text;
        };
        let tail = split.map_or("1", |(_, t)| t);
        let want = format!("{parent}{}{tail}", kd.separator);
        match respell(reader, &text, &want) {
            Some(spliced) => spliced,
            // sokf-implements: TLMD_154
            None => {
                self.refuse_on(
                    line,
                    "TLMD_154",
                    format!(
                        "a key deriving from {} cannot be written: `{key}` does not read as `{want}` where `{}` takes it",
                        kd.source_label(),
                        reader.name
                    ),
                );
                text
            }
        }
    }

    /// Record `key` against the block's `rule-id`, else the enclosing
    /// section's, which is how the checker attributes a capture.
    pub(super) fn record(&mut self, rule_id: Option<&String>, key: Option<String>) {
        let id = rule_id.or(self.section_rule.as_ref());
        if let (Some(id), Some(key)) = (id, key) {
            self.keys.entry(id.clone()).or_default().push(key);
        }
    }

    /// The keys a rule captured, in the order it captured them.
    pub(super) fn keys_of(&self, rule_id: &str) -> Vec<String> {
        self.keys.get(rule_id).cloned().unwrap_or_default()
    }

    /// The keys a pairing reads: those that its rules captured inside the
    /// occurrence of the parent being written, or every key outside one,
    /// each distinct key once, rule by rule (TLMD_197).
    pub(super) fn keys_in_scope(&self, rule_ids: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for rule_id in rule_ids {
            let from = self
                .scope
                .as_ref()
                .and_then(|s| s.get(rule_id).copied())
                .unwrap_or(0);
            let keys = self.keys.get(rule_id).into_iter().flatten().skip(from);
            push_unique(&mut out, keys.cloned());
        }
        out
    }

    /// How many keys each rule has captured so far: where a scope begins.
    pub(super) fn key_counts(&self) -> BTreeMap<String, usize> {
        self.keys
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect()
    }

    /// One text per key a pairing unit rule demands, each its example with
    /// the key spliced in. `None` where the rule pairs nothing.
    // sokf-implements: MD-7_AC-5
    // sokf-implements: TLMD_129
    pub(super) fn paired_units(&mut self, block: &Block, rule: &Unit) -> Option<Vec<String>> {
        let with = rule.for_each_key_of.as_deref()?;
        let example = self.unit_text_at(block, Some(rule), 0);
        let reader = self.key_reader(rule, &example)?;
        let mut out = Vec::new();
        for key in self.keys_in_scope(with) {
            match respell(&reader, &example, &key) {
                Some(text) => out.push(text),
                // sokf-implements: TLMD_154
                None => self.refuse_on(
                    rule.common.line,
                    "TLMD_154",
                    format!(
                        "a rule demands one unit per key of {}, and its example `{example}` does not carry `{key}` where `{}` reads it",
                        check::rules_named(with),
                        reader.name
                    ),
                ),
            }
        }
        Some(out)
    }
}

/// What `reader` captures from `text`. A line of a fence is read without
/// its indent (TLMD_6); no other unit's text starts with one.
pub(super) fn capture_of(reader: &KeyReader, text: &str) -> Option<String> {
    reader
        .regex
        .captures(text.trim_start())?
        .name(&reader.name)
        .map(|m| m.as_str().to_string())
}

/// `text` with what `reader` captures replaced by `want`, or `None` where
/// it captures nothing or the result no longer reads as `want`. The splice
/// is how one derived key is made to agree with another: a paired heading
/// takes the key its rule captured, and a derived key takes the prefix its
/// parent captured. An indent is kept as written.
pub(super) fn respell(reader: &KeyReader, text: &str, want: &str) -> Option<String> {
    let body = text.trim_start();
    let indent = &text[..text.len() - body.len()];
    let at = reader.regex.captures(body)?.name(&reader.name)?;
    let mut out = String::with_capacity(text.len());
    out.push_str(indent);
    out.push_str(&body[..at.start()]);
    out.push_str(want);
    out.push_str(&body[at.end()..]);
    (capture_of(reader, &out).as_deref() == Some(want)).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_splice_keeps_the_shape_or_gives_up() {
        let p = KeyReader {
            regex: Regex::new(r"^(?P<id>[A-Z]+-\d+): ").unwrap(),
            name: "id".into(),
        };
        assert_eq!(
            respell(&p, "EX-1: a thing", "EX-2").as_deref(),
            Some("EX-2: a thing")
        );
        // The wanted key does not answer to the pattern that holds it.
        assert_eq!(respell(&p, "EX-1: a thing", "nope"), None);
        // Nothing to splice into.
        assert_eq!(respell(&p, "no key here", "EX-2"), None);
    }
}
