//! The command's own texts for a limit's lead: its English, then the
//! entries of the file that `--texts` names, in the shape of sokf's file
//! of texts, and the host that words each limit and each label from them
//! in the language of the schema. The command holds no words of its own
//! for a label.

use std::{collections::BTreeMap, path::Path};

use tramlines_core::frontmatter::Host as FrontmatterHost;
use tramlines_md::{
    Locator, Spanned, Yaml,
    check::{
        CheckOptions, CheckOutput, Counts, DocView, Host, LabelFacts, LimitFacts, LimitSubject,
        NullHost, RuleName, ScaffoldFront, Subject,
    },
    grammar::{LimitKind, States},
    markdown::{BlockKind, Dialect},
    text, yaml,
};

use crate::{Stop, load};

/// The command's own texts in English.
const ENGLISH: &str = include_str!("texts.yaml");

/// Each unit that a limit counts, whose forms the entry `units` gives.
const UNITS: &[&str] = &[
    "paragraph",
    "item",
    "row",
    "line",
    "fence",
    "image",
    "definition",
    "repetition",
    "value",
    "unit",
];

/// One entry of a language: words, or the two forms of a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Text {
    /// The words of a template.
    Words(String),
    /// The form for a count of 1, and the form for every other count and
    /// for a ban.
    Forms { one: String, other: String },
}

/// The command's texts, each by its language, as a tag in lower case, and
/// its entry, such as `limit.hard` or `units.item`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    entries: BTreeMap<(String, String), Text>,
}

impl Texts {
    /// The command's own texts alone.
    // sokf-implements: CLI-16_AC-4
    pub fn english() -> Texts {
        let mut texts = Texts::default();
        texts.add(parse(ENGLISH).expect("the command's own texts parse"));
        texts
    }

    /// The command's own texts, then each entry of the file at `path`, in
    /// place of the same entry of its own. A file that isn't a file of
    /// texts stops the run.
    // sokf-implements: CLI-16_AC-5
    // sokf-implements: CLI-16_AC-10
    // sokf-implements: CLI-16_AC-11
    pub fn load(path: Option<&Path>) -> Result<Texts, Stop> {
        let mut texts = Texts::english();
        if let Some(path) = path {
            let shown = path.display();
            let entries = parse(&load::read(path)?).map_err(|fault| match fault {
                Fault::Yaml(why) => Stop::plain(format!("{shown}: {why}")),
                Fault::At(line, why) => Stop::plain(format!("{shown}:{line}: {why}")),
            })?;
            texts.add(entries);
        }
        Ok(texts)
    }

    fn add(&mut self, entries: Vec<Entry>) {
        for e in entries {
            self.entries.insert((e.language, e.entry), e.text);
        }
    }

    /// The entry `entry` of `language`: of the tag without case, else of
    /// the tag less its last subtags, down to the language alone.
    // sokf-implements: CLI-16_AC-6
    fn get(&self, language: &str, entry: &str) -> Option<&Text> {
        lookup(language)
            .into_iter()
            .find_map(|tag| self.entries.get(&(tag, entry.to_string())))
    }

    fn words(&self, language: &str, entry: &str) -> Option<&str> {
        match self.get(language, entry)? {
            Text::Words(w) => Some(w),
            Text::Forms { .. } => None,
        }
    }

    fn forms(&self, language: &str, unit: &str) -> Option<(&str, &str)> {
        match self.get(language, &format!("units.{unit}"))? {
            Text::Forms { one, other } => Some((one, other)),
            Text::Words(_) => None,
        }
    }

