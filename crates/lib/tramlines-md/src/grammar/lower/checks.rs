//! The semantic checks of a markdown schema the generic check cannot see:
//! references between rules, and what a unit rule may carry where it sits.

use std::collections::HashSet;

use super::MdLowerer;
use crate::{
    grammar::{
        Block, DerivationSource, KeyDerivation, Label, Schema, Segment, Shape, Unit, carries_labels,
    },
    markdown::BlockKind,
    text,
};

impl MdLowerer<'_> {
    /// Each rule of a `for-each-key-of` captures a key, resolved through
    /// `Schema::rule_captures_key`, the one definition of a rule's keys.
    // sokf-implements: TLMD_167
    fn check_pairs_with(&mut self, schema: &Schema, line: usize, rules: &[String]) {
        for r in rules {
            if let Err(why) = schema.rule_captures_key(r) {
                self.fail(line, "TLMD_167", why.message("for-each-key-of", r));
            }
        }
    }

    // sokf-implements: TL_15
    pub(super) fn check_references(&mut self, schema: &Schema) {
        let ids: HashSet<String> = schema.declared_names();
        let mut seen = HashSet::new();
        // The second declaration is the one that clashes. Only an id that
        // a reference can name clashes: a unit rule's `rule-id` names the
        // rule alone, and several rules can share one.
        for (id, line) in schema.referable_id_lines() {
            if !seen.insert(id.clone()) {
                self.fail(line, "TL_15", format!("`rule-id` `{id}` is declared twice"));
            }
        }
        for (i, s) in schema.sections.iter().enumerate() {
            if let Some(r) = &s.for_each_key_of {
                self.check_pairs_with(schema, s.common.line, r);
            }
            if let Some(d) = &s.heading.key_derives_from {
                self.check_derivation(schema, s.common.line, d);
            }
            self.check_parent(schema, &ids, i);
            self.check_blocks(s.blocks.as_deref(), schema);
        }
        if let Some(p) = &schema.preamble {
            self.check_blocks(p.blocks.as_deref(), schema);
        }
    }

    /// A derivation names a rule the schema declares, or a frontmatter
    /// key it declares.
    fn check_derivation(&mut self, schema: &Schema, line: usize, d: &KeyDerivation) {
        let missing = match &d.source {
            // A derivation reads a set of keys as a pairing does, so a name
            // that holds no key is refused, and one that only a unit rule's
            // `rule-id` carries says to declare `key-of` (TLMD_238).
            DerivationSource::Rule(r) => schema
                .rule_captures_key(r)
                .err()
                .map(|why| why.message("key-derives-from", r)),
            DerivationSource::From(k) => (!schema.core.frontmatter.iter().any(|(f, _)| f == k))
                .then(|| format!("`key-derives-from` names no frontmatter key `{k}`")),
        };
        if let Some(what) = missing {
            // sokf-implements: TL_117
            self.fail(line, "TL_117", what);
        }
    }

    /// A `parent` names a section of a smaller level, listed before this
    /// one with only its own subsections between, so the order check can
    /// return from a subsection to the next occurrence of its parent.
    // sokf-implements: TLMD_34
    fn check_parent(&mut self, schema: &Schema, ids: &HashSet<String>, i: usize) {
        let s = &schema.sections[i];
        let Some(p) = &s.parent else {
            return;
        };
        let Some(pi) = schema
            .sections
            .iter()
            .position(|o| o.rule_id.as_deref() == Some(p.as_str()))
        else {
            let what = if ids.contains(p) {
                format!("`parent` names `{p}`, which is not a section")
            } else {
                format!("`parent` names no rule `{p}`")
            };
            self.fail(s.common.line, "TLMD_34", what);
            return;
        };
        let parent = &schema.sections[pi];
        if parent.level >= s.level {
            // sokf-implements: TLMD_156
            self.fail(
                s.common.line,
                "TLMD_156",
                format!(
                    "`parent` `{p}` is level {}; a section lies inside one of a smaller level, and this one is level {}",
                    parent.level, s.level
                ),
            );
        } else if pi > i
            || schema.sections[pi + 1..i]
                .iter()
                .any(|o| o.level <= parent.level)
        {
            // sokf-implements: TLMD_157
            self.fail(
                s.common.line,
                "TLMD_157",
                format!(
                    "section {} is not listed after its `parent` `{p}` with only its subsections between",
                    text::code(&s.label())
                ),
            );
        }
    }

    /// A rule that declares `label` names a label of the schema's
    /// `labels`, so the schema declares some.
    // sokf-implements: TLMD_241
    fn check_label(&mut self, schema: &Schema, line: usize, label: Option<Label>) {
        if label.is_some() && schema.labels.is_empty() {
            self.fail(
                line,
                "TLMD_241",
                "`label` on a rule of a schema that declares no `labels`: declare the key of each label under `labels`",
            );
        }
    }

    fn check_blocks(&mut self, blocks: Option<&[Block]>, schema: &Schema) {
        for b in blocks.unwrap_or(&[]) {
            self.check_label(schema, b.common.line, b.label);
            self.check_bounds(b.common.line, b.min, b.max.as_ref());
            // sokf-implements: TLMD_153
            let requires = match b.kind {
                BlockKind::Code if b.lang.is_none() => Some("a `code` block requires `lang`"),
                BlockKind::Table if b.columns.is_none() => {
                    Some("a `table` block requires `columns`")
                }
                BlockKind::Group if b.blocks.is_none() => Some("a `group` block requires `blocks`"),
                _ => None,
            };
            if let Some(what) = requires {
                self.fail(b.common.line, "TLMD_153", what);
            }
            self.check_example_columns(b);
            // A code block holds one fence, and the scaffold writes one fence
            // for each text of a list, so only a code block that repeats can
            // hold the fences that a list of several texts writes.
            if b.kind == BlockKind::Code
                && !b.repeat
                && (b.examples.len() > 1 || b.units().any(|u| u.examples.len() > 1))
            {
                self.fail(
                    b.common.line,
                    "TLMD_183",
                    "a list `example` on a `code` block writes one fence for each text, so the block declares `repeat: true`",
                );
            }
            // sokf-implements: TLMD_47
            if b.rules.is_some() && (b.each.is_some() || b.sequence.is_some()) {
                self.fail(
                    b.common.line,
                    "TLMD_47",
                    "`rules` beside `each` or `sequence`: a block takes one of the three",
                );
            }
            for u in b.units() {
                if u.lines.is_some() && !matches!(b.kind, BlockKind::Code | BlockKind::Html) {
                    let message = format!(
                        "`lines` applies to a `code` or `html` block, not {} `{}` block",
                        text::article(b.kind.name_in(&self.dialect)),
                        b.kind.name_in(&self.dialect)
                    );
                    self.fail(u.common.line, "TLMD_90", message);
                }
                // A label stands straight after the key and before more
                // text (TLMD_243, TLMD_244), so a unit that is only its key
                // has no place for one.
                // sokf-implements: TLMD_269
                if carries_labels(b.kind)
                    && b.label_of(Some(u)) == Some(Label::Required)
                    && let Some(key) = only_key(schema, u)
                {
                    self.fail(
                        u.common.line,
                        "TLMD_269",
                        format!(
                            "a unit of this rule is only its key, the hole `{key}`, which leaves no place for a label, and the rule requires one: give the template text after the key, or declare `label: optional`"
                        ),
                    );
                }
                self.check_unit(u, Some(b.kind), schema);
            }
            self.check_blocks(b.blocks.as_deref(), schema);
        }
    }

    /// The `example-columns` of a table block: at least one column, and
    /// only beside `columns: []` and `columns-open: true`, whose check
    /// accepts the header that the scaffold writes from it. The derivation
    /// holds each row that the scaffold writes to the columns (`TLMD_208`).
    // sokf-implements: MD-2_AC-8
    fn check_example_columns(&mut self, b: &Block) {
        let Some(header) = &b.example_columns else {
            return;
        };
        if b.kind != BlockKind::Table {
            return;
        }
        let declared = b.columns.as_ref().is_some_and(|c| !c.is_empty());
        let why = if header.is_empty() {
            Some("names no column, so the scaffold writes a header with no cell")
        } else if declared {
            Some("the block declares `columns`, which the scaffold writes as its header")
        } else if !b.columns_open {
            Some(
                "the block does not declare `columns-open: true`, so its check accepts no header but an empty one",
            )
        } else {
            None
        };
        if let Some(why) = why {
            let message = if header.is_empty() {
                format!("`example-columns` {why}")
            } else {
                format!(
                    "`example-columns` names the header of a table with `columns: []` and `columns-open: true`, and {why}"
                )
            };
            // sokf-implements: TLMD_207
            self.fail(b.common.line, "TLMD_207", message);
        }
    }

    /// One unit rule. `kind` is the enclosing block's kind, or `None` for a
    /// rule under `lines`, whose unit is one line of text.
    // sokf-implements: TLMD_90
    fn check_unit(&mut self, u: &Unit, kind: Option<BlockKind>, schema: &Schema) {
        self.check_label(schema, u.common.line, u.label);
        // A rule pairs what it captures, so it captures something, and the
        // rule it pairs with is one the schema declares.
        // sokf-implements: TLMD_98
        if let Some(r) = &u.for_each_key_of {
            self.check_pairs_with(schema, u.common.line, r);
            if !schema.unit_captures_key(u) {
                self.fail(
                    u.common.line,
                    "TLMD_98",
                    "`for-each-key-of` on a rule that captures no key: declare `key`, or bind a `format` that declares one",
                );
            }
        }
        if let Some(Shape::Text(_)) = &u.shape
            && let Some(k) = kind
            && !matches!(
                k,
                BlockKind::Prose
                    | BlockKind::Blockquote
                    | BlockKind::BulletList
                    | BlockKind::NumberedList
            )
        {
            let message = format!(
                "`text` applies to a paragraph, an item or a line, not {} `{}` block",
                text::article(k.name_in(&self.dialect)),
                k.name_in(&self.dialect)
            );
            self.fail(u.common.line, "TLMD_42", message);
        }
        if u.shape.is_some() && u.format.is_some() {
            self.fail(
                u.common.line,
                "TL_22",
                "a shape beside `format`: a rule has one shape, its own or the format's",
            );
        }
        let format = u.format_in(&schema.core.formats);
        if let Some(name) = &u.format {
            match format {
                // sokf-implements: TL_125
                None if self.unloaded.contains(name) => {}
                // sokf-implements: TL_97
                None => self.fail(
                    u.common.line,
                    "TL_97",
                    format!("`format` names no format `{name}`"),
                ),
                // sokf-implements: TL_113
                Some(f) if f.key.is_some() && u.key.is_some() => self.fail(
                    u.common.line,
                    "TL_113",
                    format!("`key` beside format `{name}`, which captures a key itself"),
                ),
                Some(_) => {}
            }
        }
        // A key is a name: of a hole of the rule's template, or of a named
        // group of its pattern, or of every shape of its format.
        // sokf-implements: TL_31
        if let Some(k) = &u.key {
            if u.shape.is_some() || format.is_some() {
                self.check_key(u.common.line, k, u.shape.as_ref(), format);
            } else {
                self.fail(
                    u.common.line,
                    "TL_31",
                    format!("`key` `{k}` names a hole of a shape, and the rule declares none"),
                );
            }
        }
        // A rule captures a key of its own or its format's; a format that
        // resolves to nothing has its finding under `TL_97`.
        let no_key =
            u.key.is_none() && (u.format.is_none() || format.is_some_and(|f| f.key.is_none()));
        if let Some(d) = &u.key_derives_from {
            self.check_derivation(schema, u.common.line, d);
            if no_key {
                // sokf-implements: TL_96
                self.fail(
                    u.common.line,
                    "TL_96",
                    "`key-derives-from` on a rule that captures no key",
                );
            }
        }
        // `key-optional` waives a key, so there has to be one to waive.
        // sokf-implements: TL_47
        if u.key_optional && !schema.unit_captures_key(u) {
            self.fail(
                u.common.line,
                "TL_47",
                "`key-optional` without a `key` or a `format` that declares one",
            );
        }
        // `key-of` names the set that the rule's keys go to, so the rule
        // captures one.
        if u.key_of.is_some() && no_key {
            // sokf-implements: TLMD_237
            self.fail(
                u.common.line,
                "TLMD_237",
                "`key-of` on a rule that captures no key: declare `key`, or bind a `format` that declares one",
            );
        }
        if u.lines_open && u.lines.is_none() {
            // sokf-implements: TL_103
            self.fail(u.common.line, "TL_103", "`lines-open` without `lines`");
        }
        self.check_bounds(u.common.line, u.min, u.max.as_ref());
        self.check_blocks(u.blocks.as_deref(), schema);
        for line in u.lines.iter().flatten() {
            self.check_unit(line, None, schema);
        }
    }
}

/// The key that is the whole of a unit of `u`: the name of the one hole of
/// a template, its own or its format's, that is nothing but that hole and
/// captures the rule's key.
fn only_key<'a>(schema: &'a Schema, u: &'a Unit) -> Option<&'a str> {
    let (shape, key) = match (&u.shape, &u.format) {
        (Some(shape), _) => (shape, u.key.as_deref()?),
        (None, Some(name)) => {
            let format = schema.core.formats.get(name)?;
            let key = u.key.as_deref().or(format.key.as_deref())?;
            // A format with variants is only its key where every variant is.
            let shapes: Vec<&Shape> = format
                .shape
                .iter()
                .chain(format.variants.iter().map(|(_, v)| &v.shape))
                .collect();
            return (!shapes.is_empty() && shapes.iter().all(|s| is_key(s, key))).then_some(key);
        }
        (None, None) => return None,
    };
    is_key(shape, key).then_some(key)
}

/// Whether `shape` is a template that is only the hole `key`.
fn is_key(shape: &Shape, key: &str) -> bool {
    match shape {
        Shape::Template(t) => matches!(
            t.segments.as_slice(),
            [Segment::Hole { name: Some(n), .. }] if n == key
        ),
        _ => false,
    }
}
