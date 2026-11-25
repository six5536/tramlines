//! A key as the engine compares it with another key (`TLMD_287`): two keys
//! whose rules read alike are one key where their readings are equal, and
//! a key of a rule that reads as written, such as a line of a fence, and a
//! key of a rule that reads prose are one key where their plain texts are
//! equal, the plain text from which the anchor of a key is formed.

use std::{cell::OnceCell, ops::Range};

use tramlines_core::grammar::{KeyDerivation, Squashed, captured, squash};

use super::{Block, Schema, Section, Unit, Words, without};
use crate::markdown;

/// A key as the comparison of two keys reads it (`TLMD_287`): its text as
/// its rule reads it, its text as written, and whether its rule reads its
/// text as written. The text that writes it alone and its plain text are
/// read only where a comparison or a scaffold needs them.
#[derive(Debug, Clone)]
pub(crate) struct KeyText {
    /// The key as its rule reads it: in the reading of its unit, squashed,
    /// or as written where the rule reads the lines of a fence or a block
    /// that holds no prose.
    pub read: String,
    /// The bytes of its unit's text that `read` stands for.
    written: String,
    /// Whether the rule reads its text as written.
    pub verbatim: bool,
    /// `spelt`, once read.
    spelt: OnceCell<String>,
    /// `plain`, once read.
    plain: OnceCell<String>,
}

impl KeyText {
    /// The key `read`, as its rule reads it, whose text writes it `written`,
    /// of a rule that reads its text as written where `verbatim`.
    pub(crate) fn new(read: &str, written: &str, verbatim: bool) -> KeyText {
        KeyText {
            read: read.to_string(),
            written: written.to_string(),
            verbatim,
            spelt: OnceCell::new(),
            plain: OnceCell::new(),
        }
    }

    /// The bytes of its unit's text that the key stands for.
    pub(crate) fn written(&self) -> &str {
        &self.written
    }

    /// The key as its text writes it, squashed, where that text alone reads
    /// as the key, and else the key as read: the text that a scaffold writes
    /// for the key. A key that cuts a piece of markup in two, such as `*a-1`
    /// of `*a-1 b*`, reads otherwise alone, so it is spelt as read, which
    /// its twins share.
    pub(crate) fn spelt(&self) -> &str {
        self.spelt.get_or_init(|| {
            if self.verbatim {
                return self.read.clone();
            }
            let written = squash(&self.written);
            if squash(&markdown::reading(&written).text) == self.read {
                written
            } else {
                self.read.clone()
            }
        })
    }

    /// The plain text of `spelt`, read as a paragraph alone: the text that
    /// the key renders as in prose, from which its anchor is formed.
    pub(crate) fn plain(&self) -> &str {
        self.plain
            .get_or_init(|| markdown::plain_text(self.spelt()))
    }

    /// The two texts by which `self` and `other` are compared: their
    /// readings where their rules read alike, and their plain texts where
    /// one reads as written and the other doesn't (`TLMD_287`).
    fn compared<'a>(&'a self, other: &'a KeyText) -> (&'a str, &'a str) {
        if self.verbatim == other.verbatim {
            (&self.read, &other.read)
        } else {
            (self.plain(), other.plain())
        }
    }

    /// Whether `self` and `other` are one key (`TLMD_287`).
    // sokf-implements: TLMD_287
    // sokf-implements: MD-4_AC-21
    pub(crate) fn same(&self, other: &KeyText) -> bool {
        let (a, b) = self.compared(other);
        a == b
    }

    /// The longest of `parents` that `self` starts with, followed by the
    /// separator of `kd`, each compared as `same` compares two keys
    /// (`TL_33`, `TL_126`, `TLMD_287`), through `KeyDerivation::prefix_of`.
    // sokf-implements: TLMD_287
    pub(crate) fn parent_in<'a>(
        &self,
        kd: &KeyDerivation,
        parents: &'a [KeyText],
    ) -> Option<&'a KeyText> {
        let matching: Vec<&KeyText> = parents
            .iter()
            .filter(|p| {
                let (key, prefix) = self.compared(p);
                kd.prefix_of(key, [prefix]).is_some()
            })
            .collect();
        // The longest is measured in one text for all: the reading where
        // every parent reads as the key does, else the plain text.
        let alike = matching.iter().all(|p| p.verbatim == self.verbatim);
        matching
            .into_iter()
            .max_by_key(|p| if alike { p.read.len() } else { p.plain().len() })
    }
}

/// Two keys are equal where their texts and their readers are, whichever
/// of their other texts were read.
impl PartialEq for KeyText {
    fn eq(&self, other: &KeyText) -> bool {
        (&self.read, &self.written, self.verbatim) == (&other.read, &other.written, other.verbatim)
    }
}

impl Eq for KeyText {}

/// The key as `text` writes it: the bytes of `text` that the bytes `at` of
/// `squashed`, the squashed `reading` of `text` mapped to it, stand for, or
/// `key`, the key as read, where `at` names none.
pub(crate) fn written_key(
    text: &str,
    reading: &markdown::Reading,
    squashed: &Squashed,
    at: Option<Range<usize>>,
    key: &str,
) -> String {
    at.and_then(|at| {
        let span = reading.span_of(text, squashed.original(at.start), squashed.original(at.end));
        text.get(span)
    })
    .unwrap_or(key)
    .to_string()
}

