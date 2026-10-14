//! The shape of one unit (TL_22 to TL_26): exact `text`, a `template` with typed
//! holes, or a `pattern` regex, one key per meaning wherever a rule
//! declares a shape. Every shape reads the unit's text squashed, each run
//! of whitespace one space.

use regex::{Captures, Regex};

use super::{LineTemplate, RegexCache, Text, TypeRegistry, template};
use crate::{Problem, Spanned};

/// A unit's shape.
#[derive(Debug, Clone)]
pub enum Shape {
    /// `text`: what the text must equal, or the frontmatter key whose value
    /// it must equal.
    Text(Text),
    /// `template`: the text written with typed holes.
    Template(LineTemplate),
    /// `pattern`: a regex the text must match somewhere.
    Pattern(Regex),
}

impl Shape {
    /// The shape's regex, whose named groups are the keys it can capture;
    /// none for `text`.
    pub fn regex(&self) -> Option<&Regex> {
        match self {
            Shape::Text(_) => None,
            Shape::Template(t) => Some(t.regex()),
            Shape::Pattern(p) => Some(p),
        }
    }

    /// Whether `text`, squashed, has this shape. `value` resolves a
    /// `{ from: <key> }` text to the frontmatter value, squashed.
    pub fn matches(&self, text: &str, value: impl Fn(&str) -> Option<String>) -> bool {
        match self {
            Shape::Text(Text::Literal(want)) => squash(want) == text,
            Shape::Text(Text::From(key)) => value(key).is_some_and(|v| squash(&v) == text),
            Shape::Template(t) => t.is_match(text),
            Shape::Pattern(p) => p.is_match(text),
        }
    }

    /// The groups the shape captures from `text`.
    pub fn captures<'t>(&self, text: &'t str) -> Option<Captures<'t>> {
        self.regex()?.captures(text)
    }

    /// Whether the shape holds a group named `name`.
    pub fn has_group(&self, name: &str) -> bool {
        self.regex()
            .is_some_and(|r| r.capture_names().flatten().any(|n| n == name))
    }

    /// Why `text` does not have this shape, as a finding continues a
    /// sentence that opens with the text: `does not match …` or `fails at
    /// column …`. `text` names the shape of a literal.
    pub fn failure(&self, text: &str) -> String {
        match self {
            Shape::Text(Text::Literal(want)) => format!("must read `{}`", squash(want)),
            Shape::Text(Text::From(key)) => format!("must equal the frontmatter `{key}`"),
            Shape::Template(t) => format!("fails at {}", t.failure(text).describe(text)),
            Shape::Pattern(p) => format!("does not match `{}`", p.as_str()),
        }
    }

    /// How far into `text` the shape matched before it failed, for the
    /// furthest-failure choice between shapes: a template and a text know,
    /// a regex does not.
    pub fn reach(&self, text: &str) -> usize {
        match self {
            Shape::Template(t) => t.failure(text).reach,
            Shape::Text(Text::Literal(want)) => template::common_prefix(&squash(want), text),
            _ => 0,
        }
    }

    /// The shape as a finding or a label names it.
    pub fn label(&self) -> String {
        match self {
            Shape::Text(Text::Literal(t)) => t.clone(),
            Shape::Text(Text::From(k)) => format!("<{k}>"),
            Shape::Template(t) => t.source.clone(),
            Shape::Pattern(p) => format!("/{}/", p.as_str()),
        }
    }
}

// sokf:begin shape-of
/// The shape a rule map declares, from its `text`, `template` or `pattern`,
/// or `None` where it declares none. Two are refused: a rule has one shape.
/// `text` may be `{ from: <key> }` where `from` is allowed.
// sokf-implements: TL_22
// sokf-implements: TL_118
pub fn shape_of(
    node: &Spanned,
    types: &TypeRegistry,
    regexes: &RegexCache,
) -> Result<Option<Shape>, Problem> {
    let declared: Vec<&str> = ["text", "template", "pattern"]
        .into_iter()
        .filter(|k| node.get(k).is_some())
        .collect();
    if declared.len() > 1 {
        return Err(Problem::new(
            "TL_22",
            format!(
                "`{}` and `{}` beside each other: a rule has one shape",
                declared[0], declared[1]
            ),
        ));
    }
    if let Some(t) = node.get("text") {
        return Ok(match t.as_str() {
            Some(s) => Some(Shape::Text(Text::Literal(s.to_string()))),
            None => t
                .get("from")
                .and_then(|f| f.as_str())
                .map(|k| Shape::Text(Text::From(k.to_string()))),
        });
    }
    if let Some(t) = node.get("template").and_then(|t| t.as_str()) {
        return LineTemplate::parse(t, types).map(|t| Some(Shape::Template(t)));
    }
    if let Some(p) = node.get("pattern").and_then(|p| p.as_str()) {
        return regexes
            .get(p)
            .map(|r| Some(Shape::Pattern(r)))
            .map_err(|e| Problem::new("TL_118", e));
    }
    Ok(None)
}
// sokf:end shape-of

