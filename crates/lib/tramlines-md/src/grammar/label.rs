//! How a label is read off the text of a unit (`TLMD_243` to `TLMD_247`): its
//! words in square brackets followed by one space, straight after the key
//! of a unit that begins with the key that its rule captures, and
//! otherwise first, after the box of a task item. The check, the check of
//! an example, and the derivation all read a label through here.

use std::ops::Range;

use serde::Serialize;

use super::{Block, Schema, Squashed, Unit};
use crate::{markdown::BlockKind, text};

/// The labels of a schema, each key with the words that a document
/// writes for it: the words that the host gives in the language of the
/// schema, or the key where the host gives none (`TLMD_253`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Words {
    labels: Vec<(String, String)>,
}

impl Words {
    /// The labels `labels`, each a key and its words.
    pub(crate) fn new(labels: Vec<(String, String)>) -> Words {
        Words { labels }
    }

    /// The labels of `schema`, each with its key as its words, as a host
    /// that gives no words has them.
    pub(crate) fn keys_of(schema: &Schema) -> Words {
        Words::new(
            schema
                .labels
                .iter()
                .map(|k| (k.clone(), k.clone()))
                .collect(),
        )
    }

    /// Whether the schema declares no label.
    pub(crate) fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// The key of the first label of the schema.
    pub(crate) fn first(&self) -> Option<&str> {
        self.labels.first().map(|(k, _)| k.as_str())
    }

    /// Whether `key` is the key of a label.
    pub(crate) fn has_key(&self, key: &str) -> bool {
        self.labels.iter().any(|(k, _)| k == key)
    }

    /// Each label, its key and its words, in the order of the schema.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.labels.iter().map(|(k, w)| (k.as_str(), w.as_str()))
    }

    /// The words of the label `key`.
    pub(crate) fn words(&self, key: &str) -> Option<&str> {
        self.labels
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, w)| w.as_str())
    }

    /// Whether `words` are the words of a label.
    pub(crate) fn accepts(&self, words: &str) -> bool {
        self.labels.iter().any(|(_, w)| w == words)
    }

    /// The key of the label that `words` stand for: the label whose words
    /// they are, else the label whose key they are.
    pub(crate) fn key_of(&self, words: &str) -> Option<&str> {
        self.labels
            .iter()
            .find(|(_, w)| w == words)
            .or_else(|| self.labels.iter().find(|(k, _)| k == words))
            .map(|(k, _)| k.as_str())
    }

    /// The words of each label as a document writes them, for a finding.
    pub(crate) fn listed(&self) -> String {
        text::code_list(self.labels.iter().map(|(_, w)| format!("[{w}]")), " or ")
    }
}

/// Grammar `label` of a block or a unit rule: whether each unit carries
/// one of the labels that the schema declares (`TLMD_240`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Label {
    /// Every unit carries a label.
    Required,
    /// A unit carries a label where the writer gives one.
    Optional,
}

impl Block {
    /// Whether a unit of the block that `rule` governs carries a label: the
    /// `label` of the rule, else, for a position of a `sequence`, the
    /// `label` of the block's `each`, else the block's (`TLMD_242`). `rule` is
    /// `None` for a unit of a block that declares no unit rule.
    // sokf-implements: TLMD_242
    pub fn label_of(&self, rule: Option<&Unit>) -> Option<Label> {
        let position =
            rule.is_some_and(|r| self.sequence.iter().flatten().any(|p| std::ptr::eq(p, r)));
        rule.and_then(|r| r.label)
            .or_else(|| {
                position
                    .then(|| self.each.as_ref().and_then(|e| e.label))
                    .flatten()
            })
            .or(self.label)
    }
}

/// Whether `words`, squashed, can stand between a label's brackets: not
/// empty, with no square bracket, and not opening with `^`, which marks a
/// footnote reference (`TLMD_243`).
pub(crate) fn is_words(words: &str) -> bool {
    !words.is_empty() && !words.contains(['[', ']']) && !words.starts_with('^')
}

/// Whether the units of a block of `kind` can carry a label: paragraphs,
/// items, and rows (`TLMD_240`).
pub(crate) fn carries_labels(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::Prose
            | BlockKind::Blockquote
            | BlockKind::BulletList
            | BlockKind::NumberedList
            | BlockKind::Table
    )
}

