//! The markdown format grammar on the core grammar, and the lowering of a
//! schema in it. The core items are re-exported here.

mod embedded;
mod label;
mod load;
mod lookup;
mod lower;
mod schema;
#[cfg(test)]
mod validate_tests;

pub use embedded::grammar_text;
pub use label::Label;
pub(crate) use label::{
    Found, Words, begins_with_key, carries_labels, cut, is_words, task_box, without,
};
pub use load::load_grammar;
pub use lookup::RuleLookup;
pub use lower::{compile, lower};
pub use schema::{
    Block, Heading, Lang, Preamble, ProhibitedHeading, Scaffold, Schema, Section, Unit,
};
pub use tramlines_core::grammar::grammar_text as core_grammar_text;
// sokf-implements: MD-9_AC-2
// sokf-implements: MD-9_AC-3
// sokf:begin grammar-reexports
pub use tramlines_core::grammar::{
    BUILT_INS, Captured, Condition, CoreSchema, DerivationSource, Example, Failure, FmFormat,
    Format, FormatMatch, FormatRegistry, FormatSource, FormatVariant, FrontmatterConstraint,
    Grammar, IdTemplate, KeyDef, KeyDerivation, LifecycleDimension, Limit, LimitKind, LineTemplate,
    Only, PackLayer, RegexCache, RuleCommon, RuleMap, SLUG, Scope as KeyScope, Segment, Shape,
    Squashed, States, Text, Token, TypeDef, TypeName, TypeRegistry, captured, has_hole,
    is_hole_name, load_core_grammar, lower_common, parse_grammar, refused, refused_as,
    refused_masked, shape_of, squash, text_or_template, validate_map, validate_schema,
};

// sokf:end grammar-reexports
