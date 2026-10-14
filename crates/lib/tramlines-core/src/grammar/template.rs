//! Templates (TL_24 to TL_26): the shape of one unit written as the unit itself,
//! with typed holes, `${name:type}`, where it varies. A template compiles
//! once to an anchored regex with one named group per named hole, and a
//! failure names the column where the text stops matching.

use std::{cell::RefCell, collections::HashMap, sync::OnceLock};

use regex::Regex;

use super::types::{TypeDef, TypeRegistry, is_hole_name};
use crate::{Problem, text};

/// One piece of a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Text the unit carries as written.
    Literal(String),
    /// A hole: its name where it is captured, and its type.
    Hole {
        /// The name, which is the name of its group.
        name: Option<String>,
        /// The type's name.
        ty: String,
        /// The type, as the registry resolved it.
        def: TypeDef,
    },
}

impl Segment {
    /// The segment as a failure names what it expected.
    fn describe(&self) -> String {
        match self {
            Segment::Literal(l) => format!("`{l}`"),
            Segment::Hole { ty, def, .. } => {
                format!("{} `{ty}` ({})", text::article(ty), def.describe())
            }
        }
    }

    fn regex(&self) -> String {
        match self {
            Segment::Literal(l) => regex::escape(l),
            Segment::Hole {
                name: Some(n), def, ..
            } => format!("(?P<{n}>{})", def.regex()),
            Segment::Hole {
                name: None, def, ..
            } => def.regex(),
        }
    }
}

/// Where a text stops matching a template, and what was expected there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The byte offset in the text where the fault is.
    pub offset: usize,
    /// What the template expected at the offset.
    pub expected: String,
    /// How far the text matches the template before it fails, which can
    /// lie past the fault: of two shapes, the text meant the one it
    /// follows furthest.
    pub reach: usize,
}

impl Failure {
    /// The failure as a finding says it: the column, one-based in
    /// characters, what was expected and what was found.
    pub fn describe(&self, text: &str) -> String {
        let column = column(text, self.offset);
        let rest = &text[self.offset.min(text.len())..];
        let found = if rest.is_empty() {
            "the end".to_string()
        } else if rest.chars().count() > 24 {
            format!("`{}…`", rest.chars().take(23).collect::<String>())
        } else {
            format!("`{rest}`")
        };
        format!("column {column}: expected {}, found {found}", self.expected)
    }
}

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template {
    /// The template as written.
    pub source: String,
    /// Its pieces, in order.
    pub segments: Vec<Segment>,
    /// The anchored regex it compiles to.
    regex: Regex,
    /// `prefixes[i]` matches the first `i` segments at the start of a text;
    /// compiled the first time a text fails, since most never do.
    prefixes: OnceLock<Vec<Regex>>,
}

thread_local! {
    /// Compiled regexes by source, so a template the layers compile again
    /// for each schema costs one compilation per run, as a pattern does
    /// through `RegexCache`.
    static COMPILED: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

/// `re` compiled, once per run and thread.
pub(super) fn compiled(re: &str) -> Result<Regex, regex::Error> {
    if let Some(hit) = COMPILED.with(|c| c.borrow().get(re).cloned()) {
        return Ok(hit);
    }
    let r = Regex::new(re)?;
    COMPILED.with(|c| c.borrow_mut().insert(re.to_string(), r.clone()));
    Ok(r)
}

impl PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.segments == other.segments
    }
}

