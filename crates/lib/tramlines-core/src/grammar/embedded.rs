//! The core grammar the engine carries.

/// `knowledge/spec/tramlines.grammar.yaml`, the core grammar, verbatim.
pub fn grammar_text() -> &'static str {
    include_str!(concat!(
        env!("OUT_DIR"),
        "/knowledge/spec/tramlines.grammar.yaml"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_file_is_the_canonical_one() {
        assert!(grammar_text().starts_with("grammar: tramlines\n"));
    }
}
