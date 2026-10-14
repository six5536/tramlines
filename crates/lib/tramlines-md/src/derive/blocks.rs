//! The blocks of a section in the scaffold: each block's units, its
//! guidance and its marker, and the refusal of a block that the next one
//! would join.

use super::Deriver;
use crate::{
    grammar::{Block, Lang},
    markdown::BlockKind,
    text,
};

impl Deriver<'_> {
    pub(super) fn blocks(&mut self, rules: Option<&[Block]>, out: &mut Vec<String>) {
        let rules = rules.unwrap_or(&[]);
        let mut i = 0;
        while i < rules.len() {
            // A run of optional blocks is one region: a label and the list
            // under it are one decision, not two.
            if rules[i].optional && !self.in_optional {
                let mut j = i;
                while j < rules.len() && rules[j].optional {
                    j += 1;
                }
                self.optional_region(out, |d, out| {
                    for block in &rules[i..j] {
                        d.one_block(block, out);
                    }
                });
                i = j;
                continue;
            }
            self.one_block(&rules[i], out);
            i += 1;
        }
    }

    fn one_block(&mut self, block: &Block, out: &mut Vec<String>) {
        // A block the states exclude is not this scaffold's, and one
        // whose `max` admits no unit bans rather than demands.
        if !block.common.only.admits(self.states) || block.max == Some(0) {
            return;
        }
        // An optional block is written where the derivation can write it,
        // and left out where a rule inside it offers nothing, as an
        // optional section is.
        self.attempt(block.optional, out, |d, out| d.block(block, out));
    }

    fn block(&mut self, block: &Block, out: &mut Vec<String>) {
        // The messages of the block's bans follow its guidance.
        // sokf-implements: TLMD_195
        let bans = self.block_bans(block);
        let guidance = block
            .guidance
            .iter()
            .cloned()
            .chain(bans)
            .collect::<Vec<_>>()
            .join(" ");
        self.guidance((!guidance.is_empty()).then_some(guidance.as_str()), out);
        if block.kind == BlockKind::Group {
            for _ in 0..block.min.max(1) {
                self.marker(&block.common, out);
                self.blocks(block.blocks.as_deref(), out);
            }
            return;
        }
        let units = self.units_of(block);
        let units_count = units.len();
        // Each fence of a code block is a block of its own, so each one
        // carries its own `only` marker, which the match below writes.
        if block.kind != BlockKind::Code {
            self.marker(&block.common, out);
        }
        let begin = out.len();
        match block.kind {
            BlockKind::Prose => {
                for text in units {
                    out.push(text);
                    out.push(String::new());
                }
            }
            BlockKind::BulletList => {
                for text in units {
                    out.push(format!("- {text}"));
                }
                out.push(String::new());
            }
            BlockKind::NumberedList => {
                for (i, text) in units.iter().enumerate() {
                    out.push(format!("{}. {text}", i + 1));
                }
                out.push(String::new());
            }
            BlockKind::Blockquote => {
                for (i, text) in units.iter().enumerate() {
                    if i > 0 {
                        out.push(">".to_string());
                    }
                    out.push(format!("> {text}"));
                }
                out.push(String::new());
            }
            BlockKind::Table => {
                let columns = block.columns.as_deref().unwrap_or_default();
                out.push(format!("| {} |", columns.join(" | ")));
                out.push(format!(
                    "| {} |",
                    columns
                        .iter()
                        .map(|_| "---")
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
                for text in units {
                    out.push(format!("| {text} |"));
                }
                out.push(String::new());
            }
            BlockKind::Code => {
                let fence = "`".repeat(block.fence.max(3));
                let tag = match &block.lang {
                    Some(Lang::Tags(tags)) => tags.first().cloned().unwrap_or_default(),
                    Some(Lang::Any) | None => "text".to_string(),
                };
                for text in units {
                    self.marker(&block.common, out);
                    out.push(format!("{fence}{tag}"));
                    out.extend(text.lines().map(str::to_string));
                    out.push(fence.clone());
                    out.push(String::new());
                }
            }
            BlockKind::ThematicBreak => {
                out.push("---".to_string());
                out.push(String::new());
            }
            // A block that names a file takes its example like any other: the
            // target it names is a placeholder, reported in the new document
            // until the author replaces it. With no example, `units_of` has
            // already refused it.
            BlockKind::Html | BlockKind::Image | BlockKind::Region | BlockKind::References => {
                for text in units {
                    out.extend(text.lines().map(str::to_string));
                }
                out.push(String::new());
            }
            BlockKind::Group => unreachable!("a group is taken above"),
            BlockKind::Heading => unreachable!("no schema declares a heading block"),
        }
        self.follows(block, units_count, begin, out.len(), out);
    }

    /// Record the block just written, and refuse the derivation where it
    /// joins the run of the block before it, which takes every unit of
    /// that run: a document reads the two as one block, and the check
    /// gives the first all of it.
    // sokf-implements: MD-7_AC-11
    // sokf-implements: TLMD_149
    fn follows(&mut self, block: &Block, units: usize, begin: usize, end: usize, out: &[String]) {
        if let Some(&(kind, open, line, at)) = self.kinds.last()
            && open
            && kind == block.kind
            && reads_as_one(kind)
            && out
                .get(at..begin)
                .is_some_and(|between| between.iter().all(|l| l.trim().is_empty()))
        {
            self.refuse_on(
                line,
                "TLMD_149",
                format!(
                    "{1} `{0}` block whose rule admits more units than the scaffold writes is followed by another `{0}` block, which a document reads as one block, so the first takes all of it; give the first block a `max` of the units it holds",
                    kind.name(),
                    text::article(kind.name())
                ),
            );
        }
        self.kinds
            .push((block.kind, takes_more(block, units), block.common.line, end));
    }
}

/// Whether a block rule takes every unit of a run beyond the `units` that
/// the scaffold wrote for it: its `max` admits more, and no shape, format,
/// ban, guard or sequence can stop it at a unit.
fn takes_more(block: &Block, units: usize) -> bool {
    block.max.is_none_or(|max| max > units)
        && block.sequence.is_none()
        && block.rules.is_none()
        && block.each.as_ref().is_none_or(|u| {
            u.shape.is_none() && u.format.is_none() && u.prohibited_patterns.is_empty()
        })
}

/// Whether two blocks of `kind`, one after the other, read as one block:
/// paragraphs join one prose block (TLMD_5), and two lists of one kind
/// join one list.
fn reads_as_one(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::Prose | BlockKind::BulletList | BlockKind::NumberedList
    )
}
