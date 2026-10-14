//! One line format: its shapes, its bans, and how a line matches it
//! (TL_42, TL_74 to TL_81).

use regex::Regex;

use super::{RegexCache, Segment, Shape, TypeRegistry, shape_of, template};
use crate::{Problem, Spanned, text};

/// One shape of a variant format (grammar `format-variant`).
#[derive(Debug, Clone)]
pub struct FormatVariant {
    /// The shape.
    pub shape: Shape,
    /// One conforming line, which a failure of this shape shows.
    pub example: String,
}

/// A line format (grammar `format`), its shapes compiled against the types
/// its layer sees.
#[derive(Debug, Clone)]
pub struct Format {
    /// The shape of a format without variants.
    pub shape: Option<Shape>,
    /// The named shapes of a variant format, in the order a line tries them.
    pub variants: Vec<(String, FormatVariant)>,
    /// The name of the hole or group that captures the line's key.
    pub key: Option<String>,
    /// `prohibited-patterns`: what no bound line may match.
    pub prohibited: Vec<Prohibition>,
    /// One conforming line of a format without variants.
    pub example: Option<String>,
    /// The entry as written, for a printed registry and a layer on top.
    pub raw: Spanned,
}

/// A key a shape captured, and the offset where the shape's match began,
/// which is where the key's anchor goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captured {
    /// The key.
    pub key: String,
    /// The start of the match.
    pub start: usize,
}

/// The key `name` that `shape` captures from `text`.
pub fn captured(shape: &Shape, name: &str, text: &str) -> Option<Captured> {
    let caps = shape.captures(text)?;
    Some(Captured {
        key: caps.name(name)?.as_str().to_string(),
        start: caps.get(0).map_or(0, |m| m.start()),
    })
}

/// The outcome of matching a line against a format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatMatch {
    /// The line conforms; its key, if the format captures one.
    Ok(Option<Captured>),
    /// The line does not have the shape, with the reason.
    Fail(String),
    /// The line has the shape and matches a ban, with the reason.
    Refused(String),
}

impl Format {
    /// Every example the format carries: its own, or each variant's.
    pub fn examples(&self) -> Vec<&str> {
        match &self.example {
            Some(e) => vec![e.as_str()],
            None => self
                .variants
                .iter()
                .map(|(_, v)| v.example.as_str())
                .collect(),
        }
    }

    /// Each shape of the format: its one shape, unnamed, or each variant's,
    /// by the variant's name.
    pub(crate) fn shapes(&self) -> impl Iterator<Item = (Option<&str>, &Shape)> {
        self.shape.iter().map(|s| (None, s)).chain(
            self.variants
                .iter()
                .map(|(name, v)| (Some(name.as_str()), &v.shape)),
        )
    }

    /// How far `text` follows the format before it fails: the furthest
    /// that any of its shapes reaches.
    pub fn reach(&self, text: &str) -> usize {
        self.shapes().map(|(_, s)| s.reach(text)).max().unwrap_or(0)
    }

    /// Whether a template of the format holds a hole of one of `types`.
    pub(super) fn uses_any(&self, types: &[&str]) -> bool {
        self.shapes().any(|(_, shape)| match shape {
            Shape::Template(t) => t
                .segments
                .iter()
                .any(|s| matches!(s, Segment::Hole { ty, .. } if types.contains(&ty.as_str()))),
            _ => false,
        })
    }

    /// The first example, which stands for the format where one is needed.
    pub fn first_example(&self) -> &str {
        self.example
            .as_deref()
            .or_else(|| self.variants.first().map(|(_, v)| v.example.as_str()))
            .unwrap_or("")
    }

    /// Match `text`, squashed, against the format. A failure is built from
    /// what the format holds, never from a sentence written to explain it:
    /// where the shape fails and its example, or what a ban matched and the
    /// example of the shape the line took.
    // sokf-implements: TL_42
    // sokf-implements: TL_74
    // sokf-implements: TL_80
    pub fn check(&self, text: &str) -> FormatMatch {
        self.check_as(text, "line")
    }

    /// As `check`, for a unit that a failure calls `noun`, such as `item`.
    pub fn check_as(&self, text: &str, noun: &str) -> FormatMatch {
        let (shape, example) = match self.matching_as(text, noun) {
            Ok(m) => m,
            Err(reason) => return FormatMatch::Fail(reason),
        };
        if let Some(reason) = refused_as(text, &self.prohibited, noun, Some(example)) {
            return FormatMatch::Refused(reason);
        }
        FormatMatch::Ok(self.key.as_ref().and_then(|k| captured(shape, k, text)))
    }