impl Template {
    /// Parse and compile `source` against `types`. Refuses an unclosed
    /// hole, a bad or repeated name, an unknown type, and a template with
    /// no hole, which is text.
    // sokf-implements: TL_24
    // sokf-implements: TL_25
    // sokf-implements: TL_118
    // sokf-implements: TL_119
    // sokf-implements: TL_120
    // sokf-implements: TL_121
    pub fn parse(source: &str, types: &TypeRegistry) -> Result<Template, Problem> {
        let mut segments = Vec::new();
        let mut literal = String::new();
        let mut rest = source;
        let mut names: Vec<String> = Vec::new();
        while let Some(c) = rest.chars().next() {
            if let Some(after) = rest.strip_prefix("$${") {
                literal.push_str("${");
                rest = after;
                continue;
            }
            let Some(after) = rest.strip_prefix("${") else {
                literal.push(c);
                rest = &rest[c.len_utf8()..];
                continue;
            };
            let Some(end) = after.find('}') else {
                return Err(Problem::new(
                    "TL_121",
                    format!("template `{source}` has an unclosed `${{`"),
                ));
            };
            let inside = &after[..end];
            rest = &after[end + 1..];
            let (name, ty) = match inside.split_once(':') {
                Some((n, t)) => (n, t),
                None => (inside, "text"),
            };
            let name = if name.is_empty() {
                None
            } else if !is_hole_name(name) {
                return Err(Problem::new(
                    "TL_121",
                    format!(
                        "template `{source}`: hole name `{name}` must be letters, digits and `_`, not starting with a digit"
                    ),
                ));
            } else if names.iter().any(|n| n == name) {
                return Err(Problem::new(
                    "TL_119",
                    format!("template `{source}` names the hole `{name}` twice"),
                ));
            } else {
                names.push(name.to_string());
                Some(name.to_string())
            };
            let ty = if ty.is_empty() { "text" } else { ty };
            let Some(def) = types.get(ty) else {
                return Err(Problem::new(
                    "TL_120",
                    format!("template `{source}`: no type `{ty}`"),
                ));
            };
            if !literal.is_empty() {
                segments.push(Segment::Literal(std::mem::take(&mut literal)));
            }
            segments.push(Segment::Hole {
                name,
                ty: ty.to_string(),
                def: def.clone(),
            });
        }
        if !literal.is_empty() {
            segments.push(Segment::Literal(literal));
        }
        if !segments.iter().any(|s| matches!(s, Segment::Hole { .. })) {
            return Err(Problem::new(
                "TL_25",
                format!("template `{source}` has no hole; write it as `text`"),
            ));
        }
        let body: String = segments.iter().map(Segment::regex).collect();
        let regex = compiled(&format!("^(?:{body})$")).map_err(|e| {
            Problem::new(
                "TL_118",
                format!(
                    "template `{source}` does not compile: {}",
                    super::regex::reason(&e)
                ),
            )
        })?;
        Ok(Template {
            source: source.to_string(),
            regex,
            segments,
            prefixes: OnceLock::new(),
        })
    }

    /// `prefixes[i]` matches the first `i` segments at the start of a text.
    /// Each is a prefix of the template's own regex, which compiled.
    fn prefixes(&self) -> &[Regex] {
        self.prefixes.get_or_init(|| {
            let mut body = String::new();
            let mut out = vec![compiled("^").expect("an anchor compiles")];
            for s in &self.segments {
                body.push_str(&s.regex());
                out.push(
                    compiled(&format!("^(?:{body})"))
                        .expect("a prefix of a template that compiled compiles"),
                );
            }
            out
        })
    }