    /// The lead of the limit that `facts` describe: its template with its
    /// count and its unit, or `None` where the schema declares no language,
    /// or the texts lack the template or the unit in that language, so
    /// that the engine writes the limit in the syntax of the schema.
    // sokf-implements: CLI-16_AC-1
    // sokf-implements: CLI-16_AC-3
    // sokf-implements: CLI-16_AC-7
    // sokf-implements: CLI-16_AC-8
    pub fn lead(&self, facts: &LimitFacts<'_>) -> Option<String> {
        let lang = facts.language?;
        let template = match (facts.kind, facts.count) {
            (LimitKind::Hard, 0) => "limit.ban",
            (LimitKind::Hard, _) => "limit.hard",
            (LimitKind::Soft, _) => "limit.soft",
        };
        let template = self.words(lang, template)?;
        let count = facts.count.to_string();
        // A template without `{unit}` needs no word for the unit.
        if !placeholders(template).contains(&"unit") {
            return Some(fill(template, &[("count", &count)]));
        }
        // The word for the unit, in the form for the count: a ban, of 0,
        // takes the form `other`. `None` where the language lacks it.
        let noun = || {
            let (one, other) = self.forms(lang, unit_of(&facts.subject))?;
            Some(if facts.count == 1 { one } else { other })
        };
        let unit = match facts.subject {
            LimitSubject::Rule {
                name: Some(RuleName::Format(name) | RuleName::RuleId(name)),
                ..
            } => {
                let rule_unit = self.words(lang, "rule-unit")?;
                // A `rule-unit` without `{unit}` needs no word for the unit.
                let noun = if placeholders(rule_unit).contains(&"unit") {
                    noun()?
                } else {
                    ""
                };
                fill(rule_unit, &[("name", name), ("unit", noun)])
            }
            _ => noun()?.to_string(),
        };
        Some(fill(template, &[("count", &count), ("unit", &unit)]))
    }
}

impl Texts {
    /// The words of the label that `facts` describe: the entry
    /// `labels.<key>` of the schema's language, or `None` where the schema
    /// declares no language or the texts lack the entry, so that the
    /// engine takes the label's key as its words.
    // sokf-implements: CLI-17_AC-1
    // sokf-implements: CLI-17_AC-2
    pub fn label(&self, facts: &LabelFacts<'_>) -> Option<String> {
        let lang = facts.language?;
        self.words(lang, &format!("labels.{}", facts.key))
            .map(str::to_string)
    }
}

/// The unit that a limit on `subject` counts, as the engine counts it.
// sokf-implements: CLI-16_AC-2
fn unit_of(subject: &LimitSubject<'_>) -> &'static str {
    let block = match subject {
        LimitSubject::Key(_) => return "value",
        LimitSubject::Rule {
            counts: Counts::Lines,
            ..
        } => return "line",
        LimitSubject::Rule { block, .. } | LimitSubject::Block(block) => *block,
    };
    match block {
        BlockKind::Prose | BlockKind::Blockquote => "paragraph",
        BlockKind::BulletList | BlockKind::NumberedList => "item",
        BlockKind::Table => "row",
        // A code block's own limit counts its fences.
        BlockKind::Code => "fence",
        BlockKind::Image => "image",
        BlockKind::References => "definition",
        BlockKind::Group => "repetition",
        _ => "unit",
    }
}

/// The tags to try for `language`, in lower case: the whole tag, then the
/// tag less each last subtag in turn, skipping a shorter tag that ends in
/// a single-letter subtag, which only introduces an extension.
fn lookup(language: &str) -> Vec<String> {
    let lower = language.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split('-').collect();
    (1..=parts.len())
        .rev()
        .filter(|&n| n == parts.len() || n == 1 || parts[n - 1].len() > 1)
        .map(|n| parts[..n].join("-"))
        .collect()
}

/// The null host, whose answers `Words` gives to every question but the
/// lead of a limit and the words of a label.
static NULL: NullHost = NullHost;

/// The host of `tramlines validate` and `tramlines new`, which words each
/// limit and each label from the texts and answers every other question as
/// the null host does.
pub struct Words<'a>(pub &'a Texts);

impl Host for Words<'_> {
    fn dialect(&self) -> Dialect {
        NULL.dialect()
    }

    fn image(&self, reference: &str) -> Subject {
        NULL.image(reference)
    }

    fn region(&self, target: &str) -> Subject {
        NULL.region(target)
    }

    fn scaffold_front(&self, states: &States) -> Option<ScaffoldFront> {
        NULL.scaffold_front(states)
    }

    fn limit_lead(&self, facts: &LimitFacts<'_>) -> Option<String> {
        self.0.lead(facts)
    }

    fn label_words(&self, facts: &LabelFacts<'_>) -> Option<String> {
        self.0.label(facts)
    }

    // sokf-implements: CLI-17_AC-5
    fn scaffold_label(&self, labels: &[String]) -> Option<String> {
        NULL.scaffold_label(labels)
    }

    fn records_accepted(&self) -> bool {
        NULL.records_accepted()
    }

    fn check(
        &self,
        doc: &DocView<'_>,
        states: &States,
        options: CheckOptions,
        locator: &Locator,
        out: &mut CheckOutput,
    ) {
        NULL.check(doc, states, options, locator, out);
    }
}