    /// The shape `text` has and its example, where it has one: the check
    /// alone, with no failure built.
    pub fn shape_for(&self, text: &str) -> Option<(&Shape, &str)> {
        match &self.shape {
            Some(shape) => shape
                .matches(text, |_| None)
                .then(|| (shape, self.example.as_deref().unwrap_or(""))),
            None => self
                .variants
                .iter()
                .find(|(_, v)| v.shape.matches(text, |_| None))
                .map(|(_, v)| (&v.shape, v.example.as_str())),
        }
    }

    /// Whether `text` has the format's shape and passes its bans, silently.
    pub fn conforms(&self, text: &str) -> bool {
        self.shape_for(text).is_some() && !self.prohibited.iter().any(|b| b.pattern.is_match(text))
    }

    /// The shape `text` has and its example, or why it has none. A variant
    /// format tries its variants in order; where none matches, the one the
    /// line follows furthest is the one it meant (TL_42).
    // sokf-implements: TL_42
    pub fn matching(&self, text: &str) -> std::result::Result<(&Shape, &str), String> {
        self.matching_as(text, "line")
    }

    /// As `matching`, for a unit that a failure calls `noun`.
    pub fn matching_as(
        &self,
        text: &str,
        noun: &str,
    ) -> std::result::Result<(&Shape, &str), String> {
        if let Some(found) = self.shape_for(text) {
            return Ok(found);
        }
        if let Some(shape) = &self.shape {
            let example = self.example.as_deref().unwrap_or("");
            return Err(format!(
                "{}; {} {noun} of this kind reads {example}",
                shape.failure(text),
                text::article(noun)
            ));
        }
        let reach: Vec<usize> = self
            .variants
            .iter()
            .map(|(_, v)| v.shape.reach(text))
            .collect();
        let best = reach.iter().copied().max().unwrap_or(0);
        let at: Vec<&(String, FormatVariant)> = self
            .variants
            .iter()
            .zip(&reach)
            .filter(|(_, r)| **r == best)
            .map(|(v, _)| v)
            .collect();
        match at.as_slice() {
            [] => Err("has no variants to match".into()),
            [(name, v)] => Err(format!(
                "does not match the `{name}` form: it {}; {} {noun} of this kind reads {}",
                v.shape.failure(text),
                text::article(noun),
                v.example
            )),
            many => {
                // A regex does not say where it fails; a template does.
                let at = if best > 0 {
                    format!(": each fails at column {}", template::column(text, best))
                } else {
                    String::new()
                };
                // Each form it names is one it may have meant, so each
                // shows its example.
                Err(format!(
                    "matches none of the forms {}{at}; {} {noun} of this kind reads {}",
                    text::code_list(many.iter().map(|(n, _)| n), ", "),
                    text::article(noun),
                    many.iter()
                        .map(|(_, v)| v.example.as_str())
                        .collect::<Vec<_>>()
                        .join(" or ")
                ))
            }
        }
    }
}

/// The failure of the first ban in `bans` that `text` matches: the span it
/// matched, the pattern, and, where the ban sits on a rule, that rule's
/// example. A ban needs no sentence of its own, because its pattern is
/// exactly what the author wrote.
// sokf-implements: TL_75
// sokf-implements: TL_76
// sokf-implements: TL_81
pub fn refused(text: &str, bans: &[Prohibition], example: Option<&str>) -> Option<String> {
    refused_as(text, bans, "line", example)
}

/// As `refused`, for a unit that the failure calls `noun`. The reason names
/// the span that a ban matched and gives the ban's message, which states
/// the rule, rather than its pattern.
// sokf-implements: TL_75
// sokf-implements: TL_76
pub fn refused_as(
    text: &str,
    bans: &[Prohibition],
    noun: &str,
    example: Option<&str>,
) -> Option<String> {
    bans.iter().find_map(|ban| {
        let span = ban.pattern.find(text)?.as_str();
        let message = ban.message.trim_end();
        Some(match example {
            Some(e) => {
                let mut article = text::article(noun).to_string();
                article[..1].make_ascii_uppercase();
                format!("`{span}` is refused: {message} {article} {noun} of this kind reads {e}")
            }
            None => format!("`{span}` is refused: {message}"),
        })
    })
}