    /// The anchored regex, whose named groups are the named holes.
    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    /// Whether `text` matches the template whole.
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// The names of the captured holes, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(|s| match s {
            Segment::Hole { name: Some(n), .. } => Some(n.as_str()),
            _ => None,
        })
    }

    /// Where `text` stops matching: the longest run of segments that
    /// matches from its start, then the segment that does not.
    // sokf-implements: TL_26
    // sokf-implements: TL_74
    pub fn failure(&self, text: &str) -> Failure {
        let n = self.segments.len();
        let prefixes = self.prefixes();
        let k = (0..=n)
            .rev()
            .find(|&i| prefixes[i].is_match(text))
            .unwrap_or(0);
        let end_of = |i: usize| prefixes[i].find(text).map_or(0, |m| m.end());
        let at = |offset: usize, expected: String, reach: usize| Failure {
            offset,
            expected,
            reach,
        };
        if k == n {
            let end = end_of(n);
            // A literal end the hole before it reached inside the text, as
            // a period in `1.2`, leaves the text short of the real end.
            if let Some(h) = self.last_hole()
                && let Some(f) = self.short_end(text, h, end_of(h))
            {
                return f;
            }
            return at(end, "the end".to_string(), end);
        }
        let end = end_of(k);
        let prev = k.checked_sub(1).map(|p| &self.segments[p]);
        match (prev, &self.segments[k]) {
            // A hole that ran to the end of a text that ends the template
            // with literals leaves only those literals missing: the text
            // stops short, so the fault is at its end, after any part of
            // them that it carries.
            (Some(Segment::Hole { .. }), _)
                if end == text.len()
                    && self.last_hole() == Some(k - 1)
                    && let Some(f) = self.short_end(text, k - 1, end_of(k - 1)) =>
            {
                f
            }
            // A hole that ran to the end, where the literal after it could
            // not be part of its value, ended the line at its own end: the
            // line lacks only the rest of the template, so the fault is there.
            (Some(Segment::Hole { def, .. }), Segment::Literal(l))
                if end == text.len() && !whole(def, &format!("{}{l}", &text[end_of(k - 1)..])) =>
            {
                at(text.len(), format!("`{l}`"), text.len())
            }
            // A hole that ran to the end ate what the next segment wanted, so
            // the failure is where the hole began, naming both.
            (Some(prev @ Segment::Hole { .. }), next) if end == text.len() => at(
                end_of(k - 1),
                format!("{} then {}", prev.describe(), next.describe()),
                end,
            ),
            // A literal the text does not start to carry, after a hole: where
            // the literal comes later, the text between is the value the
            // writer meant for the hole, and the hole is what it fails.
            (Some(hole @ Segment::Hole { def, .. }), Segment::Literal(l))
                if common_prefix(l, &text[end..]) == 0
                    && text[end..]
                        .find(l.as_str())
                        .is_some_and(|i| !whole(def, &text[end_of(k - 1)..end + i])) =>
            {
                at(end_of(k - 1), hole.describe(), end)
            }
            // A literal the text starts to carry fails where the two part,
            // so of two templates that differ inside a literal the one the
            // text follows further is the one it meant.
            (_, Segment::Literal(l)) => {
                let shared = common_prefix(l, &text[end..]);
                at(end + shared, format!("`{}`", &l[shared..]), end + shared)
            }
            (_, hole) => at(end, hole.describe(), end),
        }
    }
}

impl Template {
    /// The last hole, where only literals follow it to the end.
    fn last_hole(&self) -> Option<usize> {
        let h = self
            .segments
            .iter()
            .rposition(|s| matches!(s, Segment::Hole { .. }))?;
        (h + 1 < self.segments.len()).then_some(h)
    }

    /// A text that stops short of the literal end of the template: the
    /// last hole, from `start`, takes the rest of the text, less any part
    /// of that end the text carries, so the fault is the rest of the end.
    fn short_end(&self, text: &str, h: usize, start: usize) -> Option<Failure> {
        let Segment::Hole { def, .. } = &self.segments[h] else {
            return None;
        };
        let tail: String = self.segments[h + 1..]
            .iter()
            .map(|s| match s {
                Segment::Literal(l) => l.as_str(),
                Segment::Hole { .. } => "",
            })
            .collect();
        let carried = (1..tail.len())
            .rev()
            .find(|&j| tail.is_char_boundary(j) && text.ends_with(&tail[..j]))
            .unwrap_or(0);
        let value = text.get(start..text.len() - carried)?;
        whole(def, value).then(|| Failure {
            offset: text.len(),
            expected: format!("`{}`", &tail[carried..]),
            reach: text.len(),
        })
    }
}

/// Whether the type `def` matches `value` whole.
fn whole(def: &TypeDef, value: &str) -> bool {
    compiled(&format!("^(?:{})$", def.regex())).is_ok_and(|r| r.is_match(value))
}

/// The one-based column, in characters, of the byte `offset` of `text`.
pub(super) fn column(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].chars().count() + 1
}

