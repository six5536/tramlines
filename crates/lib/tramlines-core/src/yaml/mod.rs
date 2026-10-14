//! YAML 1.2 core-schema parsing as TL_13 and TL_14 read it.

mod parse;
mod value;

pub use parse::parse;
pub use value::{Spanned, Yaml, YamlError};
