//! The scaffold writes the message of each ban after the guidance of the
//! part that the ban governs, so that a writer reads every rule before
//! writing, and not only after breaking it (TLMD_195).

use super::{Deriver, deriver::push_unique};
use crate::grammar::{Block, Unit};

impl Deriver<'_> {
    /// `schema_bans` returns the messages of the bans of the whole body.
    pub(super) fn schema_bans(&self) -> Vec<String> {
        self.schema
            .core
            .prohibited_patterns
            .iter()
            .map(|b| b.message.clone())
            .collect()
    }

    /// `block_bans` returns the messages of the bans that govern the units
    /// of `block`: the bans of its unit rules, of the formats that they
    /// bind, and of their line rules, each message once, in order.
    // sokf-implements: TLMD_195
    pub(super) fn block_bans(&self, block: &Block) -> Vec<String> {
        let mut out = Vec::new();
        for unit in block.units() {
            self.unit_bans(unit, &mut out);
        }
        out
    }

    fn unit_bans(&self, unit: &Unit, out: &mut Vec<String>) {
        let format = unit.format_in(&self.schema.core.formats);
        let bans = unit
            .prohibited_patterns
            .iter()
            .chain(format.into_iter().flat_map(|f| f.prohibited.iter()));
        push_unique(out, bans.map(|ban| ban.message.clone()));
        for line in unit.lines.iter().flatten() {
            self.unit_bans(line, out);
        }
    }
}
