//! The markdown format grammar the engine carries.

/// `knowledge/spec/tramlines-md.grammar.yaml`, the markdown format grammar,
/// verbatim.
pub fn grammar_text() -> &'static str {
    include_str!(concat!(
        env!("OUT_DIR"),
        "/knowledge/spec/tramlines-md.grammar.yaml"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_file_is_the_canonical_one() {
        assert!(grammar_text().starts_with("grammar: tramlines-md\n"));
    }
}
