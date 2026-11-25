//! Where a derived unit's text comes from, and the splices that make one
//! key agree with another.

use std::{collections::BTreeMap, ops::Range};

use regex::Regex;

use super::Deriver;
use crate::{
    check,
    grammar::{
        Block, DerivationSource, KeyDerivation, KeyText, Shape, Squashed, Text, Unit, squash,
    },
    markdown::{self, BlockKind, Reading},
    text,
};

/// How a key is read out of a text as the check reads it: the reading of
/// the text, the regex of the shape as read, and the name of its group.
#[derive(Debug, Clone)]
pub(super) struct KeyReader {
    /// The regex of the shape as read that holds the key.
    pub regex: Regex,
    /// The group's name.
    pub name: String,
    /// How the rule reads a text (`TLMD_259`).
    pub read: fn(&str) -> Reading,
    /// Whether the rule reads its text as written, as a line of a fence.
    pub verbatim: bool,
}

impl KeyReader {
    /// The key of `shape`, a shape as read, named `name`, in a text that
    /// `read` reads, as written where `verbatim`, where the shape has a
    /// regex.
    pub(super) fn of(
        shape: &Shape,
        name: &str,
        read: fn(&str) -> Reading,
        verbatim: bool,
    ) -> Option<KeyReader> {
        Some(KeyReader {
            regex: shape.regex()?.clone(),
            name: name.to_string(),
            read,
            verbatim,
        })
    }

    /// The key that the reader captures from `text`, as written, in the
    /// reading of `text` squashed, as the check captures it (`TL_23`), with
    /// the bytes of `text` that the key stands for. A backslash that the
    /// reading drops belongs to the character after it, and a backtick that
    /// the reading of a code span drops to the code span. A line of a fence
    /// is read without its indent (`TLMD_6`); no other unit's text starts
    /// with one. The third value is the byte of `text` where the first
    /// `prefix` bytes of the key end.
    fn find(&self, text: &str, prefix: usize) -> Option<(String, Range<usize>, usize)> {
        let body = text.trim_start();
        let indent = text.len() - body.len();
        let reading = (self.read)(body);
        let squashed = Squashed::mapped(&reading.text, &reading.map);
        let at = self.regex.captures(&squashed.text)?.name(&self.name)?;
        let start = squashed.original(at.start());
        let span = reading.span_of(body, start, squashed.original(at.end()));
        let end = squashed.original(at.start() + prefix.min(at.len()));
        let cut = reading.span_of(body, start, end).end;
        Some((
            at.as_str().to_string(),
            indent + span.start..indent + span.end,
            indent + cut,
        ))
    }

    /// The key that the reader captures from `text`, as written, as the
    /// check captures it.
    pub(super) fn capture(&self, text: &str) -> Option<String> {
        self.find(text, 0).map(|(key, ..)| key)
    }

    /// The key that the reader captures from `text`, with the bytes of
    /// `text` that it stands for, as the comparison of two keys reads it
    /// (`TLMD_287`).
    pub(super) fn key_text(&self, text: &str) -> Option<KeyText> {
        let (key, at, _) = self.find(text, 0)?;
        Some(KeyText::new(&key, &text[at], self.verbatim))
    }

