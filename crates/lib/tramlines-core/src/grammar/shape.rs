//! The shape of one unit (`TL_22` to `TL_26`): exact `text`, a `template` with typed
//! holes, or a `pattern` regex, one key per meaning wherever a rule
//! declares a shape. Every shape reads the unit's text squashed, each run
//! of whitespace one space.

use regex::{Captures, Regex};

use super::{LineTemplate, RegexCache, Text, TypeRegistry, template};
use crate::{Problem, Spanned, text};

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
        self.failure_quoting(text, &Quote::same(text))
    }

    /// As `failure`, for a `text` that a format grammar read in another
    /// spelling: the column and the text found of a template come from
    /// `quote`, the text as written.
    pub fn failure_quoting(&self, text: &str, quote: &Quote<'_>) -> String {
        match self {
            Shape::Text(Text::Literal(want)) => format!("must read {}", text::code(&squash(want))),
            Shape::Text(Text::From(key)) => format!("must equal the frontmatter `{key}`"),
            Shape::Template(t) => format!("fails at {}", t.failure(text).quoted(quote)),
            Shape::Pattern(p) => format!("does not match {}", text::code(p.as_str())),
        }
    }

    /// The shape as a format grammar reads it: each literal text read
    /// through `read`, which reads a text in the grammar's spelling, a
    /// template read whole with each hole as a placeholder, and a pattern
    /// and a frontmatter key as they are.
    #[must_use]
    pub fn read_with(&self, read: &dyn Fn(&str) -> String) -> Shape {
        match self {
            Shape::Text(Text::Literal(want)) => Shape::Text(Text::Literal(read(want))),
            Shape::Template(t) => Shape::Template(t.read_with(read).unwrap_or_else(|| t.clone())),
            other => other.clone(),
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

    /// `text`, whose byte `i` stands for byte `map[i]` of another text,
    /// squashed, each byte of the result mapped to that other text. `map`
    /// holds one entry past the end of `text`.
    pub fn mapped(text: &str, map: &[usize]) -> Squashed {
        let own = Squashed::new(text);
        let to = |i: usize| map.get(i).copied().unwrap_or(*map.last().unwrap_or(&0));
        Squashed {
            map: own.map.iter().map(|&i| to(i)).collect(),
            text: own.text,
        }
    }

    /// The offset in the original text of offset `i` of the squashed one.
    pub fn original(&self, i: usize) -> usize {
        self.map
            .get(i)
            .copied()
            .unwrap_or(*self.map.last().unwrap_or(&0))
    }
}

/// How a finding quotes a text that a shape read in another spelling: the
/// text as written, and the offset in it of each offset of the text as
/// read, up to one past its end.
#[derive(Clone, Copy)]
pub struct Quote<'a> {
    /// The text as written.
    pub text: &'a str,
    /// The offset in `text` of an offset of the text as read.
    pub at: &'a dyn Fn(usize) -> usize,
}

impl<'a> Quote<'a> {
    /// A text quoted as it was read.
    pub fn same(text: &'a str) -> Quote<'a> {
        Quote { text, at: &|i| i }
    }

    /// The offset in the text as written of offset `i` of the text as
    /// read, on a character boundary.
    pub fn offset(&self, i: usize) -> usize {
        self.text
            .floor_char_boundary((self.at)(i).min(self.text.len()))
    }
}

impl std::fmt::Debug for Quote<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Quote")
            .field("text", &self.text)
            .finish_non_exhaustive()
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

    #[test]
    fn a_mapped_text_squashes_and_maps_to_the_other_text() {
        // `a  _b_` read from `a  *b*`: the same length, mapped one to one,
        // and `x` read from `\\x`, the backslash dropped.
        let s = Squashed::mapped("a  _b_", &[0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(s.text, "a _b_");
        assert_eq!(s.map, vec![0, 1, 3, 4, 5, 6]);
        let s = Squashed::mapped(" x", &[0, 2, 3]);
        assert_eq!(s.text, "x");
        assert_eq!(s.map, vec![2, 3]);
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