/// The length in bytes of the longest common prefix of `a` and `b`, on a
/// char boundary.
pub(super) fn common_prefix(a: &str, b: &str) -> usize {
    a.char_indices()
        .zip(b.chars())
        .find(|((_, x), y)| x != y)
        .map_or_else(|| a.len().min(b.len()), |((i, _), _)| i)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{grammar::FormatSource, yaml};

    fn types() -> TypeRegistry {
        let node = yaml::parse(
            "code: '[A-Z]{1,10}'\nkind: [Performance, Capacity]\nlabel: '[^\"<]+'\n",
            1,
        )
        .unwrap();
        let (own, problems) =
            TypeRegistry::from_yaml(&node, &FormatSource::Repo, &TypeRegistry::default());
        assert!(problems.is_empty());
        TypeRegistry::built_ins().layered(&own)
    }

    fn t(src: &str) -> Template {
        Template::parse(src, &types()).unwrap()
    }

    #[test]
    fn parses_holes_escapes_and_literals() {
        let tpl = t("${id:code}-${n:int} [x]: ${} $${a} costs $5 ${:kind}");
        assert_eq!(tpl.names().collect::<Vec<_>>(), ["id", "n"]);
        assert!(tpl.is_match("AB-12 [x]: any text ${a} costs $5 Capacity"));
        assert!(!tpl.is_match("AB-12 [x]: any text ${a} costs $5 Speed"));
        let caps = tpl
            .regex()
            .captures("AB-12 [x]: t ${a} costs $5 Performance")
            .unwrap();
        assert_eq!(&caps["id"], "AB");
        assert_eq!(&caps["n"], "12");
        // Regex characters in the literal are text.
        assert!(t("a.b ${}").is_match("a.b c"));
        assert!(!t("a.b ${}").is_match("axb c"));
    }

    // sokf-tests: TL_25
    // sokf-tests: TL_119
    // sokf-tests: TL_120
    // sokf-tests: TL_121
    #[test]
    fn refuses_what_is_not_a_template() {
        let ty = types();
        for (bad, key, says) in [
            ("no holes", "TL_25", "no hole"),
            ("$${x}", "TL_25", "no hole"),
            ("a ${x", "TL_121", "unclosed"),
            ("${x:nope}", "TL_120", "no type `nope`"),
            ("${x} ${x}", "TL_119", "twice"),
            ("${1x}", "TL_121", "hole name"),
            ("${a-b}", "TL_121", "hole name"),
        ] {
            let e = Template::parse(bad, &ty).unwrap_err();
            assert!(e.message.contains(says), "{bad}: {e:?}");
            assert_eq!(e.criterion, key, "{bad}");
        }
    }

    // sokf-tests: TL_26
    // sokf-tests: TL_74
    #[test]
    fn a_failure_names_the_column_and_what_was_expected() {
        let tpl = t("${id:code} [event]: WHEN ${}, the ${} SHALL ${}.");
        let text = "AB [state]: WHEN x, the y SHALL z.";
        let f = tpl.failure(text);
        assert_eq!(f.offset, 4);
        assert_eq!(
            f.describe(text),
            "column 5: expected `event]: WHEN `, found `state]: WHEN x, the y S…`"
        );
        let text = "AB [event]: WHEN a user posts THEN the API SHALL go.";
        let f = tpl.failure(text);
        assert_eq!(f.offset, "AB [event]: WHEN ".len());
        assert!(
            f.expected.starts_with("a `text` (`.+`) then `, the `"),
            "{f:?}"
        );
        // A text that lacks only the end of the template stops short.
        let text = "AB [event]: WHEN x, the y SHALL z";
        assert_eq!(
            tpl.failure(text).describe(text),
            "column 34: expected `.`, found the end"
        );
        let tpl = t("(${}.)");
        assert_eq!(
            tpl.failure("(x.").describe("(x."),
            "column 4: expected `)`, found the end"
        );
        // A period inside the value does not end the template.
        let tpl = t("${}.");
        assert_eq!(
            tpl.failure("Use version 1.2").describe("Use version 1.2"),
            "column 16: expected `.`, found the end"
        );
        // A hole that can't take the rest of the template ends a short line.
        let tpl = t("${:code} x${:int}");
        assert_eq!(
            tpl.failure("AB").describe("AB"),
            "column 3: expected ` x`, found the end"
        );
        // A hole the text gives a value it does not take is the fault.
        let tpl = t("${:word} ${:slug} ${:int}");
        let text = "x a--b 4";
        let f = tpl.failure(text);
        assert_eq!(
            f.describe(text),
            "column 3: expected a `slug` (`[a-z0-9]+(?:-[a-z0-9]+)*`), found `a--b 4`"
        );
        assert_eq!(f.reach, 3);
        let tpl = t("${:int}.${:int} ${:kind}");
        let text = "2.1 Speed";
        let f = tpl.failure(text);
        assert_eq!(
            f.describe(text),
            "column 5: expected a `kind` (one of `Performance`, `Capacity`), found `Speed`"
        );
        let tpl = t("x ${:int}");
        assert_eq!(
            tpl.failure("x 12 more").describe("x 12 more"),
            "column 5: expected the end, found ` more`"
        );
        assert_eq!(
            tpl.failure("y").describe("y"),
            "column 1: expected `x `, found `y`"
        );
        assert_eq!(
            tpl.failure("x").describe("x"),
            "column 2: expected ` `, found the end"
        );
    }

    /// A template of literals and holes of simple types, a line rendered
    /// from it, and the value each hole took.
    fn arb_line() -> impl Strategy<Value = (String, String, Vec<(String, String)>)> {
        prop::collection::vec(
            prop_oneof![
                "[a-z:;,. ]{1,4}".prop_map(|l| (None, l)),
                "[A-Z]{1,5}".prop_map(|v| (Some("code"), v)),
                "[0-9]{1,3}".prop_map(|v| (Some("int"), v)),
            ],
            1..6,
        )
        .prop_filter("a template has a hole", |parts| {
            parts.iter().any(|(ty, _)| ty.is_some())
        })
        .prop_map(|parts| {
            let (mut src, mut line, mut values) = (String::new(), String::new(), Vec::new());
            let mut last_hole = false;
            for (i, (ty, v)) in parts.into_iter().enumerate() {
                match ty {
                    Some(ty) => {
                        // Two holes side by side split their text anyhow; a
                        // literal between keeps each value its own.
                        if last_hole {
                            src.push('|');
                            line.push('|');
                        }
                        src.push_str(&format!("${{h{i}:{ty}}}"));
                        line.push_str(&v);
                        values.push((format!("h{i}"), v));
                        last_hole = true;
                    }
                    None => {
                        src.push_str(&v);
                        line.push_str(&v);
                        last_hole = false;
                    }
                }
            }
            (src, line, values)
        })
    }

    // sokf-tests: TL_24
    // sokf-tests: TL_26
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A line rendered from a template, with values drawn from its hole
        /// types, matches it and captures each value.
        #[test]
        fn a_rendered_line_matches_and_captures((src, line, values) in arb_line()) {
            let tpl = t(&src);
            let caps = tpl.regex().captures(&line);
            prop_assert!(caps.is_some(), "{src} / {line}");
            let caps = caps.unwrap();
            for (name, value) in &values {
                prop_assert_eq!(&caps[name.as_str()], value.as_str());
            }
        }

        /// A changed literal fails at or before the column of the change,
        /// and inside the line.
        #[test]
        fn a_changed_literal_fails_where_it_changed(
            head in "[a-z]{1,6}",
            tail in "[a-z]{1,6}",
            value in "[A-Z]{1,5}",
        ) {
            let tpl = t(&format!("{head}: ${{v:code}} {tail}"));
            let good = format!("{head}: {value} {tail}");
            prop_assert!(tpl.is_match(&good));
            // Change the first letter of the tail.
            let bad_tail: String = std::iter::once(if tail.starts_with('z') { 'y' } else { 'z' })
                .chain(tail.chars().skip(1))
                .collect();
            let bad = format!("{head}: {value} {bad_tail}");
            let changed = format!("{head}: {value} ").len();
            let f = tpl.failure(&bad);
            prop_assert!(f.offset <= changed && f.offset <= bad.len(), "{f:?}");
            prop_assert!(!tpl.is_match(&bad));
        }
    }
}
