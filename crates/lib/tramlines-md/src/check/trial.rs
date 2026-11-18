//! A trial of a position against a block: where the output of a check
//! stands before the trial, and what a rollback of the trial drops or
//! keeps.

use super::{Accepted, Broken, Checker, LabelRepair};
use crate::{Finding, Severity, text};

/// The length of everything a check writes, taken before a trial.
#[derive(Debug, Clone, Copy)]
pub(super) struct Savepoint {
    pub(super) findings: usize,
    captures: usize,
    repairs: usize,
    label_repairs: usize,
    gates: usize,
    accepted: usize,
    broken: usize,
}

impl Checker<'_> {
    /// Where the output stands now.
    pub(super) fn save(&self) -> Savepoint {
        Savepoint {
            findings: self.out.findings.len(),
            captures: self.out.captures.len(),
            repairs: self.out.repairs.len(),
            label_repairs: self.out.label_repairs.len(),
            gates: self.out.gates.len(),
            accepted: self.out.accepted.len(),
            broken: self.broken.len(),
        }
    }

    /// Whether an error was written since `at`. A warning is a document
    /// that passed, so it is not one.
    // sokf-implements: TLMD_65
    pub(super) fn failed(&self, at: &Savepoint) -> bool {
        self.out.findings[at.findings..]
            .iter()
            .any(|f| f.severity == Severity::Error)
    }

    /// Whether an error other than units that don't fit within a hard
    /// `max` above zero was written since `at`. No limit decides which
    /// position takes a block or where a position ends in a run, so a claim
    /// reads only these. A hard `max` of zero bans what it counts, and a
    /// `min` demands units, so their findings still decide.
    // sokf-implements: TLMD_59
    // sokf-implements: TLMD_74
    pub(super) fn failed_by_text(&self, at: &Savepoint) -> bool {
        self.out.findings[at.findings..].iter().any(is_text_error)
    }

    /// Drop everything written since `at`, yielding the findings, the
    /// records of accepted units, and the units that break their rules
    /// dropped. Every list grows and none shrinks, so savepoints nest.
    // sokf-implements: TLMD_64
    pub(super) fn rollback(&mut self, at: &Savepoint) -> Dropped {
        self.out.captures.truncate(at.captures);
        self.out.repairs.truncate(at.repairs);
        self.out.gates.truncate(at.gates);
        // sokf-implements: TLMD_213
        // sokf-implements: MD-4_AC-15
        let accepted = self.out.accepted.split_off(at.accepted);
        Dropped {
            findings: self.out.findings.split_off(at.findings),
            accepted,
            broken: self.broken.split_off(at.broken),
            label_repairs: self.out.label_repairs.split_off(at.label_repairs),
            blocks: 1,
        }
    }

    /// Keep what a refused position wrote after all, where the position
    /// keeps the block: its findings, the units its rules accepted, and
    /// the units that break them.
    // sokf-implements: TLMD_228
    // sokf-implements: MD-4_AC-16
    pub(super) fn keep(&mut self, dropped: Dropped) {
        self.out.findings.extend(dropped.findings);
        self.out.accepted.extend(dropped.accepted);
        self.broken.extend(dropped.broken);
        self.out.label_repairs.extend(dropped.label_repairs);
    }
}

/// Whether `f` is an error that decides a claim: any error other than a
/// fault of a label (TLMD_249 to TLMD_251), and other than one that reports
/// more units than a hard `max` above zero allows, the units
/// of a block (TLMD_160) or the units that a rule takes or counts
/// (TLMD_159). A `max` of zero is a ban, and its finding says `at most 0`.
/// Only the engine's words before the quoted `when-at-limit` decide, so
/// that the schema's text decides nothing.
pub(super) fn is_text_error(f: &Finding) -> bool {
    let words = f
        .message
        .split_once(text::AT_LIMIT)
        .map_or(f.message.as_str(), |(words, _)| words);
    let overflow = match f.criterion.as_str() {
        "TLMD_160" => !words.contains(" allows at most 0 "),
        "TLMD_159" => !words.contains(": at most 0 are allowed;"),
        _ => false,
    };
    // A label is no part of the text that decides a claim (TLMD_248).
    let label = matches!(f.criterion.as_str(), "TLMD_249" | "TLMD_250" | "TLMD_251");
    f.severity == Severity::Error && !overflow && !label
}

/// What a rollback dropped: the findings, the records of the units that
/// the rules accepted, and the units that break their rules.
#[derive(Debug)]
pub(super) struct Dropped {
    pub(super) findings: Vec<Finding>,
    accepted: Vec<Accepted>,
    pub(super) broken: Vec<Broken>,
    /// The repairs of labels, which the kept findings report.
    label_repairs: Vec<LabelRepair>,
    /// The blocks that the refusal covers, from the block refused: one,
    /// or each block of the repetition that an absent group gave back.
    pub(super) blocks: usize,
}

/// No findings and no records, over one block, so that a walk that keeps
/// the refusal still moves on.
impl Default for Dropped {
    fn default() -> Self {
        Self {
            findings: Vec::new(),
            accepted: Vec::new(),
            broken: Vec::new(),
            label_repairs: Vec::new(),
            blocks: 1,
        }
    }
}
