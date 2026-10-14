//! What a schema check yields: the findings, the captured keys and their
//! anchors, the repairs, and the gates.

use crate::{
    Finding,
    grammar::{KeyDerivation, KeyScope, Only},
};

/// A key a rule captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// The key.
    pub key: String,
    /// The line it is captured on.
    pub line: usize,
    /// The heading's slug, when a heading's `key` captured it.
    pub heading_slug: Option<String>,
    /// `unique`.
    pub scope: KeyScope,
    /// The `rule-id` of the capturing rule.
    pub rule_id: Option<String>,
    /// The declared section the capture lies in.
    pub section: Option<usize>,
    /// The pairing the capturing rule declares, where it declares one.
    pub pairs: Option<Paired>,
    /// Whether the capturing rule declares `key-repeats`.
    pub repeats: bool,
    /// Where the key's anchor goes, when this capture is the one that
    /// places it (TLMD_104 to TLMD_109).
    pub spot: Spot,
    /// The prefix the key takes, where the capturing rule derives it.
    pub derives: Option<KeyDerivation>,
}

/// Where a key's anchor goes (TLMD_104 to TLMD_109).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// At the start of the text of the heading on this line.
    Heading(usize),
    /// Before this byte column of this line, anchors removed.
    Inline(usize, usize),
    /// On a line of its own before the block that starts on this line.
    Before(usize),
}

impl Spot {
    /// The line a finding about the anchor names.
    pub fn line(self) -> usize {
        match self {
            Spot::Heading(l) | Spot::Inline(l, _) | Spot::Before(l) => l,
        }
    }
}

/// The anchor one key needs: its id, the key, and where it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// The key's slug.
    pub id: String,
    /// The key.
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

/// The rules of a pairing as a finding names them: "rule `a`", "rule `a`
/// or `b`", or "rule `a`, `b`, or `c`".
pub(crate) fn rules_named(with: &[String]) -> String {
    let quoted: Vec<String> = with.iter().map(|r| format!("`{r}`")).collect();
    match quoted.as_slice() {
        [] => "rule".to_string(),
        [one] => format!("rule {one}"),
        [a, b] => format!("rule {a} or {b}"),
        [rest @ .., last] => format!("rule {}, or {last}", rest.join(", ")),
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
    /// A heading of `level`.
    pub fn heading(line: usize, end_line: usize, level: u8, text: String) -> Self {
        Repair {
            line,
            end_line,
            prefix: format!("{} ", "#".repeat(usize::from(level))),
            text,
        }
    }
}

// sokf:begin check-output
/// A gated rule that matched a part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    /// The line of the part.
    pub line: usize,
    /// The rule's `only`.
    pub only: Only,
    /// The rule's line in the schema.
    pub rule: usize,
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
    /// Each gated rule that matched a part: what an `only` marker there
    /// must agree with.
    pub gates: Vec<Gate>,
    /// The anchor each key needs, in the order the keys were captured.
    pub anchors: Vec<Placement>,
    /// Whether the anchors the body holds differ from `anchors`.
    pub anchors_stale: bool,
}
// sokf:end check-output
