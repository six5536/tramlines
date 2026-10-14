//! The core grammar and the layers on it, schema validation, the lowering
//! of the keys every format shares, the format registry, and id templates.

mod embedded;
mod format;
mod id_template;
mod lower;
mod meta;
mod model;
mod patch;
mod regex;
mod registry;
mod shape;
mod template;
mod types;
mod validate;

pub use embedded::grammar_text;
pub use format::{
    Captured, Format, FormatMatch, FormatVariant, Prohibition, captured, prohibitions, refused,
    refused_as,
};
pub use id_template::{IdTemplate, Token};
pub use lower::{
    CORE_SCHEMA_KEYS, Lowerer, bool_of, derivation, extensions, list_of, lower_common,
    lower_formats, str_of, usize_of,
};
pub use meta::{Grammar, KeyDef, RuleMap, TypeName, load_core_grammar, parse_grammar};
pub use model::{
    CoreSchema, DerivationSource, Example, FmFormat, FrontmatterConstraint, KeyDerivation,
    LifecycleDimension, Only, RuleCommon, Scope, States, Text,
};
pub use regex::RegexCache;
pub use registry::{FormatRegistry, FormatSource};
pub use shape::{Shape, Squashed, has_hole, shape_of, squash, text_or_template};
pub use template::{Failure, Segment, Template as LineTemplate};
pub use types::{BUILT_INS, SLUG, TypeDef, TypeRegistry, is_hole_name};
pub use validate::{validate_map, validate_schema};
