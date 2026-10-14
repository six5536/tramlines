//! A section's heading as a unit rule: its shape, the key it names, and
//! its example.

use tramlines_core::grammar::{Scope, derivation, str_of};

use super::MdLowerer;
use crate::{
    Spanned,
    grammar::{Heading, Shape, Text, has_hole},
};

impl MdLowerer<'_> {
    /// A section's heading: a string of exact text, `{ from: <key> }`, or
    /// the rule a unit takes, with its shape, key and example.
    // sokf-implements: TLMD_27
    pub(super) fn heading(&mut self, section: &Spanned) -> Heading {
        let plain = |shape| Heading {
            shape,
            key: None,
            unique: Scope::Document,
            key_derives_from: None,
            example: None,
        };
        let Some(node) = section.get("heading") else {
            return plain(Shape::Text(Text::Literal(String::new())));
        };
        if let Some(s) = node.as_str() {
            // A heading that varies is a template, and a template speaks
            // through its example, which a string has no room for.
            if has_hole(s) {
                self.fail(
                    node.line,
                    "TL_66",
                    format!(
                        "heading `{s}` holds a hole: write `heading: {{ template: …, example: … }}`, since a template carries an example"
                    ),
                );
            }
            return plain(Shape::Text(Text::Literal(s.to_string())));
        }
        if let Some(key) = node
            .as_map()
            .filter(|m| m.len() == 1)
            .and_then(|_| str_of(node, "from"))
        {
            return plain(Shape::Text(Text::From(key)));
        }
        self.explained(node, &["template", "pattern"], true);
        let shape = self
            .shape(node)
            .unwrap_or_else(|| Shape::Text(Text::Literal(String::new())));
        // A former heading is rewritten to the heading's exact text, which
        // a heading that varies does not have.
        if let Some(r) = section.get("renamed-from") {
            // sokf-implements: TLMD_143
            self.fail(
                r.line,
                "TLMD_143",
                "`renamed-from` beside a heading that varies: a former heading is rewritten to exact text",
            );
        }
        let key = str_of(node, "key");
        if let Some(k) = &key {
            self.check_key(node.line, k, Some(&shape), None);
        }
        Heading {
            shape,
            key,
            unique: Scope::parse(str_of(node, "unique").as_deref()),
            key_derives_from: node.get("key-derives-from").map(derivation),
            example: str_of(node, "example"),
        }
    }
}
