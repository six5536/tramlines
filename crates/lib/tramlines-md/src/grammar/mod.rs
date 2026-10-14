//! The markdown format grammar on the core grammar, and the lowering of a
//! schema in it. The core items are re-exported here.

mod embedded;
mod load;
mod lower;
mod schema;
#[cfg(test)]
mod validate_tests;

pub use embedded::grammar_text;
pub use load::load_grammar;
pub use lower::{compile, lower};
pub use schema::{
    Block, Heading, Lang, Preamble, ProhibitedHeading, RuleLookup, Scaffold, Schema, Section, Unit,
};
pub use tramlines_core::grammar::grammar_text as core_grammar_text;
// sokf-implements: MD-9_AC-2
// sokf-implements: MD-9_AC-3
// sokf:begin grammar-reexports
pub use tramlines_core::grammar::{
    BUILT_INS, Captured, CoreSchema, DerivationSource, Example, Failure, FmFormat, Format,
    FormatMatch, FormatRegistry, FormatSource, FormatVariant, FrontmatterConstraint, Grammar,
    IdTemplate, KeyDef, KeyDerivation, LifecycleDimension, LineTemplate, Only, RegexCache,
    RuleCommon, RuleMap, SLUG, Scope as KeyScope, Segment, Shape, Squashed, States, Text, Token,
    TypeDef, TypeName, TypeRegistry, captured, has_hole, is_hole_name, load_core_grammar,
    lower_common, parse_grammar, refused, refused_as, shape_of, squash, text_or_template,
    validate_map, validate_schema,
};

// sokf:end grammar-reexports
