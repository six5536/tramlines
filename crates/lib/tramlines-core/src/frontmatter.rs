//! Frontmatter constraints, lifecycle keys and the id template, checked
//! the same way in every format. The check reports [`Event`]s, which the
//! checker of a format records with what it knows of where it is.

use crate::{
    Spanned,
    grammar::{CoreSchema, FmFormat, FrontmatterConstraint, Only, RuleCommon, States, squash},
    text::{self, is_slug, is_url},
};

// sokf:begin host
/// What a check asks of its host about frontmatter: what lies outside the
/// document.
pub trait Host {
    /// Whether `value` holds the host's scalar format `name`: `None` when
    /// the host cannot tell, else the failure, which follows the key's
    /// name in the finding.
    fn scalar_format(&self, name: &str, value: &str) -> Option<Result<(), String>>;

    /// The frontmatter keys the host owns: never rejected as undeclared.
    fn reserved_keys(&self) -> &[&str];
}
// sokf:end host

// sokf:begin event
/// One thing the frontmatter check found.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A failure of `rule`, at the rule's severity, about the frontmatter
    /// key `key`.
    Rule {
        /// The rule that failed.
        rule: RuleCommon,
        /// The key the rule constrains.
        key: String,
        /// The line of the value, or of the frontmatter.
        line: Option<usize>,
        /// The key of the criterion that the failure breaks.
        criterion: &'static str,
        /// What failed.
        message: String,
    },
    /// An error the schema implies with no rule that states it.
    Error {
        /// The line.
        line: Option<usize>,
        /// The key of the criterion that the failure breaks.
        criterion: &'static str,
        /// What failed.
        message: String,
    },
    /// A gated rule matched the value at `line`.
    Gate {
        /// The gate.
        only: Only,
        /// The line of the value.
        line: usize,
        /// The line of the rule in the schema.
        rule: usize,
    },
}
// sokf:end event

/// The state of one frontmatter check.
struct Fm<'a> {
    states: &'a States,
    host: &'a dyn Host,
    /// The key whose constraint is being checked.
    key: String,
    events: Vec<Event>,
}

// sokf-implements: CORE-7_AC-12
impl Fm<'_> {
    // sokf-implements: CORE-7_AC-7
    fn emit(
        &mut self,
        rule: &RuleCommon,
        line: Option<usize>,
        criterion: &'static str,
        message: impl Into<String>,
    ) {
        self.events.push(Event::Rule {
            rule: rule.clone(),
            key: self.key.clone(),
            line,
            criterion,
            message: message.into(),
        });
    }

    // sokf-implements: CORE-7_AC-8
    fn error(&mut self, line: Option<usize>, criterion: &'static str, message: impl Into<String>) {
        self.events.push(Event::Error {
            line,
            criterion,
            message: message.into(),
        });
    }

    // sokf-implements: CORE-7_AC-9
    fn gate(&mut self, rule: &RuleCommon, line: usize) {
        if rule.only.is_gated() {
            self.events.push(Event::Gate {
                only: rule.only.clone(),
                line,
                rule: rule.line,
            });
        }
    }
}

/// Every declared key is present unless optional and holds its shape;
/// undeclared keys are refused unless `frontmatter-open`; lifecycle keys
/// hold a state; `id` matches its template.
// sokf-implements: TL_54
// sokf-implements: TL_55
// sokf-implements: TL_56
// sokf-implements: TL_57
// sokf-implements: TL_60
// sokf-implements: TL_61
// sokf-implements: TL_62
// sokf-implements: CORE-7_AC-1
// sokf-implements: CORE-7_AC-3
// sokf:begin frontmatter-check
pub fn check(
    frontmatter: Option<&Spanned>,
    schema: &CoreSchema,
    states: &States,
    host: &dyn Host,
) -> Vec<Event> {
    // sokf:end frontmatter-check
    let mut c = Fm {
        states,
        host,
        key: String::new(),
        events: Vec::new(),
    };
    let Some(frontmatter) = frontmatter.filter(|f| f.as_map().is_some()) else {
        return c.events;
    };
    // sokf-implements: CORE-7_AC-11
    let first_line = Some(frontmatter.line);
    for (key, constraint) in &schema.frontmatter {
        // sokf-implements: CORE-7_AC-10
        if !constraint.common.only.admits(c.states) {
            continue;
        }
        let value = frontmatter.get(key);
        // A frontmatter constraint has no `rule-id` and sits in no
        // section, so it names its key.
        // sokf-implements: TL_73
        c.key.clone_from(key);
        match value {
            None => {
                if !constraint.optional {
                    c.emit(
                        &constraint.common,
                        first_line,
                        "TL_54",
                        format!("frontmatter key `{key}` is missing"),
                    );
                }
            }
            Some(v) => {
                c.gate(&constraint.common, v.line);
                check_value(&mut c, key, constraint, v);
            }
        }
    }
    if !schema.frontmatter_open {
        // sokf-implements: CORE-7_AC-6
        let reserved = c.host.reserved_keys();
        let written = frontmatter.as_map().unwrap_or(&[]);
        for (key, value) in written {
            let key = &key.value;
            let declared = schema.frontmatter.iter().any(|(k, _)| k == key)
                || schema.lifecycle.iter().any(|d| &d.key == key)
                || reserved.contains(&key.as_str());
            if !declared {
                c.error(
                    Some(value.line),
                    "TL_55",
                    format!("frontmatter key `{key}` is not declared"),
                );
            }
        }
    }
    for dim in &schema.lifecycle {
        match frontmatter.get(&dim.key) {
            None => c.error(
                first_line,
                "TL_62",
                format!("lifecycle key `{}` is missing", dim.key),
            ),
            Some(v) => {
                let text = v.scalar_text().unwrap_or_default();
                // sokf-implements: TL_108
                if !dim.states.contains(&text) {
                    c.error(
                        Some(v.line),
                        "TL_108",
                        format!(
                            "`{}` must be one of {}; `{text}` is not",
                            dim.key,
                            text::code_list(&dim.states, ", ")
                        ),
                    );
                }
            }
        }
    }
    c.events
}