/// A label read off the squashed text of a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Found {
    /// The bytes of `[words] ` in the squashed text.
    pub at: Range<usize>,
    /// The words between the brackets.
    pub words: String,
}

/// The words of the label that `text` opens with, and the length of the
/// label with the space after it, where `text` opens with one (`TLMD_243`).
fn label_at(text: &str) -> Option<(&str, usize)> {
    let inner = text.strip_prefix('[')?;
    let end = inner.find([']', '['])?;
    let words = &inner[..end];
    (inner[end..].starts_with("] ") && words == words.trim() && is_words(words))
        .then_some((words, end + 3))
}

/// How long the box of a task item is that opens `text`, with the space or
/// the tab after it, as GitHub reads a box, or 0 where it opens with none
/// (`TLMD_246`).
pub(crate) fn task_box(text: &str) -> usize {
    match text.as_bytes() {
        [b'[', b' ' | b'x' | b'X', b']', b' ' | b'\t', ..] => 4,
        _ => 0,
    }
}

/// `text` without the bytes `at`.
pub(crate) fn cut(text: &str, at: &Range<usize>) -> String {
    format!("{}{}", &text[..at.start], &text[at.end..])
}

/// `squashed` without the bytes `at`, each byte left mapped to where it
/// was in the original text.
pub(crate) fn without(squashed: &Squashed, at: &Range<usize>) -> Squashed {
    let mut map = squashed.map.clone();
    map.drain(at.clone());
    Squashed {
        text: cut(&squashed.text, at),
        map,
    }
}

/// Whether `text` begins with `key`, followed by a space or by nothing.
pub(crate) fn begins_with_key(text: &str, key: &str) -> bool {
    !key.is_empty()
        && text
            .strip_prefix(key)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
}

/// How a rule reads the label of a unit: whether the unit is an item of
/// a list, which can be a task item, whether the rule declares `label`,
/// and the key that the rule captures from a text without its label.
pub(crate) struct Reader<'a> {
    /// The labels of the schema in the host's words.
    pub words: &'a Words,
    /// Whether the unit is an item of a list.
    pub item: bool,
    /// Whether the rule declares `label`, so that any words read as one.
    pub labelled: bool,
    /// Whether the rule captures a key.
    pub keyed: bool,
    /// The key that the rule captures from a squashed text, or `None`
    /// where it captures none from it.
    pub key: &'a dyn Fn(&str) -> Option<String>,
}

impl Reader<'_> {
    /// The label of the squashed `text`, where it holds one at its place:
    /// straight after a key that begins the text, and else first, after a
    /// task's box. Where the rule declares no `label`, only the words or the
    /// key of a label of the schema read as one.
    // sokf-implements: TLMD_243
    // sokf-implements: TLMD_244
    // sokf-implements: TLMD_245
    // sokf-implements: TLMD_246
    // sokf-implements: TLMD_247
    pub(crate) fn find(&self, text: &str) -> Option<Found> {
        if self.words.is_empty() {
            return None;
        }
        let base = if self.item { task_box(text) } else { 0 };
        let rest = &text[base..];
        let known = |w: &str| self.words.key_of(w).is_some();
        let ok = |w: &str| self.labelled || known(w);
        // Straight after the key: after a space, where the text before it
        // is the key that the rule captures from the text without the
        // label, so a key can hold a space. A text that its rule can't read
        // still drops a label of the schema after its first word, so that
        // the failure of its shape points at the fault.
        if self.keyed {
            let first = rest.find(' ');
            let mut fallback = None;
            for (space, _) in rest.match_indices(" [") {
                let start = base + space + 1;
                let Some((words, len)) = label_at(&text[start..]) else {
                    continue;
                };
                if !ok(words) {
                    continue;
                }
                let found = Found {
                    at: start..start + len,
                    words: words.to_string(),
                };
                match (self.key)(&cut(text, &found.at)) {
                    Some(key) if key == rest[..space] => return Some(found),
                    None if Some(space) == first && known(words) => fallback = Some(found),
                    _ => {}
                }
            }
            if fallback.is_some() {
                return fallback;
            }
        }
        // First: where the text without the label begins with its key, the
        // label belongs after the key, so this is none.
        let (words, len) = label_at(rest)?;
        if !ok(words) {
            return None;
        }
        let at = base..base + len;
        let unlabelled = cut(text, &at);
        // A label before a task's box belongs after the box.
        if self.item && base == 0 && task_box(&unlabelled) > 0 {
            return None;
        }
        if self.keyed
            && (self.key)(&unlabelled).is_some_and(|k| begins_with_key(&unlabelled[base..], &k))
        {
            return None;
        }
        Some(Found {
            at,
            words: words.to_string(),
        })
    }
}

