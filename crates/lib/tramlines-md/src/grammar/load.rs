//! The grammar a markdown schema is checked against.

use super::{Grammar, grammar_text, load_core_grammar};
use crate::Result;

// sokf-implements: MD-1_AC-1
// sokf-implements: MD-1_AC-2
// sokf:begin load-grammar
/// The grammar of a markdown schema: the core grammar with the markdown
/// format grammar laid on it.
pub fn load_grammar() -> Result<Grammar> {
    let mut grammar = load_core_grammar()?;
    grammar.base(grammar_text())?;
    Ok(grammar)
}
// sokf:end load-grammar
