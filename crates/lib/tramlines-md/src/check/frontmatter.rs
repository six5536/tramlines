//! The frontmatter of a markdown document, checked by tramlines-core and
//! recorded here with where each finding sits.

use tramlines_core::frontmatter::{Event, check as check_core};

use super::{At, Checker, Gate};

/// The frontmatter against the core keys of the schema. A frontmatter
/// constraint has no `rule-id` and sits in no section, so a finding of one
/// names its key.
pub(super) fn check(c: &mut Checker<'_>) {
    // sokf-implements: MD-4_AC-6
    let events = check_core(c.doc.frontmatter, &c.schema.core, c.states, c.host);
    for event in events {
        match event {
            Event::Rule {
                rule,
                key,
                line,
                criterion,
                message,
            } => {
                let at = At {
                    named: Some(format!("key `{key}`")),
                    ..At::default()
                };
                c.located(at, |c| c.emit(&rule, line, criterion, message));
            }
            Event::Error {
                line,
                criterion,
                message,
            } => c.error(line, criterion, message),
            Event::Gate { only, line, rule } => c.out.gates.push(Gate { line, only, rule }),
        }
    }
}
