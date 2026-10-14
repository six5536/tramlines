//! The positioned YAML value tree.

use serde::{Deserialize, Serialize};

/// A YAML 1.2 core-schema value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Yaml {
    /// `null`, `~` or an empty plain scalar.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// An integer.
    Int(i64),
    /// A floating-point number.
    Float(f64),
    /// A string.
    Str(String),
    /// A sequence.
    Seq(Vec<Spanned>),
    /// A mapping, in source order.
    Map(Vec<(Spanned<String>, Spanned)>),
}

/// A value with the line it starts on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spanned<T = Yaml> {
    /// The value.
    pub value: T,
    /// The 1-based line of the value in its file.
    pub line: usize,
}

impl<T> Spanned<T> {
    /// Pair a value with its line.
    pub fn new(value: T, line: usize) -> Self {
        Spanned { value, line }
    }
}

impl Spanned {
    /// The string value, if this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            Yaml::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The mapping entries, if this is a mapping.
    pub fn as_map(&self) -> Option<&[(Spanned<String>, Spanned)]> {
        match &self.value {
            Yaml::Map(m) => Some(m),
            _ => None,
        }
    }

    /// The sequence items, if this is a sequence.
    pub fn as_seq(&self) -> Option<&[Spanned]> {
        match &self.value {
            Yaml::Seq(s) => Some(s),
            _ => None,
        }
    }

    /// The integer value, if this is an integer.
    pub fn as_int(&self) -> Option<i64> {
        match &self.value {
            Yaml::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// The boolean value, if this is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match &self.value {
            Yaml::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The value of `key`, if this is a mapping carrying it.
    pub fn get(&self, key: &str) -> Option<&Spanned> {
        self.entry(key).map(|(_, v)| v)
    }

    /// The line of `key`, if this is a mapping carrying it.
    pub fn key_line(&self, key: &str) -> Option<usize> {
        self.entry(key).map(|(k, _)| k.line)
    }

    /// The entry of `key`, if this is a mapping carrying it.
    fn entry(&self, key: &str) -> Option<&(Spanned<String>, Spanned)> {
        self.as_map()?.iter().find(|(k, _)| k.value == key)
    }

    /// A string, or the items of a sequence of strings (`string-or-list`).
    pub fn as_str_or_list(&self) -> Option<Vec<String>> {
        match &self.value {
            Yaml::Str(s) => Some(vec![s.clone()]),
            Yaml::Seq(items) => items
                .iter()
                .map(|i| i.as_str().map(str::to_string))
                .collect(),
            _ => None,
        }
    }

    /// The name of the value's kind, for messages.
    pub fn type_name(&self) -> &'static str {
        match &self.value {
            Yaml::Null => "null",
            Yaml::Bool(_) => "boolean",
            Yaml::Int(_) => "integer",
            Yaml::Float(_) => "float",
            Yaml::Str(_) => "string",
            Yaml::Seq(_) => "list",
            Yaml::Map(_) => "map",
        }
    }

    /// The scalar as text, for messages and `const` comparisons.
    pub fn scalar_text(&self) -> Option<String> {
        match &self.value {
            Yaml::Null => Some("null".into()),
            Yaml::Bool(b) => Some(b.to_string()),
            Yaml::Int(i) => Some(i.to_string()),
            Yaml::Float(f) => Some(f.to_string()),
            Yaml::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
}

// sokf:begin yaml-error
/// A construct the YAML dialect refuses (TL_14), or a syntax
/// error, with its line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum YamlError {
    /// The text is not valid YAML.
    #[error("YAML does not parse at line {line}: {message}")]
    Syntax {
        /// The line of the failure.
        line: usize,
        /// The parser's message.
        message: String,
    },
    /// An anchor (`&name`).
    #[error("YAML anchor at line {line}: anchors are not read")]
    Anchor {
        /// The line of the anchor.
        line: usize,
    },
    /// An alias (`*name`).
    #[error("YAML alias at line {line}: aliases are not read")]
    Alias {
        /// The line of the alias.
        line: usize,
    },
    /// A tag (`!tag`).
    #[error("YAML tag at line {line}: tags are not read")]
    Tag {
        /// The line of the tag.
        line: usize,
    },
    /// A mapping key that is not a string.
    #[error("YAML mapping key at line {line} is not a string")]
    NonStringKey {
        /// The line of the key.
        line: usize,
    },
}
// sokf:end yaml-error

impl YamlError {
    /// Whether the YAML parses and uses what the engine refuses: an
    /// anchor, an alias or a tag.
    // sokf:begin yaml-error
    pub fn is_refused(&self) -> bool {
        // sokf:end yaml-error
        // sokf-implements: CORE-12_AC-1
        matches!(
            self,
            YamlError::Anchor { .. } | YamlError::Alias { .. } | YamlError::Tag { .. }
        )
    }

    /// The line the error is at.
    pub fn line(&self) -> usize {
        match self {
            YamlError::Syntax { line, .. }
            | YamlError::Anchor { line }
            | YamlError::Alias { line }
            | YamlError::Tag { line }
            | YamlError::NonStringKey { line } => *line,
        }
    }
}
