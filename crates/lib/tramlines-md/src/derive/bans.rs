//! The scaffold writes the message of each ban after the guidance of the
//! part that the ban governs, so that a writer reads every rule before
//! writing, and not only after breaking it (TLMD_195). The bans of a block
//! are the guide's, in `guide.rs`.

use super::Deriver;

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
}
