//! The body a schema implies: the opening guidance, the preamble, the
//! sections in order, and the blocks of each, with the `only` marker
//! every gated rule needs and the `guidance` line every rule that
//! carries guidance writes.

use std::collections::BTreeMap;

use super::{
    Deriver,
    text::{self, KeyReader},
};
use crate::{
    check,
    grammar::{Block, RuleCommon, Section, Shape, Text, squash},
    markdown::Dialect,
};

/// How a section's heading reads its key, where it declares one.
fn heading_reader(section: &Section) -> Option<KeyReader> {
    KeyReader::of(&section.heading.shape, section.heading.key.as_deref()?)
}

/// The sentence every scaffold opens with, in the voice of the document. The
/// scaffold is made of examples, and a model that fills it has to know that
/// before it reads one.
const OPENING: &str = "The writer replaces each of the following examples with their own text.";

/// One `guidance` line. A marker is recognised a line at a time, so the
/// text is joined onto one line whatever its length.
fn guidance_line(dialect: &Dialect, text: &str) -> String {
    format!(
        "<!-- {} {} -->",
        dialect.spell("guidance"),
        squash(text).replace("-->", "- ->")
    )
}

impl Deriver<'_> {
    /// The body, heading by heading.
    // sokf-implements: TLMD_114
    pub(super) fn body(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        // The scaffold says what it is before it says anything else, then
        // what the schema says that no example can.
        // sokf-implements: MD-7_AC-3
        // sokf-implements: TLMD_118
        let mut opening = match &self.schema.core.guidance {
            Some(g) => format!("{OPENING} {g}"),
            None => OPENING.to_string(),
        };
        // sokf-implements: TLMD_195
        for message in self.schema_bans() {
            opening.push(' ');
            opening.push_str(&message);
        }
        out.push(guidance_line(&self.dialect, &opening));
        out.push(String::new());
        let schema = self.schema;
        if let Some(p) = &schema.preamble
            && p.common.only.admits(self.states)
        {
            self.guidance(p.guidance.as_deref(), &mut out);
            self.blocks(p.blocks.as_deref(), &mut out);
        }
        // A section with a `parent` is written inside each occurrence of
        // its parent, so the walk here leaves it to the parent.
        // Each optional section is a region of its own, because a writer
        // decides on each section apart from the next.
        // sokf-implements: MD-7_AC-4
        // sokf-implements: TLMD_192
        for section in schema.sections.iter().filter(|s| s.parent.is_none()) {
            if section.optional && !self.in_optional {
                self.optional_region(&mut out, |d, out| d.section(section, out));
            } else {
                self.section(section, &mut out);
            }
        }
        out
    }

    /// The `guidance` line a part with guidance carries, written before
    /// the part and before its `only` marker, so that marker still
    /// sits on the line it marks.
    // sokf-implements: MD-7_AC-3
    // sokf-implements: TLMD_119
    pub(super) fn guidance(&self, guidance: Option<&str>, out: &mut Vec<String>) {
        if let Some(g) = guidance {
            out.push(guidance_line(&self.dialect, g));
        }
    }

    /// The `only` comment line a gated rule's part carries.
    // sokf-implements: MD-7_AC-4
    // sokf-implements: TLMD_121
    pub(super) fn marker(&self, common: &RuleCommon, out: &mut Vec<String>) {
        if let Some(gate) = self.gate_text(common) {
            out.push(format!("<!-- {gate} -->"));
        }
    }

    fn section(&mut self, section: &Section, out: &mut Vec<String>) {
        if !section.common.only.admits(self.states) {
            return;
        }
        // An optional section is written where the derivation can write
        // it, because a scaffold is a prompt sheet and an author who made
        // a section optional still meant a reader to see it. Where a rule
        // inside offers nothing to write, the section is left out rather
        // than refused over, and the author is told which rule kept it out.
        self.attempt(section.optional, out, |d, out| d.write_one(section, out));
    }

    fn write_one(&mut self, section: &Section, out: &mut Vec<String>) {
        // A section demanded once per key of another rule is written once
        // per key that rule captured, its heading spliced to carry the key.
        // The occurrence of each key after the first is the next text of
        // `examples`, where there is one, so that each occurrence has its
        // own text rather than a copy of the first.
        if let Some(rule_ids) = &section.for_each_key_of {
            let mut texts = section.examples.iter();
            for (n, key) in self.keys_in_scope(rule_ids).into_iter().enumerate() {
                if n > 0
                    && let Some(text) = texts.next()
                {
                    self.occurrence(section, text, out);
                    continue;
                }
                let Some(heading) = self.paired_heading(section, &key) else {
                    continue;
                };
                self.write_section(section, &heading, out);
            }
            return;
        }
        // A heading the derivation cannot write is a refusal, and the
        // section is walked anyway: every rule inside it has something to
        // say, and an author who fixes one refusal should not have to run
        // the tool again to meet the next.
        let mut heading = self.heading(section).unwrap_or_default();
        // A heading key that derives takes the prefix the derivation allows,
        // as a unit key does.
        if let (Some(kd), Some(reader)) =
            (&section.heading.key_derives_from, heading_reader(section))
        {
            heading = self.derive_with(kd, &reader, heading, section.common.line);
        }
        self.write_section(section, &heading, out);
        // The `examples` of a repeating section follow the occurrence that
        // the derivation wrote, each written whole and unchanged after the
        // section's `only` marker, because each occurrence is a part that
        // the gate of the section covers. The key that each heading
        // captures is recorded, as `write_section` records it, so that a
        // later rule that pairs with the section or derives from it sees
        // every occurrence.
        for text in &section.examples {
            self.occurrence(section, text, out);
        }
    }

    /// One text of the `examples` of a section, written as a whole
    /// occurrence of the section, unchanged and after the section's `only`
    /// marker, with the key that its heading captures recorded.
    fn occurrence(&mut self, section: &Section, text: &str, out: &mut Vec<String>) {
        let text = text.trim_matches(|c| c == '\n' || c == '\r');
        if let (Some(id), Some(reader)) = (&section.rule_id, heading_reader(section))
            && let Some(first) = text.lines().next()
            && let Some(key) = text::capture_of(&reader, first.trim_start_matches('#').trim())
        {
            self.record(Some(id), Some(key));
        }
        self.marker(&section.common, out);
        out.extend(text.lines().map(str::to_string));
        out.push(String::new());
    }

    fn write_section(&mut self, section: &Section, heading: &str, out: &mut Vec<String>) {
        // Where this occurrence begins, for the sections inside it.
        let span = self.key_counts();
        let held = std::mem::replace(&mut self.section_rule, section.rule_id.clone());
        // A section that reads its heading as a key records it, as the
        // checker does, so a rule deriving from it has a prefix to take.
        if let Some(reader) = heading_reader(section)
            && let Some(key) = text::capture_of(&reader, heading)
        {
            self.record(section.rule_id.as_ref(), Some(key));
        }
        self.guidance(section.guidance.as_deref(), out);
        self.marker(&section.common, out);
        out.push(format!(
            "{} {heading}",
            "#".repeat(usize::from(section.level))
        ));
        out.push(String::new());
        match &section.blocks {
            // An absent `blocks` means one prose block of one or more
            // paragraphs, which the schema gives no text for; `[]` means
            // an empty body. The scaffold writes no text of its own.
            // sokf-implements: TLMD_115
            None => self.refuse(
                section.common.line,
                "a section with no `blocks` holds one prose block, and the scaffold writes no text of its own; declare the block and its `example`",
            ),
            Some(blocks) => self.blocks(Some(blocks), out),
        }
        self.section_rule = held;
        self.children(section, span, out);
    }

    /// The sections that lie inside this occurrence of `section`, each
    /// reading the keys captured since the occurrence began. One that pairs
    /// is written where it has keys to pair and left out where it has none:
    /// the pairing demands it or bans it, so it is never an optional region.
    // sokf-implements: MD-7_AC-5
    // sokf-implements: TLMD_128
    // sokf-implements: TLMD_130
    fn children(
        &mut self,
        section: &Section,
        span: BTreeMap<String, usize>,
        out: &mut Vec<String>,
    ) {
        let Some(id) = &section.rule_id else {
            return;
        };
        let schema = self.schema;
        for child in schema
            .sections
            .iter()
            .filter(|s| s.parent.as_ref() == Some(id))
        {
            let held = self.scope.replace(span.clone());
            match self.pairs(child) {
                Some(true) => {
                    if child.common.only.admits(self.states) {
                        self.write_one(child, out);
                    }
                }
                Some(false) => {}
                None if child.optional && !self.in_optional => {
                    self.optional_region(out, |d, out| d.section(child, out));
                }
                None => self.section(child, out),
            }
            self.scope = held;
        }
    }

    /// Whether a section's pairings have keys to pair in the scope being
    /// written, or `None` where nothing in it pairs.
    fn pairs(&self, section: &Section) -> Option<bool> {
        let mut withs: Vec<Vec<String>> = section.for_each_key_of.iter().cloned().collect();
        let mut stack: Vec<&Block> = section.blocks.iter().flatten().collect();
        while let Some(b) = stack.pop() {
            stack.extend(b.blocks.iter().flatten());
            for u in b.units() {
                withs.extend(u.for_each_key_of.clone());
                withs.extend(
                    u.lines
                        .iter()
                        .flatten()
                        .filter_map(|l| l.for_each_key_of.clone()),
                );
                stack.extend(u.blocks.iter().flatten());
            }
        }
        if withs.is_empty() {
            return None;
        }
        Some(withs.iter().any(|w| !self.keys_in_scope(w).is_empty()))
    }

    /// The heading text a section demands.
    fn heading(&mut self, section: &Section) -> Option<String> {
        match &section.heading.shape {
            Shape::Text(Text::Literal(t)) => Some(t.clone()),
            // sokf-implements: TLMD_155
            Shape::Text(Text::From(key)) => self.values.get(key).cloned().or_else(|| {
                self.refuse_on(
                    section.common.line,
                    "TLMD_155",
                    format!(
                        "a section takes its heading from `{key}`, which no frontmatter writes"
                    ),
                );
                None
            }),
            shape => match &section.heading.example {
                Some(e) => Some(e.clone()),
                None => {
                    self.refuse(
                        section.common.line,
                        format!(
                            "a section's heading `{}` varies and carries no `example`; write one",
                            shape.label()
                        ),
                    );
                    None
                }
            },
        }
    }

    /// The heading of a paired section, its own key spliced out for the
    /// key the rule it pairs with captured.
    fn paired_heading(&mut self, section: &Section, key: &str) -> Option<String> {
        let example = self.heading(section)?;
        let Some(reader) = heading_reader(section) else {
            return Some(example);
        };
        match text::respell(&reader, &example, key) {
            Some(spliced) => Some(spliced),
            // sokf-implements: TLMD_154
            None => {
                self.refuse_on(
                    section.common.line,
                    "TLMD_154",
                    format!(
                        "a section demands one heading per key of {}, and its example `{example}` does not carry `{key}` where `{}` reads it",
                        check::rules_named(section.for_each_key_of.as_deref().unwrap_or_default()),
                        reader.name
                    ),
                );
                None
            }
        }
    }
}
