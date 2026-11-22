//! The walk of a section whose blocks are in any order: every rule that
//! admits a block is tried, and where none is clean the rule whose refusal
//! the walk keeps claims the block.

use std::borrow::Cow;

use super::{
    Checker, blocks,
    claim::{self, Around, Claimed, Docs},
};
use crate::{grammar::Block as Rule, markdown::BlockKind};

/// Blocks in any order: every admitting rule is tried, and where none is
/// clean the rule whose refusal the walk keeps claims the block and reports.
// sokf-implements: TLMD_62
pub(super) fn match_free(
    c: &mut Checker<'_>,
    rules: &[&Rule],
    docs: &mut Docs<'_>,
    section: Option<usize>,
    at: usize,
) {
    // How many claims each rule made, how many repetitions each group
    // holds, which its `min` counts at its next claim, and the line of each
    // rule's first claim.
    let mut used = vec![0usize; rules.len()];
    let mut held = vec![0usize; rules.len()];
    let mut first = vec![at; rules.len()];
    let mut di = 0;
    while di < docs.len() {
        let free: Vec<usize> = (0..rules.len())
            // A group always repeats, so it is free after its first claim
            // too, as it takes every repetition in a walk in order.
            .filter(|&ri| used[ri] == 0 || rules[ri].repeat || rules[ri].kind == BlockKind::Group)
            .collect();
        let mut taken = None;
        let mut refusal = None;
        let fit: Vec<usize> = free
            .iter()
            .copied()
            .filter(|&ri| c.matches_rule(&docs[di], rules[ri]))
            .collect();
        for ri in fit {
            // The positions after a rule are the other free rules, and not
            // the rule itself, whose next repetition is its own business.
            // sokf-implements: TLMD_71
            let later: Vec<&Rule> = free
                .iter()
                .filter(|&&k| k != ri)
                .map(|&k| rules[k])
                .collect();
            let kinds = claim::opened(&later, c);
            let around = Around {
                later: &later,
                kinds: &kinds,
                held: held[ri],
                ordered: false,
                dividing: &[],
            };
            // Dividing a run needs an order to divide it along, so the
            // free walk takes whole blocks (LANG-5_AC-4).
            let before = c.save();
            match claim::claim(c, docs, di, rules[ri], section, false, false, around) {
                // A group that took no block leaves it to the rules after
                // it, and to the report of a block nobody claimed, and keeps
                // nothing that it wrote, such as its gate.
                Ok(Claimed { blocks: 0, .. }) => {
                    c.rollback(&before);
                }
                Ok(claimed) => {
                    taken = Some((ri, claimed));
                    break;
                }
                Err(dropped) => {
                    if refusal.is_none() {
                        refusal = Some((ri, dropped));
                    }
                }
            }
        }
        // Nothing clean: the rule whose refusal the walk keeps takes the
        // block, so a malformed block keeps its own finding, with every
        // block that the refusal covers, and the block and its findings go
        // to one position.
        // sokf-implements: TLMD_228
        let taken = taken.or_else(|| {
            let (ri, dropped) = refusal.take()?;
            let blocks = dropped.blocks;
            c.keep(dropped);
            Some((
                ri,
                Claimed {
                    blocks,
                    repetitions: 0,
                },
            ))
        });
        if let Some((ri, claimed)) = taken {
            // The count is reported at the group's first repetition.
            if held[ri] == 0 && claimed.repetitions > 0 {
                first[ri] = docs[di].line;
            }
            used[ri] += 1;
            held[ri] += claimed.repetitions;
            di += claimed.blocks;
        } else {
            claim::unclaimed(c, &docs[di], matches!(docs[di], Cow::Owned(_)));
            di += 1;
        }
    }
    for (ri, rule) in rules.iter().enumerate() {
        // A group's repetitions are counted once, over every claim of the
        // walk, so that its bounds read every repetition wherever it stands.
        // A group that holds none took its blocks only with the reason that
        // it refused them, which says why it is short.
        // sokf-implements: TLMD_146
        // sokf-implements: TLMD_160
        if rule.kind == BlockKind::Group && held[ri] > 0 {
            c.located(claim::at_block(&c.at, rule), |c| {
                blocks::check_count(c, rule, held[ri], first[ri]);
            });
        }
        if used[ri] == 0 && !c.block_optional(rule) {
            let at_rule = claim::at_block(&c.at, rule);
            c.located(at_rule, |c| {
                // sokf-implements: TLMD_152
                c.emit(
                    &rule.common,
                    Some(at),
                    "TLMD_152",
                    format!(
                        "{} block is missing",
                        rule.kind.spoken_in(&c.host.dialect())
                    ),
                );
            });
        }
    }
}