    /// The bytes of `text` that the first `prefix` bytes of the key that
    /// the reader captures stand for.
    fn prefix_written<'t>(&self, text: &'t str, prefix: usize) -> Option<&'t str> {
        let (_, at, cut) = self.find(text, prefix)?;
        text.get(at.start..cut)
    }

    /// The bytes of `key`, as read, the key that the reader captures from
    /// `text`, that `parent`, which `key` derives from, covers: its reading,
    /// where the rules of the two read alike, and else the first part of
    /// the key before a separator `sep` whose plain text is the parent's
    /// (`TLMD_287`).
    fn prefix_of(&self, text: &str, key: &KeyText, parent: &KeyText, sep: &str) -> Option<usize> {
        if key.verbatim == parent.verbatim {
            return Some(parent.read.len());
        }
        key.read.match_indices(sep).map(|(at, _)| at).find(|&at| {
            self.prefix_written(text, at).is_some_and(|written| {
                KeyText::new(&key.read[..at], written, self.verbatim).plain() == parent.plain()
            })
        })
    }

    /// `text`, as written, with the bytes that the key that the reader
    /// captures stand for replaced by `key`, a key that another rule
    /// captured, or `None` where it captures nothing or the result no
    /// longer captures the same key (`TLMD_287`). The key is written as its
    /// own text writes it, and else as it reads, so that a scaffold writes
    /// the text of the schema as written (`TLMD_259`).
    // sokf-implements: MD-7_AC-17
    pub(super) fn respell_key(&self, text: &str, key: &KeyText) -> Option<String> {
        let (_, at, _) = self.find(text, 0)?;
        spellings(key).into_iter().find_map(|write| {
            let out = splice(text, at.clone(), write);
            self.key_text(&out)
                .is_some_and(|got| got.same(key))
                .then_some(out)
        })
    }

    /// `text`, as written, with the bytes that the first `prefix` bytes of
    /// the key that the reader captures stand for replaced by `write`, and
    /// the rest of the key kept as written, or `None` where it captures
    /// nothing or the result no longer reads as `read`. A key that derives
    /// takes the prefix of its parent so, and keeps its own tail as the
    /// example writes it (`TLMD_259`).
    pub(super) fn respell_prefix(
        &self,
        text: &str,
        prefix: usize,
        write: &str,
        read: &str,
    ) -> Option<String> {
        let (_, at, cut) = self.find(text, prefix)?;
        self.splice(text, at.start..cut, write, read)
    }

    /// `text` with the bytes `at` replaced by `write`, where the result
    /// captures `read`.
    fn splice(&self, text: &str, at: Range<usize>, write: &str, read: &str) -> Option<String> {
        let out = splice(text, at, write);
        (self.capture(&out).as_deref() == Some(read)).then_some(out)
    }
}

/// `text` with the bytes `at` replaced by `write`.
fn splice(text: &str, at: Range<usize>, write: &str) -> String {
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..at.start]);
    out.push_str(write);
    out.push_str(&text[at.end..]);
    out
}

