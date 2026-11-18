//! The label of a unit in a check (TLMD_248 to TLMD_254): the rule reads
//! the unit's text without it, and the check holds it to what the rule
//! declares and to the words that the host gives.

use super::{Checker, Host, LabelFacts, LabelRepair, short};
use crate::{
    grammar::{
        Block as Rule, Found, Label, Schema, Squashed, Unit as UnitRule, Words, carries_labels,
        is_words, squash, without,
    },
    markdown::Unit,
};

/// The labels of `schema` in the words that `host` gives in the language
/// of the schema, each label whose host gives no words, blank words, or
/// words that hold a square bracket with its key as its words. The host is
/// asked only where the schema declares labels.
// sokf-implements: TLMD_252
// sokf-implements: TLMD_253
// sokf-implements: MD-5_AC-20
// sokf-implements: TLMD_234
// sokf-implements: MD-5_AC-21
pub(crate) fn words_of(schema: &Schema, host: &dyn Host) -> Words {
    let language = schema.core.language.as_deref();
    Words::new(
        schema
            .labels
            .iter()
            .map(|key| {
                let words = host
                    .label_words(&LabelFacts::new(key, language))
                    .map(|w| squash(&w))
                    .filter(|w| is_words(w))
                    .unwrap_or_else(|| key.clone());
                (key.clone(), words)
            })
            .collect(),
    )
}

/// The squashed text of a unit as its rule reads it: without its label,
/// and the label that it holds.
pub(crate) struct Read {
    /// The text without the label, each byte mapped to the unit's text.
    pub squashed: Squashed,
    /// The label, and the byte of the unit's text where it starts.
    pub label: Option<(Found, usize)>,
}

impl Checker<'_> {
    /// Whether a unit of `block` is read for a label: the schema declares
    /// labels, and the block's units carry them, which no line of a fence
    /// does.
    fn reads_labels(&self, block: &Rule) -> bool {
        !self.labels.is_empty() && carries_labels(block.kind)
    }

    /// The squashed text of `unit` as `rule`, a unit rule of `block`, reads
    /// it: without the label that it holds (TLMD_248). `rule` is `None` for
    /// a unit of a block with no unit rule. A line of a fence holds no
    /// label.
    // sokf-implements: TLMD_248
    pub(crate) fn read(&self, unit: &Unit, block: &Rule, rule: Option<&UnitRule>) -> Read {
        let squashed = Squashed::new(&unit.text);
        if !self.reads_labels(block) {
            return Read {
                squashed,
                label: None,
            };
        }
        // Words in brackets that a link reference definition of the
        // document names are a link, and no label.
        let link = |words: &str| self.links.contains(&squash(words).to_lowercase());
        match self
            .schema
            .label_in(&self.labels, block, rule, &squashed.text)
            .filter(|found| !link(&found.words))
        {
            Some(found) => Read {
                squashed: without(&squashed, &found.at),
                label: Some((found.clone(), squashed.original(found.at.start))),
            },
            None => Read {
                squashed,
                label: None,
            },
        }
    }

    /// The label of `unit` against what its rule declares: a label that
    /// the rule requires, a label of the schema that it doesn't allow, and
    /// words that are no label's in the host's language, of which a label
    /// written as its key is repaired where it stands. `shaped` says
    /// whether the unit has the shape of its rule. Returns whether the label
    /// breaks the rule: a finding that no repair settles.
    // sokf-implements: TLMD_249
    // sokf-implements: TLMD_250
    // sokf-implements: TLMD_251
    // sokf-implements: TLMD_254
    // sokf-implements: MD-4_AC-17
    pub(crate) fn check_label(
        &mut self,
        unit: &Unit,
        block: &Rule,
        rule: Option<&UnitRule>,
        read: &Read,
        shaped: bool,
    ) -> bool {
        if !self.reads_labels(block) {
            return false;
        }
        let common = rule.map_or(&block.common, |r| &r.common);
        match (block.label_of(rule), &read.label) {
            (Some(Label::Required), None) if shaped => {
                let listed = self.labels.listed();
                self.emit(
                    common,
                    Some(unit.line),
                    "TLMD_249",
                    format!(
                        "`{}` carries no label; its rule requires one: {listed}",
                        short(&read.squashed.text)
                    ),
                );
                true
            }
            (None, Some((found, _))) => {
                self.emit(
                    common,
                    Some(unit.line),
                    "TLMD_250",
                    format!(
                        "`{}` carries the label `[{}]`, and its rule declares no `label`",
                        short(&read.squashed.text),
                        found.words
                    ),
                );
                true
            }
            (Some(_), Some((found, start))) if !self.labels.accepts(&found.words) => {
                let listed = self.labels.listed();
                self.emit(
                    common,
                    Some(unit.line),
                    "TLMD_251",
                    format!(
                        "the label `[{}]` is none of the labels of the schema: {listed}",
                        found.words
                    ),
                );
                // A label written as its key takes the host's words where it
                // stands.
                if let Some(new) = self.labels.words(&found.words).map(str::to_string)
                    && let Some((line, col)) = unit.position(*start)
                {
                    self.out.label_repairs.push(LabelRepair {
                        line,
                        col,
                        old: format!("[{}]", found.words),
                        new: format!("[{new}]"),
                    });
                    return false;
                }
                true
            }
            _ => false,
        }
    }

    /// The label that a repair which rewrites `read`'s unit whole writes
    /// before its new text: the label that the unit holds, in the host's
    /// words where it is written as its key, and nothing where it holds
    /// none.
    pub(crate) fn label_kept(&self, read: &Read) -> String {
        let Some((found, _)) = &read.label else {
            return String::new();
        };
        let words = match self.labels.words(&found.words) {
            Some(w) if !self.labels.accepts(&found.words) => w,
            _ => found.words.as_str(),
        };
        format!("[{words}] ")
    }
}
