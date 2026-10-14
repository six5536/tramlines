//! The checks of a rule the generic check cannot see that every format
//! shares: the example a shape shows, and guidance that is not the example.

use super::{Lowerer, list_of, str_of};
use crate::Spanned;

impl Lowerer<'_> {
    /// A rule that declares a shape shows an example of it. A regex is a
    /// shape and not an instruction, and the example is the one voice a rule
    /// has: every failure it raises shows it, and the scaffold
    /// writes it. A unit rule bound to a format takes the format's, where
    /// `format_speaks`; the `format` of a frontmatter value checks the value
    /// and has no example. A rule that only selects, a count and a `text`
    /// state nothing.
    // sokf-implements: TL_66
    // sokf-implements: TL_67
    pub fn explained(&mut self, node: &Spanned, shapes: &[&str], format_speaks: bool) {
        if node.get("example").is_some()
            || (format_speaks && node.get("format").is_some())
            || node.get("text").is_some()
        {
            return;
        }
        let Some(key) = shapes.iter().find(|k| node.get(k).is_some()) else {
            return;
        };
        self.fail(
            node.line,
            "TL_66",
            format!(
                "a rule that declares `{key}` carries an `example` of it: a failure shows the example, and the scaffold writes it"
            ),
        );
    }

    /// Guidance says what the example cannot, so the two holding one
    /// sentence, or guidance that is one text of an `example` list, is
    /// guidance that says nothing.
    // sokf-implements: TL_68
    pub fn one_voice(&mut self, node: &Spanned) {
        if let Some(guidance) = str_of(node, "guidance") {
            self.one_voice_in(node.line, &guidance, list_of(node, "example"));
        }
    }

    /// As `one_voice`, for `guidance` and each of `examples`, which the
    /// format gathers from wherever its scaffold writes them after it.
    pub fn one_voice_in(
        &mut self,
        line: usize,
        guidance: &str,
        examples: impl IntoIterator<Item = String>,
    ) {
        if examples.into_iter().any(|e| guidance.trim() == e.trim()) {
            self.fail(
                line,
                "TL_68",
                "`guidance` and `example` hold the same words: guidance says what the example cannot",
            );
        }
    }

    /// A rule's `min` is at most its `max`.
    // sokf-implements: TL_98
    pub fn check_bounds(&mut self, line: usize, min: usize, max: Option<usize>) {
        if let Some(max) = max
            && min > max
        {
            self.fail(line, "TL_98", format!("`min` {min} is above `max` {max}"));
        }
    }
}
