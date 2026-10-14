//! The shapes a schema declares: a rule's `text`, `template` or
//! `pattern`, the key it names, and a frontmatter value's template, the
//! `id`'s the one a new document is filled with.

use super::{Lowerer, bool_of, str_of, usize_of};
use crate::{
    Spanned,
    grammar::{FmFormat, Format, FrontmatterConstraint, IdTemplate, Shape, shape_of},
};

impl Lowerer<'_> {
    /// The shape `node` declares, a failure reported on the schema.
    pub fn shape(&mut self, node: &Spanned) -> Option<Shape> {
        match shape_of(node, &self.types, self.regexes) {
            Ok(s) => s,
            Err(e) => {
                self.fail_with(node.line, e);
                None
            }
        }
    }

    /// A `key` names a hole or a named group of `shape`, or of every shape
    /// of the format the rule binds.
    // sokf-implements: TL_31
    pub fn check_key(
        &mut self,
        line: usize,
        key: &str,
        shape: Option<&Shape>,
        format: Option<&Format>,
    ) {
        let shapes: Vec<&Shape> = match (shape, format) {
            (Some(s), _) => vec![s],
            (None, Some(f)) => f.shapes().map(|(_, s)| s).collect(),
            (None, None) => vec![],
        };
        if shapes.is_empty() || shapes.iter().any(|s| !s.has_group(key)) {
            self.fail(
                line,
                "TL_31",
                format!("`key` `{key}` names no hole or named group of the rule's shape"),
            );
        }
    }

    /// Grammar `frontmatter-constraint`, for the key `key`.
    pub fn frontmatter_constraint(&mut self, node: &Spanned, key: &str) -> FrontmatterConstraint {
        // The scaffold writes the id the template generates, so the `id`
        // template needs no example to speak for it.
        if key != "id" {
            self.explained(node, &["pattern", "template"], false);
        }
        self.one_voice(node);
        let shape = self.shape(node);
        // `id` is a key with a template like any other; its template is
        // also the one a new document is filled with, so it holds only the holes the
        // tool generates, and nothing narrows it beside the template.
        let width = usize_of(node, "seq-width");
        let mut id_template = None;
        match (&shape, key) {
            (Some(Shape::Template(t)), "id") => {
                for other in ["const", "enum", "format", "list", "min", "max"] {
                    if let Some(o) = node.get(other) {
                        // sokf-implements: TL_115
                        self.fail(
                            o.line,
                            "TL_115",
                            format!(
                                "`{other}` beside the `id` template: `id` takes `template` alone"
                            ),
                        );
                    }
                }
                match IdTemplate::from_template(t, width.unwrap_or(3), &self.lifecycle) {
                    Ok(parsed) => {
                        if width.is_some() && parsed.seq_width().is_none() {
                            // sokf-implements: TL_116
                            self.fail(
                                node.line,
                                "TL_116",
                                "`seq-width` beside a template with no `seq` hole",
                            );
                        }
                        id_template = Some(parsed);
                    }
                    Err(e) => self.fail(node.line, e.criterion, e.message),
                }
            }
            // A fixed id is the one a new document takes, as a template with
            // no hole would be.
            (None, "id") if node.get("const").is_some() => {
                match IdTemplate::fixed(&str_of(node, "const").unwrap_or_default()) {
                    Ok(fixed) => id_template = Some(fixed),
                    Err(e) => self.fail(node.line, e.criterion, e.message),
                }
            }
            _ => {
                if width.is_some() {
                    self.fail(
                        node.line,
                        "TL_116",
                        "`seq-width` applies to the `id` template alone",
                    );
                }
            }
        }
        if let Some(min) = usize_of(node, "min") {
            self.check_bounds(node.line, min, usize_of(node, "max"));
        }
        FrontmatterConstraint {
            common: self.common(node),
            const_: node.get("const").and_then(|c| c.scalar_text()),
            shape,
            enum_: node
                .get("enum")
                .and_then(|e| e.as_seq())
                .map(|items| items.iter().filter_map(|i| i.scalar_text()).collect()),
            // A format the core does not know is the host's: the merged
            // grammar admitted it.
            format: str_of(node, "format").map(|f| match f.as_str() {
                "url" => FmFormat::Url,
                "slug" => FmFormat::Slug,
                _ => FmFormat::Host(f),
            }),
            id_template,
            list: bool_of(node, "list"),
            min: usize_of(node, "min"),
            max: usize_of(node, "max"),
            optional: bool_of(node, "optional"),
            example: str_of(node, "example"),
            guidance: str_of(node, "guidance"),
        }
    }
}
