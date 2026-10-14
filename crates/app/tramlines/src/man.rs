//! The man page: the top-level page that `clap_mangen` renders, then a
//! COMMANDS section with each subcommand's arguments, and EXIT STATUS.

use std::fmt::Write as _;

use clap::CommandFactory;

use crate::cli::Cli;

/// The complete man page, as roff.
// sokf-implements: CLI-14_AC-2
pub fn render() -> std::io::Result<Vec<u8>> {
    let man = clap_mangen::Man::new(Cli::command());
    let mut buf = Vec::new();
    man.render_title(&mut buf)?;
    man.render_name_section(&mut buf)?;
    man.render_synopsis_section(&mut buf)?;
    man.render_description_section(&mut buf)?;
    let mut extra = String::new();
    render_commands(&mut extra);
    extra.push_str(
        ".SH EXIT STATUS\n\
         .TP\n.B 0\nNo error.\n\
         .TP\n.B 1\nErrors found.\n\
         .TP\n.B 2\nA usage error, or a file that cannot be read, reported on stderr.\n",
    );
    buf.extend_from_slice(extra.as_bytes());
    man.render_version_section(&mut buf)?;
    Ok(buf)
}

/// One `.SS` subsection for each visible subcommand, and for each
/// subcommand of one that has them.
fn render_commands(out: &mut String) {
    out.push_str(".SH COMMANDS\n");
    for sub in Cli::command().get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        if sub.has_subcommands() {
            for inner in sub.get_subcommands().filter(|c| !c.is_hide_set()) {
                render_command(out, &format!("tramlines {}", sub.get_name()), inner);
            }
        } else {
            render_command(out, "tramlines", sub);
        }
    }
}

fn render_command(out: &mut String, prefix: &str, sub: &clap::Command) {
    let _ = writeln!(out, ".SS \"{}\"", usage(prefix, sub));
    let about = sub.get_about().map(ToString::to_string).unwrap_or_default();
    out.push_str(&text(&about));
    for arg in sub.get_arguments() {
        if arg.get_id() == "help" {
            continue;
        }
        out.push_str(".TP\n");
        let _ = writeln!(out, "{}", arg_header(arg));
        let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
        out.push_str(&text(&help));
    }
}

/// The `.SS` heading: `tramlines <command> [OPTIONS] [<POSITIONAL>]`.
fn usage(prefix: &str, sub: &clap::Command) -> String {
    let mut s = format!("{prefix} {}", sub.get_name());
    if sub.get_arguments().any(|a| !a.is_positional()) {
        s.push_str(" [OPTIONS]");
    }
    for pos in sub.get_positionals() {
        if pos.is_required_set() {
            let _ = write!(s, " <{}>", value_name(pos));
        } else {
            let _ = write!(s, " [{}]", value_name(pos));
        }
    }
    s
}

/// The bold option, or the italic positional, that heads a `.TP` entry.
fn arg_header(arg: &clap::Arg) -> String {
    if arg.is_positional() {
        return format!("\\fI<{}>\\fR", value_name(arg));
    }
    let mut s = String::new();
    if let Some(long) = arg.get_long() {
        let _ = write!(s, "\\fB\\-\\-{}\\fR", long.replace('-', "\\-"));
    }
    if arg.get_action().takes_values() {
        let _ = write!(s, " \\fI<{}>\\fR", value_name(arg));
    }
    s
}

/// An argument's `<VALUE>` placeholder: its `value_name`, or its id.
fn value_name(arg: &clap::Arg) -> String {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map(ToString::to_string)
        .unwrap_or_else(|| arg.get_id().to_string().to_uppercase())
}

/// A roff text line: backslashes escaped, a leading control character
/// protected.
fn text(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\");
    if escaped.starts_with('.') || escaped.starts_with('\'') {
        format!("\\&{escaped}\n")
    } else {
        format!("{escaped}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_lists_every_visible_command() {
        let page = String::from_utf8(render().unwrap()).unwrap();
        for heading in [
            ".SS \"tramlines validate [OPTIONS] [DOC]\"",
            ".SS \"tramlines new [OPTIONS]\"",
            ".SS \"tramlines info grammar\"",
            ".SS \"tramlines info formats [OPTIONS]\"",
            ".SS \"tramlines completions <SHELL>\"",
        ] {
            assert!(page.contains(heading), "{heading} in\n{page}");
        }
        assert!(!page.contains("tramlines man"));
        assert!(page.contains(".SH EXIT STATUS"));
    }
}