/// `Prohibition` is one entry of `prohibited-patterns`: a pattern that no
/// governed text may match, and the message that states the rule as a fact
/// about the part.
#[derive(Debug, Clone)]
pub struct Prohibition {
    /// `pattern`: the regex.
    pub pattern: Regex,
    /// `message`: the rule, in one sentence.
    pub message: String,
}

/// `prohibitions` reads the entries of `prohibited-patterns` under `node`.
/// The grammar checks that each entry is a map of a `pattern` that compiles
/// and a `message`, so an entry that is not is skipped.
pub fn prohibitions(node: &Spanned, regexes: &RegexCache) -> Vec<Prohibition> {
    node.get("prohibited-patterns")
        .and_then(|b| b.as_seq())
        .unwrap_or(&[])
        .iter()
        .filter_map(|ban| {
            let pattern = regexes.get(ban.get("pattern")?.as_str()?).ok()?;
            let message = ban.get("message")?.as_str()?.to_string();
            Some(Prohibition { pattern, message })
        })
        .collect()
}

/// One shape of a format or a variant: `text`, `template` or `pattern`,
/// exactly one, `text` a string.
fn format_shape(
    node: &Spanned,
    types: &TypeRegistry,
    regexes: &RegexCache,
) -> std::result::Result<Shape, Problem> {
    if node.get("text").is_some_and(|t| t.as_str().is_none()) {
        return Err(Problem::new("TL_101", "a format's `text` is a string"));
    }
    // sokf-implements: TL_105
    shape_of(node, types, regexes)?.ok_or_else(|| {
        Problem::new(
            "TL_105",
            "a shape is one of `text`, `template` or `pattern`",
        )
    })
}

/// Lower a validated `format` entry against the types its layer sees. The
/// problems are `(line, message)`: a shape that does not compile, an
/// example where variants speak, none where they do not, and a key no
/// shape captures.
// sokf-implements: TL_42
pub(super) fn lower_format(
    entry: &Spanned,
    types: &TypeRegistry,
    regexes: &RegexCache,
) -> std::result::Result<Format, Vec<(usize, Problem)>> {
    let mut problems = Vec::new();
    let mut variants = Vec::new();
    if let Some(v) = entry.get("variants") {
        for (name, variant) in v.as_map().unwrap_or(&[]) {
            match format_shape(variant, types, regexes) {
                Ok(shape) => variants.push((
                    name.value.clone(),
                    FormatVariant {
                        shape,
                        example: variant
                            .get("example")
                            .and_then(|d| d.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    },
                )),
                Err(e) => problems.push((
                    variant.line,
                    e.prefixed(&format!("variant `{}`: ", name.value)),
                )),
            }
        }
    }
    let shape = if entry.get("variants").is_some() {
        None
    } else {
        match format_shape(entry, types, regexes) {
            Ok(s) => Some(s),
            Err(e) => {
                problems.push((entry.line, e));
                None
            }
        }
    };
    let example = entry
        .get("example")
        .and_then(|d| d.as_str())
        .map(str::to_string);
    // sokf-implements: TL_40
    // sokf-implements: TL_110
    match (&example, entry.get("variants")) {
        (Some(_), Some(_)) => problems.push((
            entry.line,
            Problem::new(
                "TL_110",
                "a format with `variants` has no `example` of its own: each variant's example speaks for its shape",
            ),
        )),
        (None, None) => problems.push((
            entry.line,
            Problem::new("TL_40", "a format with one shape carries an `example`"),
        )),
        _ => {}
    }
    let format = Format {
        shape,
        variants,
        key: entry
            .get("key")
            .and_then(|k| k.as_str())
            .map(str::to_string),
        prohibited: prohibitions(entry, regexes),
        example,
        raw: entry.clone(),
    };
    if let Some(k) = &format.key {
        for (name, s) in format.shapes() {
            if !s.has_group(k) {
                let whose =
                    name.map_or_else(|| "the shape".to_string(), |n| format!("variant `{n}`"));
                problems.push((
                    entry.line,
                    Problem::new(
                        "TL_31",
                        format!("`key` `{k}` names no hole or named group of {whose}"),
                    ),
                ));
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(format)
}
