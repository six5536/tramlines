//! Id templates (TL_58): the `id` key's template, whose holes are
//! the ones a new document is filled with, `${:seq}`, `${:slug}` and a
//! lifecycle key's state, `${<key>}`.

use regex::Regex;

use super::{SLUG, template};
use crate::{
    Problem,
    grammar::{LifecycleDimension, LineTemplate, Segment, States},
    text::{self, is_slug},
};

/// The pieces of an id template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Literal text.
    Literal(String),
    /// A `seq` hole: the number, `seq-width` digits.
    Seq(usize),
    /// A `slug` hole: one or more slug words.
    Slug,
    /// A hole named for a lifecycle key: the document's state of that key.
    Key {
        /// The lifecycle key.
        name: String,
        /// Its states, each a slug.
        states: Vec<String>,
    },
}

/// An id template a new document can be filled with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdTemplate {
    /// The template as written.
    pub source: String,
    /// The tokens, in order.
    pub tokens: Vec<Token>,
}

impl IdTemplate {
    /// The id template of `template`, its sequence `width` digits wide, a
    /// hole named with no type standing for the state of the `lifecycle`
    /// key of its name. Refuses a hole a new document cannot be filled with,
    /// more than one `seq`, a zero width, a lifecycle key with a state that is
    /// not a slug, and a template that admits a non-slug.
    // sokf-implements: TL_58
    // sokf-implements: TL_102
    // sokf-implements: TL_114
    // sokf-implements: TL_122
    pub fn from_template(
        template: &LineTemplate,
        width: usize,
        lifecycle: &[LifecycleDimension],
    ) -> Result<Self, Problem> {
        let source = &template.source;
        let mut tokens = Vec::new();
        let mut seqs = 0;
        for segment in &template.segments {
            match segment {
                Segment::Literal(l) => tokens.push(Token::Literal(l.clone())),
                Segment::Hole { ty, .. } if ty == "seq" => {
                    seqs += 1;
                    if seqs > 1 {
                        return Err(Problem::new(
                            "TL_114",
                            format!("template `{source}` has more than one `seq` hole"),
                        ));
                    }
                    if width == 0 {
                        return Err(Problem::new(
                            "TL_102",
                            format!("template `{source}`: `seq-width` must be positive"),
                        ));
                    }
                    tokens.push(Token::Seq(width));
                }
                Segment::Hole { ty, .. } if ty == "slug" => tokens.push(Token::Slug),
                Segment::Hole {
                    name: Some(name),
                    ty,
                    ..
                } if ty == "text" => {
                    let Some(dim) = lifecycle.iter().find(|d| &d.key == name) else {
                        return Err(Problem::new(
                            "TL_58",
                            format!(
                                "template `{source}`: the hole `${{{name}}}` names no lifecycle key"
                            ),
                        ));
                    };
                    if let Some(bad) = dim.states.iter().find(|s| !is_slug(s)) {
                        return Err(Problem::new(
                            "TL_122",
                            format!(
                                "template `{source}`: the state `{bad}` of `{name}` is not a slug, so no id can hold it"
                            ),
                        ));
                    }
                    tokens.push(Token::Key {
                        name: name.clone(),
                        states: dim.states.clone(),
                    });
                }
                Segment::Hole { ty, .. } => {
                    return Err(Problem::new(
                        "TL_58",
                        format!(
                            "template `{source}`: an id holds only the holes a new document is filled with, `${{:seq}}`, `${{:slug}}` and `${{<lifecycle key>}}`, not {} `{ty}`",
                            text::article(ty)
                        ),
                    ));
                }
            }
        }
        let sample: String = tokens
            .iter()
            .map(|t| match t {
                Token::Literal(s) => s.as_str(),
                _ => "x",
            })
            .collect();
        if !is_slug(&sample) {
            return Err(Problem::new(
                "TL_122",
                format!("template `{source}` admits a non-slug: `{sample}` is not a slug"),
            ));
        }
        Ok(IdTemplate {
            source: source.clone(),
            tokens,
        })
    }

    /// The id template of a fixed id, the `const` of `id`: one literal, a
    /// slug.
    // sokf-implements: TL_122
    pub fn fixed(id: &str) -> Result<Self, Problem> {
        if !is_slug(id) {
            return Err(Problem::new(
                "TL_122",
                format!("the id `{id}` is not a slug"),
            ));
        }
        Ok(IdTemplate {
            source: id.to_string(),
            tokens: vec![Token::Literal(id.to_string())],
        })
    }

    /// Whether the template carries a `seq` hole, and its width.
    pub fn seq_width(&self) -> Option<usize> {
        self.tokens.iter().find_map(|t| match t {
            Token::Seq(n) => Some(*n),
            _ => None,
        })
    }

    /// Whether the template carries a `slug` hole.
    pub fn has_slug(&self) -> bool {
        self.tokens.contains(&Token::Slug)
    }

