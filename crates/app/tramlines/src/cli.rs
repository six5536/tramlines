//! The command line: the subcommands and their options.

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

// sokf:begin cli
/// The formats the command reads, each with the extensions that match it.
// sokf-implements: CLI-3_AC-2
pub const FORMATS: &[(&str, &[&str])] = &[("md", &["md", "markdown"])];

/// Checks documents against a tramlines schema.
#[derive(Debug, Parser)]
// sokf-implements: CLI-1_AC-9
// sokf-implements: CLI-1_AC-10
#[command(name = "tramlines", version)]
#[command(
    after_help = "Exit codes: 0 no error, 1 errors found, 2 usage error or a file that cannot be read.\n\
    Docs: https://six5536.github.io/tramlines/"
)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands.
// sokf-implements: CLI-1_AC-12
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check a schema, then each document against it, repairing with --fix.
    Validate(ValidateArgs),
    /// Print the scaffold of a new document from a schema.
    New(NewArgs),
    /// Print the schema language, or the formats and types a schema sees.
    Info(InfoArgs),
    /// Print a shell completion script to stdout.
    Completions(CompletionsArgs),
    /// Print a roff man page to stdout. Hidden: it exists for packaging.
    // sokf-implements: CLI-14_AC-3
    #[command(hide = true)]
    Man,
}

// sokf:end cli

// sokf:begin validate
/// Arguments for `validate`.
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// The schema file, in YAML.
    #[arg(long, value_name = "FILE")]
    // sokf-implements: CLI-1_AC-8
    pub schema: PathBuf,
    /// A file of line formats and types that the schema can use.
    #[arg(long, value_name = "FILE")]
    pub formats: Option<PathBuf>,
    /// Read every document in this format, whatever its extension: `md`.
    #[arg(long, value_name = "NAME")]
    pub format: Option<String>,
    /// The documents to check; `-` reads one from standard input. Without
    /// a document, the command checks the schema alone.
    #[arg(value_name = "DOC")]
    pub docs: Vec<PathBuf>,
    /// Repair the headings and anchors that the schema derives, then check.
    #[arg(long)]
    pub fix: bool,
    /// One JSON object on stdout instead of text.
    #[arg(long)]
    pub json: bool,
    /// A JMESPath expression applied to the JSON object. Requires --json.
    // sokf-implements: CLI-10_AC-4
    #[arg(long, value_name = "EXPR", requires = "json")]
    pub query: Option<String>,
    /// GitHub workflow commands on stdout instead of text.
    // sokf-implements: CLI-10_AC-7
    #[arg(long, conflicts_with = "json")]
    pub github: bool,
    /// List each warning. Default: their count.
    #[arg(long)]
    pub warnings: bool,
}

// sokf:end validate

// sokf:begin new
/// Arguments for `new`.
#[derive(Debug, Args)]
pub struct NewArgs {
    /// The schema file, in YAML.
    #[arg(long, value_name = "FILE")]
    pub schema: PathBuf,
    /// A file of line formats and types that the schema can use.
    #[arg(long, value_name = "FILE")]
    pub formats: Option<PathBuf>,
    /// Fills the `slug` hole of the id template.
    #[arg(long, value_name = "SLUG", conflicts_with = "id")]
    pub slug: Option<String>,
    /// The id, for a template with a `seq` hole or for no template.
    #[arg(long, value_name = "ID")]
    pub id: Option<String>,
    /// The title. Default: the words of the slug, or of the id.
    #[arg(long, value_name = "TEXT")]
    pub title: Option<String>,
    /// A lifecycle key at a state, as KEY=STATE. Default: each dimension's first state.
    #[arg(long, value_name = "KEY=STATE", action = ArgAction::Append, value_parser = parse_state)]
    pub state: Vec<(String, String)>,
    /// One object with the id and the document instead of the document.
    #[arg(long)]
    pub json: bool,
}

// sokf:end new

// sokf:begin info
/// Arguments for `info`.
#[derive(Debug, Args)]
pub struct InfoArgs {
    /// What to print.
    #[command(subcommand)]
    pub what: InfoCommand,
}

/// The `info` subcommands.
#[derive(Debug, Subcommand)]
pub enum InfoCommand {
    /// Print the schema language verbatim: the core grammar, then the
    /// markdown grammar.
    Grammar,
    /// Print as YAML the formats and types a schema sees, each naming its source.
    Formats(InfoFormatsArgs),
}

/// Arguments for `info formats`.
#[derive(Debug, Args)]
pub struct InfoFormatsArgs {
    /// The registry as this schema sees it. Default: the formats file alone.
    #[arg(long, value_name = "FILE")]
    pub schema: Option<PathBuf>,
    /// A file of line formats and types.
    #[arg(long, value_name = "FILE")]
    pub formats: Option<PathBuf>,
}

// sokf:end info

// sokf:begin completions
/// Arguments for `completions`.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// The shell to print a completion script for.
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}
// sokf:end completions

/// Parse `KEY=STATE` at the first `=`.
fn parse_state(s: &str) -> Result<(String, String), String> {
    match s.split_once('=') {
        Some((k, v)) if !k.is_empty() && !v.is_empty() => Ok((k.to_string(), v.to_string())),
        _ => Err(format!("`{s}` is not KEY=STATE")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_splits_at_the_first_equals_sign() {
        assert_eq!(
            parse_state("state=open"),
            Ok(("state".into(), "open".into()))
        );
        assert_eq!(parse_state("k=a=b"), Ok(("k".into(), "a=b".into())));
        assert!(parse_state("state").is_err());
        assert!(parse_state("=open").is_err());
        assert!(parse_state("state=").is_err());
    }
}
