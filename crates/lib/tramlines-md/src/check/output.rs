//! What a schema check yields: the findings, the captured keys and their
//! anchors, the repairs, and the gates.

use std::borrow::Cow;

use crate::{
    Finding,
    grammar::{Condition, KeyDerivation, KeyScope, KeyText},
    markdown::{self, id_of, slug},
};

/// A key a rule captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// The key, as the rule reads it: in the reading of its unit where the
    /// unit is prose or a heading (`TLMD_258`).
    pub key: String,
    /// The key as the document writes it: the bytes of the unit's text
    /// that `key` stands for, which the id of its anchor reads.
    pub written: String,
    /// The line it is captured on.
    pub line: usize,
    /// The heading's slug, when a heading's `key` captured it.
    pub heading_slug: Option<String>,
    /// `unique`.
    pub scope: KeyScope,
    /// The name that the key is filed under: the `key-of` of the
    /// capturing rule, else the `rule-id` of its block, its group, or its
    /// section. No key is filed under a unit rule's own `rule-id`
    /// (`TLMD_236`).
    pub rule_id: Option<String>,
    /// The `rule-id` that names the capturing rule in a finding: the
    /// rule's own, else its block's, its group's, or its section's.
    pub rule_name: Option<String>,
    /// The declared section the capture lies in.
    pub section: Option<usize>,
    /// The pairing the capturing rule declares, where it declares one.
    pub pairs: Option<Paired>,
    /// Whether the capturing rule declares `key-repeats`.
    pub repeats: bool,
    /// Where the key's anchor goes, when this capture is the one that
    /// places it (`TLMD_104` to `TLMD_109`).
    pub spot: Spot,
    /// The prefix the key takes, where the capturing rule derives it.
    pub derives: Option<KeyDerivation>,
    /// The id of the anchor that names the capture's key, which a link to
    /// the key opens (`TLMD_101` to `TLMD_106`, `TLMD_287`): the id that
    /// the capture which holds the key's anchor gives, the heading that
    /// captures the key, a heading that pairs with it among them, else the
    /// capture that defines it, else the key's first capture. A capture that
    /// pairs names the key of its set. The check sets it after every key is
    /// captured, also where the body holds no such anchor yet, where it is
    /// the id that the repair writes, and on a scaffold; it is empty until
    /// then, and where the key's slug is empty.
    pub anchor: String,
}

/// Where a key's anchor goes (`TLMD_104` to `TLMD_109`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// At the start of the text of the heading on this line.
    Heading(usize),
    /// Before this byte column of this line, anchors removed.
    Inline(usize, usize),
    /// On a line of its own before the block that starts on this line.
    Before(usize),
}

impl Capture {
    /// The key as the comparison of two keys reads it (`TLMD_287`): as
    /// written where a line of a fence or of an HTML block, which reads as
    /// written, captured it, and else in its reading, with the text that
    /// the document writes for it.
    pub(crate) fn text(&self) -> KeyText {
        KeyText::new(
            &self.key,
            &self.written,
            matches!(self.spot, Spot::Before(_)),
        )
    }

    /// The id of the key's anchor: the slug of the plain text that the key
    /// renders as where a unit of prose or a heading captured it, and the
    /// slug of the key itself where a line of a fence or of an HTML block,
    /// which reads as written, did (`TLMD_101`). The plain text is that of
    /// the key as written, where the key alone reads as it reads in its
    /// unit, because the reading writes an emphasis inside a word `_x_`,
    /// which renders as no emphasis, so `a*b*-1` gives `ab-1`. A key that
    /// cuts a piece of markup in two reads otherwise alone, and its plain
    /// text is that of its reading, which its twins share.
    // sokf-implements: TLMD_101
    // sokf-implements: MD-4_AC-19
    pub fn id(&self) -> String {
        let text = self.text();
        if text.verbatim {
            slug(&self.key)
        } else {
            id_of(text.spelt())
        }
    }
}

impl Spot {
    /// The line a finding about the anchor names.
    pub fn line(self) -> usize {
        match self {
            Spot::Heading(l) | Spot::Inline(l, _) | Spot::Before(l) => l,
        }
    }
}