/// The key that `shape`, a shape as read, captures as `name` from `text`,
/// a text of the schema that `read` reads, with its text as written, of a
/// rule that reads its text as written where `verbatim`: as the check
/// captures a key from a unit (`TLMD_259`).
fn key_of_text(
    text: &str,
    read: fn(&str) -> markdown::Reading,
    verbatim: bool,
    key: impl Fn(&str) -> Option<(String, Option<Range<usize>>)>,
    cut: impl Fn(&Squashed) -> Squashed,
) -> Option<KeyText> {
    let reading = read(text);
    let squashed = cut(&Squashed::mapped(&reading.text, &reading.map));
    let (got, at) = key(&squashed.text)?;
    let written = written_key(text, &reading, &squashed, at, &got);
    Some(KeyText::new(&got, &written, verbatim))
}

impl Section {
    /// The key that the heading's example captures, as the check captures
    /// the key of a heading: in its reading as a paragraph alone, with the
    /// bytes of the example that it stands for (`TLMD_259`).
    pub(crate) fn example_key(&self) -> Option<KeyText> {
        let (name, example) = (self.heading.key.as_deref()?, self.heading.example.as_ref()?);
        let shape = &self.heading.read;
        key_of_text(
            example,
            markdown::reading,
            false,
            |t| {
                let at = shape.captures(t).and_then(|c| Some(c.name(name)?.range()));
                Some((captured(shape, name, t)?.key, at))
            },
            Squashed::clone,
        )
    }
}

impl Schema {
    /// The key that `rule`, a unit rule of `block`, captures from `text`, an
    /// example as written, as the check captures it from a unit: in the
    /// reading of the rule, without its label in `words`, with the bytes of
    /// the example that it stands for (`TLMD_248`, `TLMD_259`).
    pub(crate) fn example_key(
        &self,
        words: &Words,
        block: &Block,
        rule: &Unit,
        text: &str,
    ) -> Option<KeyText> {
        let formats = self.formats_of(rule);
        key_of_text(
            text,
            rule.reader(),
            !rule.reads_prose,
            |t| Some((rule.key_in(formats, t)?, rule.key_span_in(formats, t))),
            |squashed| match self.label_in(words, block, Some(rule), &squashed.text) {
                Some(found) => without(squashed, &found.at),
                None => squashed.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prose(read: &str, written: &str) -> KeyText {
        KeyText::new(read, written, false)
    }

    fn fence(key: &str) -> KeyText {
        KeyText::new(key, key, true)
    }

    // sokf-tests: TLMD_287
    #[test]
    fn keys_of_two_readings_compare_by_their_plain_texts() {
        // `a*b*` of prose reads `a_b_` and renders `ab`, as `a*b*` of a
        // fence does read as prose.
        assert!(prose("a_b_", "a*b*").same(&fence("a*b*")));
        assert!(fence("a*b*").same(&prose("a_b_", "a*b*")));
        assert!(prose("ab", "ab").same(&fence("a*b*")));
        // `a_b_` with its underscores inside a word renders them.
        assert!(!prose("a_b_", "a_b_").same(&fence("a*b*")));
        assert!(prose("a_b_", "a_b_").same(&fence("a_b_")));
        // Two keys of rules that read alike compare by their readings.
        assert!(prose("a_b_", "a*b*").same(&prose("a_b_", "a_b_")));
        assert!(!prose("ab", "ab").same(&prose("a_b_", "a*b*")));
        assert!(!fence("ab").same(&fence("a*b*")));
    }

    #[test]
    fn a_key_that_cuts_markup_in_two_is_spelt_as_read() {
        let cut = prose("_a-1", "*a-1");
        assert_eq!(cut.spelt(), "_a-1");
        assert_eq!(cut.plain(), "_a-1");
        assert_eq!(prose("a_b_-1", "a*b*-1").spelt(), "a*b*-1");
    }

    // sokf-tests: TLMD_287
    #[test]
    fn a_key_derives_from_a_parent_of_another_reading_by_plain_text() {
        let kd = KeyDerivation {
            source: tramlines_core::grammar::DerivationSource::Rule("p".into()),
            separator: "-".into(),
        };
        let parents = [fence("a*b*"), fence("a")];
        let key = prose("a_b_-1", "a*b*-1");
        assert_eq!(key.parent_in(&kd, &parents), Some(&parents[0]));
        assert_eq!(prose("c_d_-1", "c*d*-1").parent_in(&kd, &parents), None);
        let parents = [prose("a_b_", "a*b*")];
        assert_eq!(fence("a*b*-1").parent_in(&kd, &parents), Some(&parents[0]));
        assert_eq!(fence("a_b_-1").parent_in(&kd, &parents), None);
    }

    // sokf-tests: TL_126
    #[test]
    fn the_longest_of_parents_of_two_readings_is_read_in_one_text() {
        let kd = KeyDerivation {
            source: tramlines_core::grammar::DerivationSource::Rule("p".into()),
            separator: "-".into(),
        };
        let parents = [fence("*a*-b"), prose("_a_", "*a*")];
        let key = prose("_a_-b-1", "*a*-b-1");
        assert_eq!(key.parent_in(&kd, &parents), Some(&parents[0]));
    }
}