impl FrontmatterHost for Words<'_> {
    fn scalar_format(&self, name: &str, value: &str) -> Option<Result<(), String>> {
        NULL.scalar_format(name, value)
    }

    fn reserved_keys(&self) -> &[&str] {
        NULL.reserved_keys()
    }
}

/// One entry of a file of texts: its language in lower case, the entry,
/// and its text.
struct Entry {
    language: String,
    entry: String,
    text: Text,
}

/// Why a file of texts doesn't read: YAML that doesn't parse, or a fault
/// at a line.
#[derive(Debug)]
enum Fault {
    Yaml(String),
    At(usize, String),
}

/// The entries of the file of texts `text`, a map from language tags to
/// their entries. It refuses any other shape, a language that the file
/// gives twice, and an entry that the command defines but whose value is of
/// another shape, holds a placeholder that the entry doesn't take, or
/// lacks the count or the name that the entry needs. It ignores every
/// other entry, such as sokf's `description`.
// sokf-implements: CLI-16_AC-9
fn parse(text: &str) -> Result<Vec<Entry>, Fault> {
    let doc = yaml::parse(text, 1).map_err(|e| Fault::Yaml(e.to_string()))?;
    if matches!(doc.value, Yaml::Null) {
        return Ok(Vec::new());
    }
    let Some(languages) = doc.as_map() else {
        return Err(Fault::At(
            doc.line,
            "the file is no map from language tags to their texts".into(),
        ));
    };
    let mut out = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (tag, value) in languages {
        if !text::is_language_tag(&tag.value) {
            return Err(Fault::At(
                tag.line,
                format!("`{}` is no language tag in the syntax of BCP 47", tag.value),
            ));
        }
        // A tag counts without case, so `EN` repeats `en`.
        let language = tag.value.to_ascii_lowercase();
        if let Some(first) = seen.insert(language.clone(), tag.line) {
            return Err(Fault::At(
                tag.line,
                format!(
                    "`{}` gives the language of line {first} again; give each language once",
                    tag.value
                ),
            ));
        }
        let mut push = |entry: String, text: Text| {
            out.push(Entry {
                language: language.clone(),
                entry,
                text,
            });
        };
        for (key, value) in map_of(value, &tag.value)? {
            match key.value.as_str() {
                "rule-unit" => push(
                    key.value.clone(),
                    Text::Words(words(value, &key.value, &["name", "unit"], &["name"])?),
                ),
                "limit" => {
                    for (kind, value) in map_of(value, "limit")? {
                        if !["hard", "soft", "ban"].contains(&kind.value.as_str()) {
                            continue;
                        }
                        let entry = format!("limit.{}", kind.value);
                        // A ban states no count, so only a hard and a soft
                        // limit need one.
                        let needs: &[&str] = if kind.value == "ban" { &[] } else { &["count"] };
                        let text = words(value, &entry, &["count", "unit"], needs)?;
                        push(entry, Text::Words(text));
                    }
                }
                // sokf-implements: CLI-17_AC-3
                "labels" => {
                    for (key, value) in map_of(value, "labels")? {
                        let entry = format!("labels.{}", key.value);
                        push(entry.clone(), Text::Words(label_words(value, &entry)?));
                    }
                }
                "units" => {
                    for (unit, value) in map_of(value, "units")? {
                        if !UNITS.contains(&unit.value.as_str()) {
                            continue;
                        }
                        let entry = format!("units.{}", unit.value);
                        push(entry.clone(), forms(value, &entry)?);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(out)
}

/// The entries of `value`, the value of `what`, which must be a map.
fn map_of<'v>(value: &'v Spanned, what: &str) -> Result<&'v [(Spanned<String>, Spanned)], Fault> {
    value
        .as_map()
        .ok_or_else(|| Fault::At(value.line, format!("`{what}` is no map")))
}

/// The string `value` of the entry `entry`, which holds no placeholder
/// other than `allowed` and each placeholder of `needs`.
fn words(value: &Spanned, entry: &str, allowed: &[&str], needs: &[&str]) -> Result<String, Fault> {
    let fail = |why: String| Fault::At(value.line, why);
    let Some(text) = value.as_str() else {
        return Err(fail(format!("`{entry}` is no string")));
    };
    let held = placeholders(text);
    if let Some(name) = held.iter().find(|n| !allowed.contains(n)) {
        let takes = if allowed.is_empty() {
            "takes no placeholder".to_string()
        } else {
            format!(
                "takes {}",
                text::code_list(allowed.iter().map(|a| format!("{{{a}}}")), ", ")
            )
        };
        return Err(fail(format!(
            "`{entry}` holds `{{{name}}}`, which is no placeholder of it; it {takes}"
        )));
    }
    if let Some(name) = needs.iter().find(|n| !held.contains(n)) {
        return Err(fail(format!(
            "`{entry}` holds no `{{{name}}}`, which it needs"
        )));
    }
    Ok(text.to_string())
}

/// The words of the label `entry`, a string that a label can hold between
/// its brackets: not blank, with no square bracket and no placeholder, and
/// not beginning with `^`, which opens a footnote reference.
fn label_words(value: &Spanned, entry: &str) -> Result<String, Fault> {
    let text = words(value, entry, &[], &[])?;
    if text.trim().is_empty() || text.contains(['[', ']']) || text.trim().starts_with('^') {
        return Err(Fault::At(
            value.line,
            format!(
                "`{entry}` holds no words that a label can hold between its brackets: give words with no `[` or `]` that don't begin with `^`"
            ),
        ));
    }
    Ok(text)
}

/// The forms `one` and `other` of the unit `entry`, a map that holds the
/// two strings. It ignores any other form, such as `few`.
fn forms(value: &Spanned, entry: &str) -> Result<Text, Fault> {
    let shape = || {
        Fault::At(
            value.line,
            format!("`{entry}` is no map of the strings `one` and `other`"),
        )
    };
    value.as_map().ok_or_else(shape)?;
    let form = |name: &str| -> Result<String, Fault> {
        let v = value.get(name).ok_or_else(shape)?;
        words(v, &format!("{entry}.{name}"), &[], &[])
    };
    Ok(Text::Forms {
        one: form("one")?,
        other: form("other")?,
    })
}

/// The name of the placeholder whose `{` stands just before `after`, and
/// where its `}` lies in `after`, or `None` where no placeholder starts
/// there. A name is one or more letters, digits, `-`, or `_`, so `{}` is
/// text.
fn name_at(after: &str) -> Option<(usize, &str)> {
    let end = after.find('}')?;
    let name = &after[..end];
    (!name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
    .then_some((end, name))
}

/// The name of each placeholder `{name}` of `text`.
fn placeholders(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        match name_at(after) {
            Some((end, name)) => {
                out.push(name);
                rest = &after[end + 1..];
            }
            None => rest = after,
        }
    }
    out
}

/// `template` with each placeholder `{name}` of `values` replaced by its
/// value, in one pass, so that no value is read as a template.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let hit = name_at(after).and_then(|(end, name)| {
            values
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| (end, *v))
        });
        if let Some((end, value)) = hit {
            out.push_str(value);
            rest = &after[end + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_is_looked_up_without_case_and_then_without_its_last_subtags() {
        assert_eq!(lookup("de-CH-1996"), ["de-ch-1996", "de-ch", "de"]);
        assert_eq!(lookup("EN"), ["en"]);
        // A single-letter subtag only introduces an extension.
        assert_eq!(lookup("en-a-bbb"), ["en-a-bbb", "en"]);
        // The whole tag is tried first, whatever its last subtag.
        assert_eq!(lookup("de-x-a"), ["de-x-a", "de"]);
    }

    #[test]
    fn a_template_is_filled_in_one_pass() {
        assert_eq!(
            fill(
                "At most {count} {unit}.",
                &[("count", "3"), ("unit", "{count}")]
            ),
            "At most 3 {count}."
        );
        assert_eq!(fill("{x} {unit} {", &[("unit", "items")]), "{x} items {");
        assert_eq!(
            placeholders("a {count} {not here} {} {unit}{"),
            ["count", "unit"]
        );
    }

    #[test]
    fn the_english_holds_each_template_and_each_unit() {
        let t = Texts::english();
        for entry in ["limit.hard", "limit.soft", "limit.ban", "rule-unit"] {
            assert!(t.words("en", entry).is_some(), "{entry}");
        }
        for unit in UNITS {
            assert!(t.forms("en-GB", unit).is_some(), "{unit}");
        }
        assert!(t.words("de", "limit.hard").is_none());
    }

    // Each unit that a limit can count is one whose forms a file gives.
    #[test]
    fn each_unit_of_a_limit_is_a_unit_of_the_texts() {
        let kinds = [
            BlockKind::Prose,
            BlockKind::BulletList,
            BlockKind::NumberedList,
            BlockKind::Table,
            BlockKind::Code,
            BlockKind::Blockquote,
            BlockKind::Image,
            BlockKind::Region,
            BlockKind::Html,
            BlockKind::ThematicBreak,
            BlockKind::References,
            BlockKind::Group,
            BlockKind::Heading,
        ];
        let mut subjects = vec![LimitSubject::Key("tags")];
        for block in kinds {
            subjects.push(LimitSubject::Block(block));
            for counts in [Counts::Units, Counts::Lines] {
                subjects.push(LimitSubject::Rule {
                    block,
                    counts,
                    name: None,
                });
            }
        }
        for subject in subjects {
            assert!(UNITS.contains(&unit_of(&subject)), "{subject:?}");
        }
    }

    #[test]
    fn a_unit_follows_what_the_engine_counts() {
        let t = Texts::english();
        let lead =
            |kind, count, subject| t.lead(&LimitFacts::new(kind, count, subject, Some("en")));
        let table = LimitSubject::Block(BlockKind::Table);
        assert_eq!(
            lead(LimitKind::Hard, 3, table).as_deref(),
            Some(
                "At most 3 rows. Never leave out or merge content to stay within the count. When at the limit:"
            )
        );
        assert_eq!(
            lead(LimitKind::Hard, 1, LimitSubject::Block(BlockKind::Code)).as_deref(),
            Some(
                "At most 1 fence. Never leave out or merge content to stay within the count. When at the limit:"
            )
        );
        assert_eq!(
            lead(LimitKind::Soft, 2, LimitSubject::Block(BlockKind::Group)).as_deref(),
            Some(
                "Usually up to 2 repetitions. Never leave out or merge content to stay within the count. When at the limit:"
            )
        );
        let html = LimitSubject::Rule {
            block: BlockKind::Html,
            counts: Counts::Units,
            name: None,
        };
        assert_eq!(
            lead(LimitKind::Hard, 0, html).as_deref(),
            Some("No units. Instead:")
        );
        // A template without `{unit}` needs no unit of the language.
        let mut own = Texts::default();
        own.add(
            parse("de: { limit: { ban: 'Nichts davon. Stattdessen:', hard: 'Höchstens {count}.' } }\n")
                .unwrap(),
        );
        let de = |count| own.lead(&LimitFacts::new(LimitKind::Hard, count, table, Some("de")));
        assert_eq!(de(0).as_deref(), Some("Nichts davon. Stattdessen:"));
        assert_eq!(de(4).as_deref(), Some("Höchstens 4."));
        // A `rule-unit` without `{unit}` needs no unit either.
        own.add(
            parse("de: { limit: { soft: 'Meist {count} {unit}.' }, rule-unit: '`{name}`' }\n")
                .unwrap(),
        );
        let tasks = LimitSubject::Rule {
            block: BlockKind::BulletList,
            counts: Counts::Units,
            name: Some(RuleName::RuleId("tasks")),
        };
        assert_eq!(
            own.lead(&LimitFacts::new(LimitKind::Soft, 9, tasks, Some("de")))
                .as_deref(),
            Some("Meist 9 `tasks`.")
        );
        assert_eq!(
            own.lead(&LimitFacts::new(LimitKind::Soft, 9, table, Some("de"))),
            None
        );
        assert_eq!(
            t.lead(&LimitFacts::new(LimitKind::Hard, 3, table, None)),
            None
        );
    }
}