fn check_value(c: &mut Fm<'_>, key: &str, constraint: &FrontmatterConstraint, value: &Spanned) {
    let common = &constraint.common;
    if constraint.list {
        let Some(items) = value.as_seq() else {
            c.emit(
                common,
                Some(value.line),
                "TL_57",
                format!("`{key}` must be a list"),
            );
            return;
        };
        if let Some(min) = constraint.min
            && items.len() < min
        {
            // sokf-implements: TL_106
            c.emit(
                common,
                Some(value.line),
                "TL_106",
                format!(
                    "`{key}` needs at least {}; it has {}",
                    text::count(min, "entry", "entries"),
                    items.len()
                ),
            );
        }
        if let Some(max) = constraint.max
            && items.len() > max
        {
            // sokf-implements: TL_107
            c.emit(
                common,
                Some(value.line),
                "TL_107",
                format!(
                    "`{key}` allows at most {}; it has {}",
                    text::count(max, "entry", "entries"),
                    items.len()
                ),
            );
        }
        for item in items {
            check_scalar(c, key, constraint, item);
        }
    } else {
        if value.as_seq().is_some() {
            c.emit(
                common,
                Some(value.line),
                "TL_57",
                format!("`{key}` must be a scalar, not a list"),
            );
            return;
        }
        check_scalar(c, key, constraint, value);
    }
}

fn check_scalar(c: &mut Fm<'_>, key: &str, constraint: &FrontmatterConstraint, value: &Spanned) {
    let common = &constraint.common;
    let line = Some(value.line);
    let Some(text) = value.scalar_text() else {
        c.emit(common, line, "TL_57", format!("`{key}` must be a scalar"));
        return;
    };
    if let Some(expected) = &constraint.const_
        && &text != expected
    {
        c.emit(
            common,
            line,
            "TL_56",
            format!("`{key}` must be `{expected}`; it is `{text}`"),
        );
    }
    // The `id` template is also the one a new document is filled with, and fixes the
    // width of its sequence, so it answers for the `id` key alone.
    let squashed = squash(&text);
    if let Some(shape) = &constraint.shape
        && constraint.id_template.is_none()
        && !shape.matches(&squashed, |_| None)
    {
        c.emit(
            common,
            line,
            "TL_56",
            format!(
                "`{key}` `{text}` {}{}",
                shape.failure(&squashed),
                constraint
                    .example
                    .as_deref()
                    .map_or_else(String::new, |e| format!("; a value of this kind reads {e}"))
            ),
        );
    }
    if let Some(allowed) = &constraint.enum_
        && !allowed.contains(&text)
    {
        c.emit(
            common,
            line,
            "TL_56",
            format!(
                "`{key}` must be one of {}; `{text}` is not",
                text::code_list(allowed, ", ")
            ),
        );
    }
    match &constraint.format {
        // A format the host owns, the host answers, and a value it refuses
        // fails its format as any other does.
        Some(FmFormat::Host(name)) => {
            // sokf-implements: CORE-7_AC-4
            // sokf-implements: CORE-7_AC-5
            if let Some(Err(failure)) = c.host.scalar_format(name, &text) {
                c.emit(common, line, "TL_56", format!("`{key}` {failure}"));
            }
        }
        Some(FmFormat::Url) if !is_url(&text) => c.emit(
            common,
            line,
            "TL_56",
            format!("`{key}` must be an absolute URL; `{text}` is not"),
        ),
        Some(FmFormat::Slug) if !is_slug(&text) => c.emit(
            common,
            line,
            "TL_56",
            format!("`{key}` must be a slug; `{text}` is not"),
        ),
        Some(FmFormat::Url | FmFormat::Slug) | None => {}
    }
    // A fixed id is its `const`, which answers for it above.
    if let Some(template) = &constraint.id_template
        && constraint.const_.is_none()
    {
        if !template.matches(&text) {
            c.emit(
                common,
                line,
                "TL_56",
                format!(
                    "id `{text}` does not match the template `{}`",
                    template.source
                ),
            );
        }
        // sokf-implements: CORE-7_AC-2
        // sokf-implements: TL_59
        for (name, held) in template.keys_of(&text) {
            let Some(state) = c.states.0.get(&name).and_then(|s| s.first()) else {
                continue;
            };
            if &held != state {
                c.emit(
                    common,
                    line,
                    "TL_59",
                    format!("id `{text}` holds `{held}` for `{name}`, but `{name}` is `{state}`"),
                );
            }
        }
    }
}
