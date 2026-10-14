//! The units of a block in the scaffold: the text each rule writes for a
//! unit, the keys those texts capture, and the lines of a fence.

use super::Deriver;
use crate::{
    grammar::{Block, Shape, Unit},
    markdown::BlockKind,
};

impl Deriver<'_> {
    /// The text of every unit a block demands, and the keys they capture.
    /// A rule whose `example` is a list writes one unit for each text of the
    /// list, in order, and no more units than the `max` of the rule or of
    /// the block allows.
    pub(super) fn units_of(&mut self, block: &Block) -> Vec<String> {
        let cap = block.max.unwrap_or(usize::MAX);
        let mut out: Vec<String> = Vec::new();
        if let Some(sequence) = &block.sequence {
            for rule in sequence {
                if rule.optional {
                    continue;
                }
                // A position that repeats writes every text of its list, and
                // any other position holds one unit.
                let n = if rule.repeat {
                    listed(Some(rule), block).max(1)
                } else {
                    1
                };
                for i in 0..n {
                    if i > 0 && out.len() >= cap {
                        break;
                    }
                    let text = self.unit_of(block, Some(rule), i);
                    out.push(text);
                }
            }
        } else if let Some(rules) = &block.rules {
            for rule in rules
                .iter()
                .filter(|r| !r.is_counter() || !r.examples.is_empty())
            {
                if let Some(paired) = self.paired_units(block, rule) {
                    for text in paired {
                        self.record_capture(block, rule, &text);
                        out.push(text);
                    }
                    continue;
                }
                // A guarded rule governs the units of earlier rules that its
                // guard selects: it reads its keys out of them, and they count
                // toward what it writes (TLMD_199).
                // sokf-implements: TLMD_199
                let selected: Vec<String> = match &rule.select {
                    Some(guard) => out.iter().filter(|u| guard.is_match(u)).cloned().collect(),
                    None => Vec::new(),
                };
                for text in &selected {
                    self.record_capture(block, rule, text);
                }
                // A rule writes the units that its `min` demands, and the
                // further texts of a list while the block has room for them.
                let n = wanted(rule, block).saturating_sub(selected.len());
                for i in 0..n {
                    if i >= rule.min && out.len() >= cap {
                        break;
                    }
                    let text = self.unit_of(block, Some(rule), i);
                    out.push(text);
                }
            }
            // A block whose guards demand nothing still holds the units the
            // block itself demands; the first rule writes them.
            let mut i = 0;
            while out.len() < block.min {
                let text = self.unit_of(block, rules.first(), i);
                out.push(text);
                i += 1;
            }
        } else if let Some(rule) = &block.each
            && let Some(paired) = self.paired_units(block, rule)
        {
            for text in paired {
                self.record_capture(block, rule, &text);
                out.push(text);
            }
            return out;
        } else {
            let each = block.each.as_ref();
            let n = block.min.max(1).max(listed(each, block)).min(cap);
            for i in 0..n {
                let text = self.unit_of(block, each, i);
                out.push(text);
            }
        }
        if out.is_empty() && block.kind != BlockKind::ThematicBreak {
            let text = self.unit_of(block, None, 0);
            out.push(text);
        }
        out
    }

    /// The text of the unit at position `i` among the units that `rule`
    /// writes, with its key recorded and its nested rules written where the
    /// kind carries them.
    fn unit_of(&mut self, block: &Block, rule: Option<&Unit>, i: usize) -> String {
        if let Some(r) = rule
            && block.kind == BlockKind::Code
            && let Some(lines) = &r.lines
        {
            // The example takes precedence. Built a line at a time, a
            // diagram of five classes is five copies of one node: one
            // line per rule, repeated to meet its `min`. The line rules
            // read their keys out of the example instead, as the check
            // of the scaffold does.
            // sokf-implements: MD-7_AC-5
            // sokf-implements: TLMD_126
            // sokf-implements: TLMD_127
            if !r.examples.is_empty() || !block.examples.is_empty() {
                let text = self.unit_text_at(block, rule, i);
                self.record_fence_keys(block, lines, &text);
                return text;
            }
            let text = self.fence_lines(block, lines);
            if !text.is_empty() {
                return text;
            }
        }
        let mut text = self.unit_text_at(block, rule, i);
        if let Some(r) = rule {
            text = self.derived_key(r, text);
            self.record_capture(block, r, &text);
            // A list item carries its nested blocks beyond its own text,
            // indented to the column of that text.
            // sokf-implements: MD-7_AC-5
            // sokf-implements: TLMD_131
            let indent = match block.kind {
                BlockKind::BulletList => Some(2),
                BlockKind::NumberedList => Some(3),
                _ => None,
            };
            if let (Some(indent), Some(inner)) = (indent, r.blocks.as_deref()) {
                // The nested blocks are written apart, so they meet only
                // one another, and the blocks around the list never see them.
                let mut nested = Vec::new();
                let around = std::mem::take(&mut self.kinds);
                self.blocks(Some(inner), &mut nested);
                let first = self.kinds.first().map(|(kind, ..)| *kind);
                self.kinds = around;
                while nested.last().is_some_and(String::is_empty) {
                    nested.pop();
                }
                // A blank line ends the paragraph of the item, which a
                // nested paragraph would otherwise continue. A nested list
                // starts on the next line, as a sublist is written, and
                // keeps the list tight.
                // sokf-implements: MD-7_AC-5
                // sokf-implements: TLMD_150
                let list = matches!(first, Some(BlockKind::BulletList | BlockKind::NumberedList));
                if !nested.is_empty() && !list {
                    text.push('\n');
                }
                for line in nested {
                    text.push('\n');
                    if !line.is_empty() {
                        text.push_str(&" ".repeat(indent));
                        text.push_str(&line);
                    }
                }
            }
        }
        text
    }

    /// The keys the line rules read out of a fence the schema states,
    /// one line at a time, as the checker reads them.
    // sokf-implements: TLMD_127
    fn record_fence_keys(&mut self, block: &Block, rules: &[Unit], text: &str) {
        for rule in rules {
            for line in text.lines().map(str::trim_start) {
                if rule.select.as_ref().is_none_or(|s| s.is_match(line)) {
                    self.record_capture(block, rule, line);
                }
            }
        }
    }

    /// The content of a fence its unit rule governs line by line: each line
    /// rule writes the lines it demands.
    fn fence_lines(&mut self, block: &Block, rules: &[Unit]) -> String {
        let mut out: Vec<String> = Vec::new();
        // A counter counts what the other rules write, and writes nothing
        // of its own unless it states the line it counts (TLMD_52 to TLMD_57).
        for rule in rules
            .iter()
            .filter(|r| !r.is_counter() || !r.examples.is_empty())
        {
            let texts = match self.paired_units(block, rule) {
                Some(paired) => paired,
                None => (0..wanted(rule, block))
                    .map(|i| self.unit_text_at(block, Some(rule), i))
                    .collect(),
            };
            for text in texts {
                // A line rule reads a line without its indent (TLMD_6).
                self.record_capture(block, rule, text.trim_start());
                out.push(text);
            }
        }
        out.join("\n")
    }
}

/// How many units `rule` writes: what its `min` demands and its `example`
/// list gives, up to its `max`.
fn wanted(rule: &Unit, block: &Block) -> usize {
    rule.min
        .max(listed(Some(rule), block))
        .min(rule.max.unwrap_or(usize::MAX))
}

/// The number of texts that the `example` list of a rule gives its units,
/// or zero where the text that the rule writes is not a list. It follows
/// the order in which `unit_texts` looks for a text: the rule's `text`, its
/// `example`, its format's example, and the block's `example`.
fn listed(rule: Option<&Unit>, block: &Block) -> usize {
    match rule {
        Some(r) if matches!(r.shape, Some(Shape::Text(_))) => 0,
        Some(r) if !r.examples.is_empty() => {
            if r.example_list {
                r.examples.len()
            } else {
                0
            }
        }
        Some(r) if r.format.is_some() => 0,
        _ if block.example_list => block.examples.len(),
        _ => 0,
    }
}