impl Schema {
    /// The label that the squashed `text` of a unit of `block` holds, as
    /// `rule`, a unit rule of the block or `None` for a block with none,
    /// reads it, with `words` the labels in the host's words: none where
    /// the schema declares no label or the block's units carry none.
    pub(crate) fn label_in(
        &self,
        words: &Words,
        block: &Block,
        rule: Option<&Unit>,
        text: &str,
    ) -> Option<Found> {
        if words.is_empty() || !carries_labels(block.kind) {
            return None;
        }
        let key = |t: &str| rule.and_then(|r| r.key_in(self.formats_of(r), t));
        Reader {
            words,
            item: matches!(block.kind, BlockKind::BulletList | BlockKind::NumberedList),
            labelled: block.label_of(rule).is_some(),
            keyed: rule.is_some_and(|r| self.unit_captures_key(r)),
            key: &key,
        }
        .find(text)
    }

    /// `text`, the text of a unit of `block` that `rule` governs, as the
    /// rule reads it: without the label that it holds, squashed, with the
    /// words of the label and the place where it stood, as `label_in` reads
    /// it with `words`.
    pub(crate) fn unlabelled(
        &self,
        words: &Words,
        block: &Block,
        rule: Option<&Unit>,
        text: &str,
    ) -> (String, Option<Found>) {
        let squashed = super::squash(text);
        match self.label_in(words, block, rule, &squashed) {
            Some(found) => (cut(&squashed, &found.at), Some(found)),
            None => (squashed, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(keys: &[&str]) -> Words {
        Words::new(
            keys.iter()
                .map(|k| (k.to_string(), k.to_string()))
                .collect(),
        )
    }

    #[test]
    fn a_label_is_words_in_brackets_and_a_space() {
        assert_eq!(label_at("[human] D-2"), Some(("human", 8)));
        assert_eq!(label_at("[un humain] x"), Some(("un humain", 12)));
        assert_eq!(label_at("[human]: x"), None);
        assert_eq!(label_at("[human][x] y"), None);
        assert_eq!(label_at("[] x"), None);
        assert_eq!(label_at("[ ] x"), None);
        assert_eq!(label_at("[a[b] x"), None);
        assert_eq!(label_at("[human]"), None);
        assert_eq!(label_at("[^1] x"), None);
    }

    #[test]
    fn a_label_stands_after_a_key_that_begins_the_unit_and_else_first() {
        let w = words(&["human", "agent"]);
        let key = |t: &str| {
            t.split(' ')
                .find(|k| k.starts_with("D-"))
                .map(str::to_string)
        };
        let r = Reader {
            words: &w,
            item: true,
            labelled: true,
            keyed: true,
            key: &key,
        };
        assert_eq!(r.find("D-2 [human] Why.").map(|f| f.at), Some(4..12));
        // A label first, before a key that begins the unit, is none.
        assert_eq!(r.find("[human] D-2 Why."), None);
        // A unit that begins with no key holds its label first.
        assert_eq!(r.find("[agent] See D-2.").map(|f| f.at), Some(0..8));
        assert_eq!(r.find("[agent] **Topic**: x").map(|f| f.at), Some(0..8));
        // A task's box comes first.
        assert_eq!(r.find("[ ] [agent] WHEN x").map(|f| f.at), Some(4..12));
        assert_eq!(r.find("[x] D-3 [agent] y").map(|f| f.at), Some(8..16));
        assert_eq!(r.find("[agent] [ ] WHEN x"), None);
        // A key can hold a space.
        let step = |t: &str| t.starts_with("Step 1").then(|| "Step 1".to_string());
        let s = Reader { key: &step, ..r };
        assert_eq!(s.find("Step 1 [human] Do it.").map(|f| f.at), Some(7..15));
        // Any words read as a label where the rule declares one.
        assert_eq!(r.find("[Mensch] x").map(|f| f.words), Some("Mensch".into()));
        let plain = Reader {
            labelled: false,
            ..r
        };
        assert_eq!(plain.find("[sic] x"), None);
        assert_eq!(plain.find("[human] x").map(|f| f.at), Some(0..8));
    }
}