// sokf-implements: MD-7_AC-17
/// The texts that can write `key`, in the order to try them: as its text
/// writes it, then as it alone writes it, squashed, then as it reads.
fn spellings(key: &KeyText) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for write in [key.written(), key.spelt(), key.read.as_str()] {
        if !out.contains(&write) {
            out.push(write);
        }
    }
    out
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
                Some(Shape::Text(Text::Literal(t))) => {
                    let subject = "the `text` of a unit rule";
                    return (vec![self.bound_text(block, line, t, subject)], false);
                }
                Some(Shape::Text(Text::From(key))) => {
                    // sokf-implements: TLMD_155
                    let Some(value) = self.values.get(key).cloned() else {
                        self.refuse_on(
                            line,
                            "TLMD_155",
                            format!("a unit rule takes its text from `{key}`, which no frontmatter writes"),
                        );
                        return (vec![String::new()], false);
                    };
                    let subject =
                        format!("the value that the scaffold writes for the frontmatter `{key}`");
                    return (vec![self.bound_text(block, line, &value, &subject)], false);
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

    /// `text`, the text that a rule of `block` fixes, as the scaffold writes
    /// it after the block's marker: squashed, as the check reads it, and
    /// written to read as a unit of its kind (`TLMD_275`), or refused where
    /// no unit of its kind holds it (`TLMD_277`). `subject` names the text.
    // sokf-implements: TLMD_275
    // sokf-implements: TLMD_277
    fn bound_text(&mut self, block: &Block, line: usize, text: &str, subject: &str) -> String {
        let marker = match block.kind {
            BlockKind::BulletList => "- ",
            BlockKind::NumberedList => "1. ",
            BlockKind::Blockquote => "> ",
            BlockKind::Prose => "",
            // A line of a fence or of an HTML block reads as written.
            _ => return text.to_string(),
        };
        let text = squash(text);
        match markdown::unit_written(marker, &text) {
            Ok(written) => written.into_owned(),
            Err(why) => {
                let noun = check::unit_of(block.kind);
                self.refuse_on(line, "TLMD_277", why.said(subject, &text, noun));
                text
            }
        }
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

    /// How `rule` reads a key out of `text`, an example as written, as the
    /// check reads it: in the reading of the rule, through its own shape as
    /// read and `key`, or the shape of its format as read that the reading
    /// takes and the format's key (`TLMD_259`).
    // sokf-implements: TLMD_259
    pub(super) fn key_reader(&self, rule: &Unit, text: &str) -> Option<KeyReader> {
        if let Some(shape) = rule.matcher() {
            return KeyReader::of(
                shape,
                rule.key.as_deref()?,
                rule.reader(),
                !rule.reads_prose,
            );
        }
        let format = self.schema.formats_of(rule).get(rule.format.as_ref()?)?;
        let name = rule.key.as_ref().or(format.key.as_ref())?;
        let (shape, _) = format.shape_for(&squash(&rule.read_text(text.trim_start())))?;
        KeyReader::of(shape, name, rule.reader(), !rule.reads_prose)
    }

    /// The key `rule` would capture from `text`, an example as written, as
    /// the check captures it.
    pub(super) fn captured(&self, rule: &Unit, text: &str) -> Option<KeyText> {
        self.key_reader(rule, text)?.key_text(text)
    }

    /// Record the key `rule` captures from `text` under the name that the
    /// check files it under: the rule's `key-of`, else the `rule-id` of
    /// its block, its group, or its section.
    pub(super) fn record_capture(&mut self, block: &Block, rule: &Unit, text: &str) {
        let key = self.captured(rule, text);
        let id = rule
            .files_under(&[block.rule_id.as_deref(), self.group_rule.as_deref()])
            .map(str::to_string);
        self.record(id.as_ref(), key);
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
    /// for its frontmatter key. The splice replaces exactly the example of
    /// the parent, the value that the schema gives the frontmatter key or a
    /// key of the rule's example, that starts the key followed by the
    /// separator. A key that starts with no example of its parent is left
    /// as written, because a guess at its prefix would hide the fault that
    /// `TL_127` reports. Where the schema states no value of the parent, as
    /// for a value that only the host seeds, no example check applies, and
    /// the prefix is the text before the key's last separator, as the
    /// finding of `TL_33` reads it. The key reads as the check reads it,
    /// as the keys of its parents are recorded (`TLMD_259`), and a parent of
    /// another reading is compared by its plain text (`TLMD_287`); where no
    /// part of the key before a separator has that plain text, the splice
    /// has no place, and the derivation refuses it (`TLMD_154`).
    // sokf-implements: MD-7_AC-12
    // sokf-implements: TLMD_205
    pub(super) fn derive_with(
        &mut self,
        kd: &KeyDerivation,
        reader: &KeyReader,
        text: String,
        line: usize,
    ) -> String {
        let Some(key) = reader.key_text(&text) else {
            return text;
        };
        let sep = &kd.separator;
        let examples = self
            .schema
            .parent_examples(kd, reader.read, reader.verbatim);
        // A key reads a value of the frontmatter as its rule reads it, and the
        // splice writes the value as the frontmatter writes it (`TLMD_259`).
        let parents: Vec<KeyText> = match &kd.source {
            DerivationSource::Rule(r) => self.keys_of(r),
            DerivationSource::From(k) => self
                .values
                .get(k)
                .map(|v| KeyText::new(&squash(&(reader.read)(v).text), v, reader.verbatim))
                .into_iter()
                .collect(),
        };
        // A key with no parent written is left as the example writes it.
        if parents.is_empty() {
            return text;
        }
        // The bytes of the key as read that its prefix covers.
        let prefix = if examples.is_empty() {
            // sokf-implements: TLMD_216
            if key.parent_in(kd, &parents).is_some() {
                return text;
            }
            match key.read.rfind(sep.as_str()) {
                Some(at) => at,
                None => return text,
            }
        } else {
            let Some(example) = key.parent_in(kd, &examples) else {
                return text;
            };
            // A parent the scaffold wrote as its example is the prefix already.
            if parents.iter().any(|p| p.same(example)) {
                return text;
            }
            let Some(at) = reader.prefix_of(&text, &key, example, sep) else {
                // sokf-implements: TLMD_154
                self.refuse_on(
                    line,
                    "TLMD_154",
                    format!(
                        "a key deriving from {} cannot be written: no part of {} before a separator {} is {} where `{}` takes it",
                        kd.source_label(),
                        text::code(&key.read),
                        text::code(sep),
                        text::code(&example.read),
                        reader.name
                    ),
                );
                return text;
            };
            at
        };
        let parent = &parents[0];
        // The splice replaces the prefix alone, and keeps the separator and
        // the tail as the example writes them.
        let tail = &key.read[prefix..];
        let mut want = String::new();
        for write in spellings(parent) {
            want = format!("{}{tail}", squash(&(reader.read)(write).text));
            if let Some(spliced) = reader.respell_prefix(&text, prefix, write, &want) {
                return spliced;
            }
        }
        // sokf-implements: TLMD_154
        self.refuse_on(
            line,
            "TLMD_154",
            format!(
                "a key deriving from {} cannot be written: {} does not read as {} where `{}` takes it",
                kd.source_label(),
                text::code(&key.read),
                text::code(&want),
                reader.name
            ),
        );
        text
    }

    /// Record `key` against the block's `rule-id`, else the enclosing
    /// section's, which is how the checker attributes a capture.
    pub(super) fn record(&mut self, rule_id: Option<&String>, key: Option<KeyText>) {
        let id = rule_id.or(self.section_rule.as_ref());
        if let (Some(id), Some(key)) = (id, key) {
            self.keys.entry(id.clone()).or_default().push(key);
        }
    }

    /// The keys a rule captured, in the order it captured them.
    pub(super) fn keys_of(&self, rule_id: &str) -> Vec<KeyText> {
        self.keys.get(rule_id).cloned().unwrap_or_default()
    }

    /// The keys a pairing reads: those that its rules captured inside the
    /// occurrence of the parent being written, or every key outside one,
    /// each distinct key once, rule by rule (`TLMD_197`), a key of another
    /// reading with its plain text the same key (`TLMD_287`).
    pub(super) fn keys_in_scope(&self, rule_ids: &[String]) -> Vec<KeyText> {
        let mut out: Vec<KeyText> = Vec::new();
        for rule_id in rule_ids {
            let from = self
                .scope
                .as_ref()
                .and_then(|s| s.get(rule_id).copied())
                .unwrap_or(0);
            for key in self.keys.get(rule_id).into_iter().flatten().skip(from) {
                if !out.iter().any(|k| k.same(key)) {
                    out.push(key.clone());
                }
            }
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
        let (example, carried) = self.unlabel(block, Some(rule), example);
        let reader = self.key_reader(rule, &example)?;
        let mut out = Vec::new();
        for key in self.keys_in_scope(with) {
            match reader.respell_key(&example, &key) {
                Some(text) => out.push(self.relabel(block, Some(rule), text, &carried)),
                // sokf-implements: TLMD_154
                None => self.refuse_on(
                    rule.common.line,
                    "TLMD_154",
                    format!(
                        "a rule demands one unit per key of {}, and its example {example} does not carry {} where `{}` reads it",
                        check::rules_named(with),
                        text::code(&key.read),
                        reader.name, example = text::code(&example)),
                ),
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown;

    /// A key that writes as it reads.
    fn key(text: &str) -> KeyText {
        KeyText::new(text, text, false)
    }

    #[test]
    fn a_splice_keeps_the_shape_or_gives_up() {
        let p = KeyReader {
            regex: Regex::new(r"^(?P<id>[A-Z]+-\d+): ").unwrap(),
            name: "id".into(),
            read: Reading::written,
            verbatim: true,
        };
        assert_eq!(
            p.respell_key("EX-1: a thing", &key("EX-2")).as_deref(),
            Some("EX-2: a thing")
        );
        // The wanted key does not answer to the pattern that holds it.
        assert_eq!(p.respell_key("EX-1: a thing", &key("nope")), None);
        // Nothing to splice into.
        assert_eq!(p.respell_key("no key here", &key("EX-2")), None);
    }

    // sokf-tests: TLMD_259
    #[test]
    fn a_splice_reads_the_key_in_the_reading_and_writes_the_text() {
        let p = KeyReader {
            regex: Regex::new(r"^\*\*Term\*\*: (?P<t>.+)$").unwrap(),
            name: "t".into(),
            read: markdown::reading,
            verbatim: false,
        };
        assert_eq!(p.capture("__Term__: x").as_deref(), Some("x"));
        assert_eq!(
            p.respell_key("__Term__: x", &key("y")).as_deref(),
            Some("__Term__: y")
        );
        // A backslash that the reading drops belongs to the key that it
        // starts, and a backtick that it drops to the code span.
        assert_eq!(
            p.respell_key("**Term**: \\_x", &key("y")).as_deref(),
            Some("**Term**: y")
        );
        let code = KeyReader {
            regex: Regex::new(r"^`(?P<t>[a-z]+)`$").unwrap(),
            name: "t".into(),
            read: markdown::reading,
            verbatim: false,
        };
        assert_eq!(
            code.respell_key("``x``", &key("y")).as_deref(),
            Some("``y``")
        );
        assert_eq!(
            code.respell_key("` x `", &key("y")).as_deref(),
            Some("` y `")
        );
        // The key reads as the check reads it, squashed.
        assert_eq!(p.capture("__Term__:  x").as_deref(), Some("x"));
        assert_eq!(
            p.respell_key("__Term__:  x", &key("y")).as_deref(),
            Some("__Term__:  y")
        );
    }
}
