//! The label of each unit that a scaffold writes (`TLMD_255`, `TLMD_256`): the
//! label that the host names, else the one that the unit's example
//! carries, else the first of the schema, in the words of the host, where
//! the unit's rule requires one or its example carries one.

use super::Deriver;
use crate::{
    grammar::{
        Block, Found, Label, Shape, Squashed, Unit, begins_with_key, carries_labels, cut, squash,
        task_box,
    },
    markdown::{BlockKind, Reading},
};

/// What an example says of its label: whether it carries one, and the key
/// of the label that its words stand for, where they stand for one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Carried {
    /// Whether the example carries a label.
    pub(super) any: bool,
    /// The key of the label, where its words are a label's or its key.
    pub(super) key: Option<String>,
    /// Whether the text without its label fails the shape of its rule, so
    /// that the scaffold writes the example as it stands, and the check of
    /// the example reports it once.
    pub(super) fails: bool,
}

impl Deriver<'_> {
    /// Whether the units of `block` that `rule` governs carry labels that
    /// the scaffold writes: the schema declares labels, the block's units
    /// carry them, and the rule or the block declares `label`.
    fn writes_labels(&self, block: &Block, rule: Option<&Unit>) -> bool {
        !self.labels.is_empty() && carries_labels(block.kind) && block.label_of(rule).is_some()
    }

    /// `text`, the text of a unit of `block` that `rule` governs, without
    /// the label that it carries, and what it says of that label. A unit
    /// whose rule declares no `label` keeps its text, as the example states
    /// it, so the check of the example reports its label.
    pub(super) fn unlabel(
        &self,
        block: &Block,
        rule: Option<&Unit>,
        text: String,
    ) -> (String, Carried) {
        if !self.writes_labels(block, rule) {
            return (text, Carried::default());
        }
        let (bare, found) = self.cut_label(block, rule, &text);
        let shaped = |r: &Unit| {
            matches!(r.shape, Some(Shape::Template(_) | Shape::Pattern(_))) || r.format.is_some()
        };
        if let Some(r) = rule.filter(|r| shaped(r))
            && !r.fits(
                self.schema.formats_of(r),
                &squash(&r.read_text(&bare)),
                |_| None,
            )
        {
            return (
                text,
                Carried {
                    fails: true,
                    ..Carried::default()
                },
            );
        }
        let carried = Carried {
            any: found.is_some(),
            key: found.and_then(|f| self.labels.key_of(&f.words).map(str::to_string)),
            fails: false,
        };
        (bare, carried)
    }

    /// `text`, the text of a unit of `block` that `rule` governs without a
    /// label, with the label that the scaffold writes where it writes one:
    /// on a unit whose rule requires a label, and on one whose rule allows
    /// one and whose example carries one. The label is the one that the
    /// host names, else the one that `carried` names, else the first of the
    /// schema, in the host's words, straight after a key that begins the
    /// text, and else first, after a task's box.
    // sokf-implements: TLMD_255
    // sokf-implements: TLMD_256
    // sokf-implements: MD-7_AC-15
    pub(super) fn relabel(
        &self,
        block: &Block,
        rule: Option<&Unit>,
        text: String,
        carried: &Carried,
    ) -> String {
        if !self.writes_labels(block, rule) || carried.fails {
            return text;
        }
        if block.label_of(rule) == Some(Label::Optional) && !carried.any {
            return text;
        }
        let key = self
            .named_label
            .as_deref()
            .or(carried.key.as_deref())
            .or_else(|| self.labels.first());
        let Some(words) = key.and_then(|k| self.labels.words(k)) else {
            return text;
        };
        let label = format!("[{words}] ");
        let item = matches!(block.kind, BlockKind::BulletList | BlockKind::NumberedList);
        let base = if item { task_box(&text) } else { 0 };
        // The rule captures its key from the reading, so the key is found
        // there, and its end mapped back to the text as written.
        let read = rule.map_or_else(|| Reading::written(&text), |r| r.reading_of(&text));
        let read_base = if item { task_box(&read.text) } else { 0 };
        let after_key = rule
            .filter(|r| self.schema.unit_captures_key(r))
            .and_then(|r| r.key_in(self.schema.formats_of(r), &squash(&read.text)))
            .filter(|k| begins_with_key(&read.text[read_base..], k))
            .map(|k| read.original(read_base + k.len()).max(base));
        let at = match after_key {
            // A unit that is only its key has no room for a label after it.
            Some(end) if end >= text.len() => return text,
            Some(end) => end + 1,
            None => base,
        };
        format!("{}{label}{}", &text[..at], &text[at..])
    }

    /// `text`, a unit of `block` that the scaffold wrote, as `rule` reads
    /// it in the check: without the label that the rule reads there, which
    /// for a rule with no `label` is only a label of the schema (`TLMD_247`).
    pub(super) fn read_as(&self, block: &Block, rule: &Unit, text: &str) -> String {
        self.cut_label(block, Some(rule), text).0
    }

    /// `text` without the label that `rule` reads in it, cut from the text
    /// as written with the whitespace after it, and the label.
    fn cut_label(&self, block: &Block, rule: Option<&Unit>, text: &str) -> (String, Option<Found>) {
        if self.labels.is_empty() || !carries_labels(block.kind) {
            return (text.to_string(), None);
        }
        let squashed = Squashed::new(text);
        match self
            .schema
            .label_in(&self.labels, block, rule, &squashed.text)
        {
            Some(f) => {
                let at = squashed.original(f.at.start)..squashed.original(f.at.end);
                (cut(text, &at), Some(f))
            }
            None => (text.to_string(), None),
        }
    }
}
