//! Why a reference to a set of keys finds no rule that captures one.

/// Why `Schema::rule_captures_key` found no rule that captures a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleLookup {
    /// No rule of the schema carries the id.
    NoRule,
    /// A section or a block declares the id as its `rule-id`, or a unit
    /// rule names it as its `key-of`, and no key belongs to it.
    NoKey,
    /// A unit rule or a line rule that captures a key declares the id as
    /// its `rule-id`, which holds no key, and no key belongs to the id:
    /// the rule that means it declares `key-of`.
    NameOnly,
}

impl RuleLookup {
    /// The refusal, for the key `what` that named the set `id`.
    pub fn message(self, what: &str, id: &str) -> String {
        match self {
            RuleLookup::NoRule => format!("`{what}` names no set `{id}`"),
            RuleLookup::NoKey => format!("`{what}` names set `{id}`, which holds no key"),
            RuleLookup::NameOnly => format!(
                "`{what}` names `{id}`, the `rule-id` of a unit rule, which holds no key: declare `key-of: {id}` on each rule whose keys it means, or name the `key-of` that the rule declares"
            ),
        }
    }
}
