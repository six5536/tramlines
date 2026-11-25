//! What a unit rule reads and captures: the name its keys are filed
//! under, its shape and its format as a unit reads them, and its bounds.

use std::ops::Range;

use super::{FormatRegistry, Unit, captured, nearest};
use crate::{
    grammar::{Format, Limit, Shape, row_reading},
    markdown,
};

impl Unit {
    /// The name that a key this rule captures is filed under: its
    /// `key-of`, else the first of `around`, the `rule-id`s of its block,
    /// its group, and its section, in that order. The rule's own
    /// `rule-id` files nothing. The check, the derivation, and the
    /// lookup of a reference all file a key through this.
    // sokf-implements: TLMD_89
    // sokf-implements: TLMD_236
    pub(crate) fn files_under<'a>(&'a self, around: &[Option<&'a str>]) -> Option<&'a str> {
        self.key_of.as_deref().or_else(|| nearest(around))
    }

    /// The `rule-id` that names the rule in a finding on its capture: its
    /// own, else the first of `around`, as `files_under` reads them.
    pub(crate) fn named_by<'a>(&'a self, around: &[Option<&'a str>]) -> Option<&'a str> {
        self.rule_id.as_deref().or_else(|| nearest(around))
    }

    /// The most units or lines the rule may take or count: the count of a
    /// hard `max`, and `None` for a soft `max` or no `max`, neither of
    /// which bounds anything.
    pub fn hard_max(&self) -> Option<usize> {
        self.max.as_ref().and_then(Limit::hard)
    }

    /// The format the rule binds, as `formats` defines it.
    pub(crate) fn format_in<'f>(&self, formats: &'f FormatRegistry) -> Option<&'f Format> {
        formats.get(self.format.as_deref()?)
    }

    /// Whether the rule, or the format it binds as `formats` defines it,
    /// bans any text.
    pub(crate) fn bans_in(&self, formats: &FormatRegistry) -> bool {
        !self.prohibited_patterns.is_empty()
            || self
                .format_in(formats)
                .is_some_and(|f| !f.prohibited.is_empty())
    }

    /// The key the rule captures from the squashed `text`: its own `key` of
    /// its own shape, or else the key of the shape of its format that
    /// `text` takes.
    pub(crate) fn key_in(&self, formats: &FormatRegistry, text: &str) -> Option<String> {
        if let Some(shape) = self.matcher() {
            return Some(captured(shape, self.key.as_deref()?, text)?.key);
        }
        let format = self.format_in(formats)?;
        let name = self.key.as_ref().or(format.key.as_ref())?;
        let (shape, _) = format.shape_for(text)?;
        Some(captured(shape, name, text)?.key)
    }

    /// The bytes of the squashed `text` that the key that `key_in` gives
    /// covers.
    pub(crate) fn key_span_in(&self, formats: &FormatRegistry, text: &str) -> Option<Range<usize>> {
        let (shape, name) = if let Some(shape) = self.matcher() {
            (shape, self.key.as_deref()?)
        } else {
            let format = self.format_in(formats)?;
            let name = self.key.as_ref().or(format.key.as_ref())?;
            (format.shape_for(text)?.0, name.as_str())
        };
        Some(shape.captures(text)?.name(name)?.range())
    }

    /// Whether the squashed `text` has the rule's shape, its own or else its
    /// format's. `value` gives the frontmatter value a `text` is bound to.
    pub(crate) fn fits(
        &self,
        formats: &FormatRegistry,
        text: &str,
        value: impl Fn(&str) -> Option<String>,
    ) -> bool {
        match self.matcher() {
            Some(shape) => shape.matches(text, value),
            None => self
                .format_in(formats)
                .is_some_and(|f| f.shape_for(text).is_some()),
        }
    }

    /// How far `text` follows the rule's shape, its own or else its
    /// format's, before it fails.
    pub(crate) fn reach(&self, formats: &FormatRegistry, text: &str) -> usize {
        match self.matcher() {
            Some(shape) => shape.reach(text),
            None => self.format_in(formats).map_or(0, |f| f.reach(text)),
        }
    }

    /// `text`, a text of the schema such as an example, as the rule reads
    /// a unit: its reading where the rule reads prose (`TLMD_259`), else
    /// as written.
    pub(crate) fn read_text(&self, text: &str) -> String {
        self.reading_of(text).text
    }

    /// `text`, a text of the schema, in the reading of the rule, as
    /// `read_text` reads it, with the map of its bytes.
    pub(crate) fn reading_of(&self, text: &str) -> markdown::Reading {
        self.reader()(text)
    }

    /// How the rule reads a text: cell by cell where it reads a table row,
    /// as a paragraph alone where it reads other prose, else as written.
    pub(crate) fn reader(&self) -> fn(&str) -> markdown::Reading {
        if self.reads_row {
            row_reading
        } else if self.reads_prose {
            markdown::reading
        } else {
            markdown::Reading::written
        }
    }

    /// The shape that a match of the rule uses: the shape as read, or the
    /// shape as written where no reading was lowered.
    pub(crate) fn matcher(&self) -> Option<&Shape> {
        self.read.as_ref().or(self.shape.as_ref())
    }

    /// The first text of the rule's `example`, which a failure quotes as the
    /// text of a valid unit.
    pub fn example(&self) -> Option<&str> {
        self.examples.first().map(String::as_str)
    }

    /// Whether the rule declares what a unit looks like, by its own shape
    /// or a format's.
    pub fn has_shape(&self) -> bool {
        self.shape.is_some() || self.format.is_some()
    }

    /// Whether the rule only counts what its guard selects: a guard and a
    /// count, and nothing that would check a unit (`TLMD_52` to `TLMD_57`).
    pub fn is_counter(&self) -> bool {
        self.select.is_some()
            && !self.has_shape()
            && self.key.is_none()
            && self.prohibited_patterns.is_empty()
            && self.blocks.is_none()
            && self.lines.is_none()
            && self.for_each_key_of.is_none()
    }

    /// Whether the rule is a production: a shape and no guard, taking the
    /// units its shape matches (`TLMD_52` to `TLMD_57`).
    pub fn is_production(&self) -> bool {
        self.select.is_none() && self.has_shape()
    }
}
