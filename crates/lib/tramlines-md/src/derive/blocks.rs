//! The blocks of a section in the scaffold: each block's units, its
//! guidance and its marker, and the refusal of a block that the next one
//! would join.

use super::{Deriver, guide};
use crate::check::is_run;
use crate::{
    grammar::{Block, Lang, LifecycleDimension, States},
    markdown::BlockKind,
    text,
};

impl Deriver<'_> {
    pub(super) fn blocks(&mut self, rules: Option<&[Block]>, out: &mut Vec<String>) {
        let rules = rules.unwrap_or(&[]);
        self.meetings(rules);
        let mut i = 0;
        while i < rules.len() {
            // A run of optional blocks is one region: a label and the list
            // under it are one decision, not two.
            if rules[i].optional_at(self.states, &self.schema.core.lifecycle) && !self.in_optional {
                let mut j = i;
                while j < rules.len()
                    && rules[j].optional_at(self.states, &self.schema.core.lifecycle)
                {
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
        // whose hard `max` admits no unit bans rather than demands.
        if !block.common.only.admits(self.states) || block.hard_max() == Some(0) {
            return;
        }
        // An optional block is written where the derivation can write it,
        // and left out where a rule inside it offers nothing, as an
        // optional section is.
        // sokf-implements: TLMD_123
        self.attempt(
            block.optional_at(self.states, &self.schema.core.lifecycle),
            out,
            |d, out| d.block(block, out),
        );
    }

    fn block(&mut self, block: &Block, out: &mut Vec<String>) {
        // The host's words and the `when-at-limit` of each limit, and the
        // messages of the block's bans, follow its guidance, as the host's
        // guide gives them.
        // sokf-implements: TLMD_195
        // sokf-implements: TLMD_231
        // sokf-implements: MD-7_AC-14
        let text = guide::block_text(self.schema, self.host, block);
        self.guidance(text.as_deref(), out);
        if block.kind == BlockKind::Group {
            // A group's `rule-id` names the keys captured inside it, as the
            // check files them.
            let inner = block.rule_id.clone().or(self.group_rule.clone());
            let held = std::mem::replace(&mut self.group_rule, inner);
            for _ in 0..block.min.max(1) {
                self.marker(&block.common, out);
                self.blocks(block.blocks.as_deref(), out);
            }
            self.group_rule = held;
            return;
        }
        let units = self.units_of(block);
        // Each fence of a code block is a block of its own, so each one
        // carries its own `only` marker, which the match below writes.
        if block.kind != BlockKind::Code {
            self.marker(&block.common, out);
        }
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
                // A table with no declared columns names the header of its
                // scaffold under `example-columns`, so that the separator
                // has one `---` for each cell of the header.
                // sokf-implements: TLMD_206
                // sokf-implements: MD-7_AC-13
                let columns = block
                    .example_columns
                    .as_deref()
                    .or(block.columns.as_deref())
                    .unwrap_or_default();
                // A `|` in a name would end its cell, so the header escapes
                // it, as a GFM table writes a pipe inside a cell.
                let names: Vec<String> = columns.iter().map(|c| c.replace('|', "\\|")).collect();
                out.push(format!("| {} |", names.join(" | ")));
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
        self.follows(block);
    }

    /// Refuse the derivation where a position whose units have no shape
    /// of their own can stand directly before a position of its kind: the
    /// two are one block in a document, the first takes all of it, and no
    /// count divides a run. The positions between them, where there are
    /// any, are optional, so a document can leave them out. A group that
    /// can repeat, and closes with such a position and opens with its kind,
    /// meets itself. A later group whose opening has a shape and is
    /// followed by a block of another kind takes the end of the run by that
    /// shape (TLMD_193), so the position before it meets nothing.
    // sokf-implements: MD-7_AC-11
    // sokf-implements: TLMD_149
    fn meetings(&mut self, rules: &[Block]) {
        let states = self.states;
        let lifecycle = &self.schema.core.lifecycle[..];
        // A block whose hard `max` of zero bans it is written nowhere, so a
        // document leaves it out as it leaves out an optional one, but the
        // check still reads it as a position that takes the block after a run.
        let admitted: Vec<&Block> = rules
            .iter()
            .filter(|b| b.common.only.admits(states))
            .collect();
        let banned = |b: &Block| b.hard_max() == Some(0);
        let mut met: Vec<&Block> = Vec::new();
        for (i, b) in admitted.iter().enumerate() {
            if banned(b) {
                continue;
            }
            for last in closings(b, states, lifecycle) {
                if !is_run(last.kind) || !takes_more(last) {
                    continue;
                }
                let later = &admitted[i + 1..];
                // The check reserves the end of a run only for the position
                // that takes the run in the list of the later group, never
                // across the end of a group that the run closes.
                let own = std::ptr::eq(last, *b);
                let opened = |next: &Block| {
                    openings(next, states, lifecycle).iter().any(|o| {
                        o.kind == last.kind && !(own && reserved(later, next, o, states, lifecycle))
                    })
                };
                // The next repetition of a group is no later group, so no
                // reservation takes the end of the run for it.
                let mut meets = b.kind == BlockKind::Group
                    && b.hard_max().is_none_or(|m| m > 1)
                    && openings(b, states, lifecycle)
                        .iter()
                        .any(|o| o.kind == last.kind);
                for next in &admitted[i + 1..] {
                    if meets {
                        break;
                    }
                    if banned(next) {
                        continue;
                    }
                    meets = opened(next);
                    if !next.optional_at(states, lifecycle) {
                        break;
                    }
                }
                if meets && !met.iter().any(|m| std::ptr::eq(*m, last)) {
                    met.push(last);
                }
            }
        }
        for last in met {
            let kind = last.kind.name();
            self.refuse_on(
                last.common.line,
                "TLMD_149",
                format!(
                    "{} `{kind}` block whose units have no shape of their own can stand directly before another `{kind}` block, which a document reads as one block, so the first takes all of it; give the units of the first block a shape that the units of the second do not have, or put the second under a heading of its own",
                    text::article(kind)
                ),
            );
        }
    }

    /// Record the block just written, whose kind tells an item what its
    /// nested blocks open with.
    fn follows(&mut self, block: &Block) {
        self.kinds.push(block.kind);
    }
}

/// The blocks that `b` can end with at `states`: `b` itself, or for a
/// group, what its last admitted inner block can end with, and what the
/// block before it can end with while the blocks after are optional.
fn closings<'b>(
    b: &'b Block,
    states: &'b States,
    lifecycle: &[LifecycleDimension],
) -> Vec<&'b Block> {
    ends(b, states, lifecycle, true)
}