/// The anchor one key needs, or one that a unit which breaks its rule
/// holds (`TLMD_139`): its id, the key, and where it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// The key's slug.
    pub id: String,
    /// The key, or the id of an anchor that a unit which breaks its rule
    /// holds, because that unit captures no key.
    pub key: String,
    /// Where it goes.
    pub spot: Spot,
}

/// A `for-each-key-of`: the rules a capture's key pairs with, and the rule
/// that declares the pairing, by its line in the schema. Two rules of one
/// section may pair with the same rules, and each pairs on its own.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Paired {
    /// The rules whose keys this key pairs with.
    pub with: Vec<String>,
    /// The schema line of the rule that declares the pairing.
    pub by: usize,
}

/// The sets of keys of a pairing as a finding names them: "set `a`", "set
/// `a` or `b`", or "set `a`, `b`, or `c`". A set is named by the `rule-id`
/// of a section or a block, or by a `key-of`, which is no rule.
pub(crate) fn rules_named(with: &[String]) -> String {
    let quoted: Vec<String> = with.iter().map(|r| format!("`{r}`")).collect();
    match quoted.as_slice() {
        [] => "set".to_string(),
        [one] => format!("set {one}"),
        [a, b] => format!("set {a} or {b}"),
        [rest @ .., last] => format!("set {}, or {last}", rest.join(", ")),
    }
}

/// A line range a repair rewrites as one line: a heading, or a paragraph
/// or item bound to a `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repair {
    /// The first line.
    pub line: usize,
    /// The last line: the underline of a setext heading, the last line
    /// of a wrapped paragraph.
    pub end_line: usize,
    /// What precedes the text on the new line: `## ` for a heading,
    /// `- ` for an item, `> ` inside a blockquote, nothing for a
    /// paragraph.
    pub prefix: String,
    /// The text.
    pub text: String,
}

impl Repair {
    /// A heading of `level`, its text written so that the parser reads it
    /// back as `text`.
    // sokf-implements: TLMD_274
    pub fn heading(line: usize, end_line: usize, level: u8, text: String) -> Self {
        Repair {
            line,
            end_line,
            prefix: format!("{} ", "#".repeat(usize::from(level))),
            text: match markdown::atx_written(&text) {
                Cow::Borrowed(_) => text,
                Cow::Owned(written) => written,
            },
        }
    }
}

// sokf:begin check-output
/// Text of one line that a repair rewrites where it stands: a label
/// written as its key, rewritten in the words that the host gives for it
/// (`TLMD_254`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRepair {
    /// The line.
    pub line: usize,
    /// The byte column in the line, anchors removed, where `old` starts.
    pub col: usize,
    /// The text as written.
    pub old: String,
    /// The text that replaces it.
    pub new: String,
}

/// A gated rule that matched a part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    /// The line of the part.
    pub line: usize,
    /// The rule's `only`.
    pub only: Condition,
    /// The rule's line in the schema.
    pub rule: usize,
}

/// A paragraph, an item, or a row that a unit rule took and whose shape it
/// has: the unit's lines, and the rule, by its `Unit::index`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accepted {
    /// The first line of the unit.
    pub line: usize,
    /// The last line of the unit: the last line of a wrapped paragraph.
    pub end_line: usize,
    /// The rule's `Unit::index`, which `Schema::unit` finds the rule by.
    pub rule_index: usize,
    /// The rule's `rule-id`, where it declares one.
    pub rule_id: Option<String>,
}

/// What a schema check yields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckOutput {
    /// The findings.
    pub findings: Vec<Finding>,
    /// The captured keys.
    pub captures: Vec<Capture>,
    /// The line rewrites: headings and bound units.
    pub repairs: Vec<Repair>,
    /// The rewrites of text where it stands: each label written as its
    /// key where the host gives other words for it.
    pub label_repairs: Vec<LabelRepair>,
    /// Each gated rule that matched a part: what an `only` marker there
    /// must agree with.
    pub gates: Vec<Gate>,
    /// Each unit a unit rule accepted, in the order of the units' first
    /// lines in the document, where `Host::records_accepted` asks for them;
    /// otherwise none.
    pub accepted: Vec<Accepted>,
    /// The anchor each key needs, in the order the keys were captured,
    /// then each anchor that a unit which breaks its rule holds, where it
    /// stands.
    pub anchors: Vec<Placement>,
    /// Whether the anchors the body holds differ from `anchors`.
    pub anchors_stale: bool,
}
// sokf:end check-output
