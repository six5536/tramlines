//! The key of an example against the examples of the parent that its
//! rule's `key-derives-from` names (`TL_127`), which the check of the
//! examples of a schema reads.

use crate::{
    grammar::{Block, DerivationSource, KeyDerivation, KeyText, Schema, Unit, Words},
    markdown::Reading,
    text,
};

/// What a check of one example finds: the reason it fails, if it does.
type Problem = Option<String>;

/// The example `text` of a unit of `block` that `rule` governs against
/// the rule's `key-derives-from`, read without its label in `words`.
pub(super) fn derived_unit(
    schema: &Schema,
    words: &Words,
    block: &Block,
    rule: &Unit,
    text: &str,
) -> Problem {
    let kd = rule.key_derives_from.as_ref()?;
    let key = schema.example_key(words, block, rule, text);
    derived(schema, kd, key, rule.reader())
}

/// The key of an example, `key`, against the example of the parent that
/// `kd` names: the key starts with the value that the scaffold writes for
/// the frontmatter key from the schema, or with a key of the example of
/// the rule, followed by the separator. The scaffold splices its prefix in
/// there, so an example that starts with another contradicts its schema. A
/// key the example does not carry is a fault of its own, and a parent whose
/// value the schema does not state has no example to start with. A value of
/// the frontmatter reads squashed, as `read` reads the key (`TLMD_259`).
pub(super) fn derived(
    schema: &Schema,
    kd: &KeyDerivation,
    key: Option<KeyText>,
    read: fn(&str) -> Reading,
) -> Problem {
    let key = key?;
    let sep = &kd.separator;
    let parents = schema.parent_examples(kd, read, key.verbatim);
    if parents.is_empty() || key.parent_in(kd, &parents).is_some() {
        return None;
    }
    let (of, mut named): (String, Vec<String>) = match &kd.source {
        DerivationSource::From(k) => (
            format!("the value that the scaffold writes for the frontmatter `{k}`"),
            parents.iter().map(|p| text::code(p.written())).collect(),
        ),
        DerivationSource::Rule(r) => (
            format!("a key of an example of set `{r}`"),
            parents.iter().map(|p| text::code(&p.read)).collect(),
        ),
    };
    let mut seen = std::collections::HashSet::new();
    named.retain(|n| seen.insert(n.clone()));
    Some(format!(
        "the key {} of the example does not start with {}, {of}, followed by the separator {}",
        text::code(&key.read),
        named.join(" or "),
        text::code(sep)
    ))
}