/// The blocks that `b` can open with at `states`, as `closings` reads the
/// end of a group.
fn openings<'b>(
    b: &'b Block,
    states: &'b States,
    lifecycle: &[LifecycleDimension],
) -> Vec<&'b Block> {
    ends(b, states, lifecycle, false)
}

/// The blocks that `b` can open with, or close with where `last`: for a
/// group, those of each inner block from that end, up to and with the
/// first that is not optional.
fn ends<'b>(
    b: &'b Block,
    states: &'b States,
    lifecycle: &[LifecycleDimension],
    last: bool,
) -> Vec<&'b Block> {
    if b.kind != BlockKind::Group {
        return vec![b];
    }
    let mut inner: Vec<&Block> = b.admitted(states).collect();
    if last {
        inner.reverse();
    }
    let mut out = Vec::new();
    for i in inner {
        out.extend(ends(i, states, lifecycle, last));
        if !i.optional_at(states, lifecycle) {
            break;
        }
    }
    out
}

/// Whether the group `next` takes the end of a run by the shape of its
/// opening `o` in every document, as `reserve_for` reserves it in the
/// check (TLMD_193): `o` is the group's first inner block, required and with
/// a shape, the group's second block is required and of another kind, and
/// no other position of `later`, the positions after the run, opens with
/// that kind and so takes the block after the run itself.
fn reserved(
    later: &[&Block],
    next: &Block,
    o: &Block,
    states: &States,
    lifecycle: &[LifecycleDimension],
) -> bool {
    let mut inner = next.admitted(states);
    let (Some(first), Some(second)) = (inner.next(), inner.next()) else {
        return false;
    };
    if next.kind != BlockKind::Group
        || !std::ptr::eq(first, o)
        || o.optional_at(states, lifecycle)
        || takes_more(o)
    {
        return false;
    }
    let Some(after) = second.first_position(states).map(|s| s.kind) else {
        return false;
    };
    !first.optional_at(states, lifecycle)
        && !second.optional_at(states, lifecycle)
        && after != o.kind
        && !later
            .iter()
            .filter(|p| !std::ptr::eq(**p, next))
            .any(|p| p.first_position(states).is_some_and(|f| f.kind == after))
}

/// Whether a block rule takes every unit of a run: no shape, format, ban,
/// guard or sequence can stop it at a unit, and no count divides a run.
fn takes_more(block: &Block) -> bool {
    block.sequence.is_none()
        && block.rules.is_none()
        && block.each.as_ref().is_none_or(|u| {
            u.shape.is_none() && u.format.is_none() && u.prohibited_patterns.is_empty()
        })
}