    /// Whether the template carries a hole for a lifecycle key.
    pub fn has_keys(&self) -> bool {
        self.tokens.iter().any(|t| matches!(t, Token::Key { .. }))
    }

    /// An anchored regex matching the ids the template admits; the `seq`
    /// hole is the group `seq`, and a lifecycle key's hole the group
    /// `key_<name>`.
    pub fn regex(&self) -> String {
        let mut s = String::from("^");
        for t in &self.tokens {
            match t {
                Token::Literal(l) => s.push_str(&regex::escape(l)),
                Token::Seq(n) => s.push_str(&format!("(?P<seq>[0-9]{{{n}}})")),
                Token::Slug => s.push_str(SLUG),
                Token::Key { name, states } => {
                    let alts: Vec<String> = states.iter().map(|s| regex::escape(s)).collect();
                    s.push_str(&format!("(?P<key_{name}>{})", alts.join("|")));
                }
            }
        }
        s.push('$');
        s
    }

    fn compiled(&self) -> Regex {
        template::compiled(&self.regex()).expect("template regex is built from escaped parts")
    }

    /// Whether `id` matches the template.
    pub fn matches(&self, id: &str) -> bool {
        self.compiled().is_match(id)
    }

    /// The sequence number of `id`, when it matches and the template has a
    /// sequence.
    pub fn seq_of(&self, id: &str) -> Option<u64> {
        let caps = self.compiled().captures(id)?;
        caps.name("seq")?.as_str().parse().ok()
    }

    /// The lifecycle keys whose state `id` holds, each with that state, when
    /// `id` matches the template.
    pub fn keys_of(&self, id: &str) -> Vec<(String, String)> {
        let Some(caps) = self.compiled().captures(id) else {
            return vec![];
        };
        self.tokens
            .iter()
            .filter_map(|t| match t {
                Token::Key { name, .. } => caps
                    .name(&format!("key_{name}"))
                    .map(|m| (name.clone(), m.as_str().to_string())),
                _ => None,
            })
            .collect()
    }

    /// Render the id for `seq` and `slug`, each lifecycle key's hole taking
    /// its first state. Refuses what [`IdTemplate::render_with`] refuses.
    pub fn render(&self, seq: u64, slug: Option<&str>) -> Result<String, String> {
        self.render_with(seq, slug, &States::default())
    }