/// A string as a `heading` short form, or as `text` a template may hold:
/// a template when it has a hole, else exact text.
pub fn text_or_template(s: &str, types: &TypeRegistry) -> Result<Shape, Problem> {
    if has_hole(s) {
        LineTemplate::parse(s, types).map(Shape::Template)
    } else {
        Ok(Shape::Text(Text::Literal(s.to_string())))
    }
}

/// Whether `s` holds a `${`, not escaped as `$${`.
pub fn has_hole(s: &str) -> bool {
    let mut rest = s;
    while let Some(i) = rest.find("${") {
        if i == 0 || !rest[..i].ends_with('$') {
            return true;
        }
        rest = &rest[i + 2..];
    }
    false
}

/// A text squashed, each run of whitespace one space and the ends trimmed,
/// with the byte offset in the original text of each byte of the result
/// and one past the end, so a position found in the squashed text maps
/// back to the text as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Squashed {
    /// The squashed text.
    pub text: String,
    /// `map[i]` is the offset in the original text of byte `i`.
    pub map: Vec<usize>,
}

// sokf-implements: TL_23
// sokf-implements: TLMD_6
impl Squashed {
    /// `text` squashed.
    pub fn new(text: &str) -> Squashed {
        let mut out = String::with_capacity(text.len());
        let mut map = Vec::with_capacity(text.len() + 1);
        let mut pending: Option<usize> = None;
        for (i, c) in text.char_indices() {
            if c.is_whitespace() {
                if !out.is_empty() && pending.is_none() {
                    pending = Some(i);
                }
                continue;
            }
            if let Some(at) = pending.take() {
                out.push(' ');
                map.push(at);
            }
            let start = out.len();
            out.push(c);
            map.extend((0..out.len() - start).map(|k| i + k));
        }
        map.push(text.trim_end().len());
        Squashed { text: out, map }
    }

    /// The offset in the original text of offset `i` of the squashed one.
    pub fn original(&self, i: usize) -> usize {
        self.map
            .get(i)
            .copied()
            .unwrap_or(*self.map.last().unwrap_or(&0))
    }
}

/// `text` with each run of whitespace one space and the ends trimmed.
pub fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn a_hole_is_a_dollar_brace_not_escaped() {
        assert!(has_hole("a ${x}"));
        assert!(has_hole("$${a} ${b}"));
        assert!(!has_hole("$${a} and $5"));
        assert!(!has_hole("plain"));
    }

    #[test]
    fn a_literal_reaches_no_further_than_its_squashed_text() {
        let shape = Shape::Text(Text::Literal("a  b".into()));
        // The text holds the whole squashed literal, `a b`, and goes on.
        assert_eq!(shape.reach("a bc"), 3);
        assert_eq!(shape.reach("a b"), 3);
        assert_eq!(shape.reach("a c"), 2);
    }

    #[test]
    fn one_shape_per_rule() {
        let node = crate::yaml::parse("text: a\npattern: b\n", 1).unwrap();
        let e = shape_of(&node, &TypeRegistry::built_ins(), &RegexCache::new()).unwrap_err();
        assert!(e.message.contains("one shape"), "{e:?}");
        assert_eq!(e.criterion, "TL_22");
    }

    // sokf-tests: TL_23
    // sokf-tests: TL_24
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// The squashed text is `squash`, and each of its bytes maps back
        /// to the same byte of the text as written, a space to a space.
        #[test]
        fn a_squashed_text_maps_back(text in "[ a-z\\t\\n]{0,24}") {
            let s = Squashed::new(&text);
            prop_assert_eq!(&s.text, &squash(&text));
            for (i, b) in s.text.bytes().enumerate() {
                let o = s.original(i);
                if b == b' ' {
                    prop_assert!(text.as_bytes()[o].is_ascii_whitespace());
                } else {
                    prop_assert_eq!(text.as_bytes()[o], b);
                }
            }
        }
    }
}
