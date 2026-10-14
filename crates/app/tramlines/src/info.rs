//! `tramlines info`: the schema language, and the formats and types that a
//! schema sees.

use tramlines_md::grammar::{core_grammar_text, grammar_text};

use crate::{
    Stop,
    cli::{InfoCommand, InfoFormatsArgs},
    load,
};

/// Print what `what` asks for.
pub fn info(what: &InfoCommand) -> Result<u8, Stop> {
    match what {
        InfoCommand::Grammar => print!("{}", grammar()),
        InfoCommand::Formats(args) => print!("{}", formats(args)?),
    }
    Ok(0)
}

/// The core grammar, then the markdown grammar, as two YAML documents.
// sokf-implements: CLI-13_AC-1
fn grammar() -> String {
    format!("{}---\n{}", core_grammar_text(), grammar_text())
}

/// The registry as YAML with the built-in types: as the schema sees it,
/// or the formats file alone.
// sokf-implements: CLI-13_AC-2
// sokf-implements: CLI-13_AC-3
fn formats(args: &InfoFormatsArgs) -> Result<String, Stop> {
    let engine = load::engine(args.formats.as_deref())?;
    let Some(path) = &args.schema else {
        return Ok(engine.registry.to_yaml(true));
    };
    Ok(load::compiled(&engine, path)?.core.formats.to_yaml(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grammar_is_the_core_then_the_markdown_grammar() {
        let text = grammar();
        assert!(text.starts_with("grammar: tramlines\n"), "{text}");
        assert!(text.contains("---\ngrammar: tramlines-md\n"));
    }
}