    /// Render the id for `seq`, `slug` and `states`, a lifecycle key's hole
    /// taking the state `states` gives it, else its first. Refuses a sequence
    /// that needs more digits than the width, a missing slug, a slug that is
    /// not slug words, and a state that is not one of the key's.
    pub fn render_with(
        &self,
        seq: u64,
        slug: Option<&str>,
        states: &States,
    ) -> Result<String, String> {
        let mut s = String::new();
        for t in &self.tokens {
            match t {
                Token::Literal(l) => s.push_str(l),
                Token::Seq(n) => {
                    let digits = format!("{seq:0width$}", width = n);
                    if digits.len() > *n {
                        return Err(format!("the next number {seq} needs more than {n} digits"));
                    }
                    s.push_str(&digits);
                }
                Token::Slug => match slug {
                    Some(slug) if is_slug(slug) => s.push_str(slug),
                    Some(slug) => return Err(format!("`{slug}` is not slug words")),
                    None => return Err("the template has a `slug` hole; pass --slug".into()),
                },
                Token::Key { name, states: all } => {
                    let state = states
                        .0
                        .get(name)
                        .and_then(|s| s.first())
                        .or_else(|| all.first())
                        .cloned()
                        .unwrap_or_default();
                    if !all.contains(&state) {
                        return Err(format!("`{state}` is not a state of `{name}`"));
                    }
                    s.push_str(&state);
                }
            }
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::grammar::TypeRegistry;

    fn parse(src: &str, width: usize) -> Result<IdTemplate, String> {
        parse_in(src, width, &[])
    }

    fn parse_in(
        src: &str,
        width: usize,
        lifecycle: &[LifecycleDimension],
    ) -> Result<IdTemplate, String> {
        let t = LineTemplate::parse(src, &TypeRegistry::built_ins()).map_err(|e| e.message)?;
        IdTemplate::from_template(&t, width, lifecycle).map_err(|e| e.message)
    }

    fn kind(states: &[&str]) -> LifecycleDimension {
        LifecycleDimension {
            key: "kind".into(),
            states: states.iter().map(|s| s.to_string()).collect(),
            guidance: None,
            line: 1,
            extensions: Default::default(),
        }
    }

    // sokf-tests: TL_58
    #[test]
    fn a_lifecycle_key_fills_its_hole() {
        let kinds = [kind(&["feature", "bug", "chore"])];
        let t = parse_in("issue-${:seq}-${kind}-${:slug}", 3, &kinds).unwrap();
        assert!(t.has_keys());
        assert!(t.matches("issue-004-bug-retry-logic"));
        assert!(!t.matches("issue-004-task-retry-logic"));
        assert_eq!(t.seq_of("issue-004-bug-retry-logic"), Some(4));
        assert_eq!(
            t.keys_of("issue-004-bug-retry-logic"),
            vec![("kind".to_string(), "bug".to_string())]
        );
        let mut states = States::default();
        states.0.insert("kind".into(), vec!["chore".into()]);
        assert_eq!(
            t.render_with(7, Some("tidy"), &states).unwrap(),
            "issue-007-chore-tidy"
        );
        assert_eq!(t.render(7, Some("tidy")).unwrap(), "issue-007-feature-tidy");
        states.0.insert("kind".into(), vec!["task".into()]);
        assert!(t.render_with(7, Some("tidy"), &states).is_err());
        // A hole of another name, or a key whose states are not slugs, is refused.
        assert!(parse_in("issue-${:seq}-${state}-${:slug}", 3, &kinds).is_err());
        assert!(parse_in("issue-${:seq}-${kind}", 3, &[kind(&["Bug"])]).is_err());
    }

    #[test]
    fn parses_templates() {
        let t = parse("issue-${:seq}-${:slug}", 3).unwrap();
        assert_eq!(
            t.tokens,
            vec![
                Token::Literal("issue-".into()),
                Token::Seq(3),
                Token::Literal("-".into()),
                Token::Slug
            ]
        );
        assert_eq!(t.seq_width(), Some(3));
        assert!(t.has_slug());
        assert_eq!(
            t.regex(),
            "^issue\\-(?P<seq>[0-9]{3})\\-[a-z0-9]+(?:-[a-z0-9]+)*$"
        );
        assert!(t.matches("issue-004-retry-logic"));
        assert!(!t.matches("issue-4-retry"));
        assert!(!t.matches("issue-004-Retry"));
        assert_eq!(t.seq_of("issue-004-retry-logic"), Some(4));
        assert_eq!(t.seq_of("nope"), None);
        assert_eq!(
            t.render(4, Some("retry-logic")).unwrap(),
            "issue-004-retry-logic"
        );
        assert!(t.render(1000, Some("x")).is_err());
        assert!(t.render(1, None).is_err());
        assert!(t.render(1, Some("Bad Slug")).is_err());
        let plain = parse("adr-${:seq}", 4).unwrap();
        assert!(!plain.has_slug());
        assert_eq!(plain.render(12, None).unwrap(), "adr-0012");
        assert_eq!(parse("${:slug}", 3).unwrap().seq_width(), None);
        // A named hole is still the hole of its type.
        assert!(parse("x-${n:seq}", 2).unwrap().matches("x-07"));
    }

    #[test]
    fn refuses_bad_templates() {
        for (bad, width) in [
            ("issue-${:seq}-${:seq}", 3),
            ("issue-${:seq}", 0),
            ("issue-${:int}", 3),
            ("issue-${slug}", 3),
            ("Issue-${:seq}", 3),
            ("issue_${:slug}", 3),
            ("issue--${:slug}", 3),
            ("-${:slug}", 3),
        ] {
            assert!(parse(bad, width).is_err(), "{bad}");
        }
    }

    fn arb_template() -> impl Strategy<Value = (String, usize)> {
        (
            "[a-z][a-z0-9]{0,4}",
            1usize..5,
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(|(prefix, width, with_slug, with_kind)| {
                let kind = if with_kind { "-${kind}" } else { "" };
                let t = if with_slug {
                    format!("{prefix}-${{:seq}}{kind}-${{:slug}}")
                } else {
                    format!("{prefix}-${{:seq}}{kind}")
                };
                (t, width)
            })
    }

    // sokf-tests: TL_58
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn render_then_seq_of_round_trips((template, width) in arb_template(), seq in 0u64..10_000, slug in "[a-z0-9]+(-[a-z0-9]+){0,2}", state in prop::sample::select(vec!["feature", "bug", "chore"])) {
            let t = parse_in(&template, width, &[kind(&["feature", "bug", "chore"])]).unwrap();
            let mut states = States::default();
            states.0.insert("kind".into(), vec![state.to_string()]);
            let rendered = t.render_with(seq, Some(&slug), &states);
            let fits = seq.to_string().len() <= width;
            prop_assert_eq!(rendered.is_ok(), fits);
            if let Ok(id) = rendered {
                prop_assert!(is_slug(&id), "{id}");
                prop_assert!(t.matches(&id));
                prop_assert_eq!(t.seq_of(&id), Some(seq));
                if t.has_keys() {
                    prop_assert_eq!(t.keys_of(&id), vec![("kind".to_string(), state.to_string())]);
                }
            }
        }
    }
}
