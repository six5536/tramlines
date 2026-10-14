//! tramlines: check documents against one schema.
//!
//! `tramlines validate --schema <file> [--formats <file>] [--format <name>]
//! [<doc>...]` holds the schema to itself and each document to the schema,
//! with the engine's own vocabulary and no host, and repairs each document
//! first with `--fix`. `tramlines new` prints the scaffold of a schema,
//! `tramlines info` the schema language and the formats a schema sees, and
//! `tramlines completions` a shell completion script. Each finding is one
//! line, `path:line: severity[KEY]: message`, then a count, or one JSON
//! object with `--json`. Exit 0 on no error, 1 on an error, 2 on a usage
//! error or a file that cannot be read.

#![warn(missing_docs)]

mod cli;
mod info;
mod load;
mod man;
mod new;
mod report;
mod validate;

use std::{io::Write, process::ExitCode};

use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command};

/// A failure that stops the run: exit 2. Each of its errors names the
/// criterion that it breaks, where one does.
pub struct Stop {
    errors: Vec<(Option<String>, String)>,
}

impl Stop {
    /// A stop that breaks `criterion`.
    pub fn on(criterion: &str, message: String) -> Self {
        Stop {
            errors: vec![(Some(criterion.to_string()), message)],
        }
    }

    /// A stop that no criterion describes, such as a file that does not
    /// read.
    pub fn plain(message: String) -> Self {
        Stop {
            errors: vec![(None, message)],
        }
    }

    /// A stop of several errors, each `(criterion, message)`.
    pub fn each(errors: impl IntoIterator<Item = (String, String)>) -> Self {
        Stop {
            errors: errors.into_iter().map(|(k, m)| (Some(k), m)).collect(),
        }
    }
}

fn main() -> ExitCode {
    // sokf-implements: CLI-5_AC-5
    let result = match Cli::parse().command {
        Command::Validate(args) => validate::validate(&args),
        Command::New(args) => new::new(&args),
        Command::Info(args) => info::info(&args.what),
        // sokf-implements: CLI-14_AC-1
        Command::Completions(args) => {
            clap_complete::generate(
                args.shell,
                &mut Cli::command(),
                "tramlines",
                &mut std::io::stdout(),
            );
            Ok(0)
        }
        Command::Man => man::render()
            .and_then(|page| std::io::stdout().write_all(&page))
            .map(|()| 0)
            .map_err(|e| Stop::plain(e.to_string())),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        // sokf-implements: CLI-5_AC-4
        // sokf-implements: CLI-5_AC-6
        // sokf-implements: TL_99
        Err(Stop { errors }) => {
            for (criterion, message) in errors {
                match criterion {
                    Some(key) => eprintln!("error[{key}]: {message}"),
                    None => eprintln!("error: {message}"),
                }
            }
            ExitCode::from(2)
        }
    }
}
