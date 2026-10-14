//! Builds a positioned value tree from saphyr's events, in the dialect of
//! TL_13 and TL_14: YAML 1.2 core schema, no anchor, alias or tag.

use saphyr::Scalar;
use saphyr_parser::{Event, Parser, ScalarStyle, Span};

use super::{Spanned, Yaml, YamlError};

/// Parse one YAML document. `first_line` is the file line of the text's first
/// line, so every node carries a file line. Empty text is `Null`.
// sokf-implements: TL_13
// sokf-implements: TL_14
// sokf-implements: TL_52
// sokf:begin yaml-parse
pub fn parse(text: &str, first_line: usize) -> Result<Spanned, YamlError> {
    // sokf:end yaml-parse
    let parser = Parser::new_from_str(text);
    let offset = first_line.saturating_sub(1);
    let line_of = |span: &Span| span.start.line() + offset;
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Spanned> = None;

    for item in parser {
        let (event, span) = match item {
            Ok(pair) => pair,
            Err(e) => {
                let line = e.marker().line() + offset;
                // The scanner refuses an alias whose anchor it has not seen,
                // which is every alias here: an anchor is refused first.
                if e.info().contains("unknown anchor") {
                    return Err(YamlError::Alias { line });
                }
                return Err(YamlError::Syntax {
                    line,
                    message: e.info().to_string(),
                });
            }
        };
        let line = line_of(&span);
        let node = match event {
            Event::Nothing | Event::StreamStart | Event::StreamEnd | Event::DocumentEnd => continue,
            Event::DocumentStart(_) => {
                if root.is_some() {
                    return Err(YamlError::Syntax {
                        line,
                        message: "more than one document".into(),
                    });
                }
                continue;
            }
            Event::Alias(_) => return Err(YamlError::Alias { line }),
            Event::Scalar(value, style, anchor, tag) => {
                plain(anchor, tag.is_some(), line)?;
                Spanned::new(scalar(value, style), line)
            }
            Event::SequenceStart(anchor, tag) => {
                plain(anchor, tag.is_some(), line)?;
                stack.push(Frame::Seq(Vec::new(), line));
                continue;
            }
            Event::MappingStart(anchor, tag) => {
                plain(anchor, tag.is_some(), line)?;
                stack.push(Frame::Map(Vec::new(), None, line));
                continue;
            }
            Event::SequenceEnd | Event::MappingEnd => match stack.pop() {
                Some(Frame::Seq(items, start)) => Spanned::new(Yaml::Seq(items), start),
                Some(Frame::Map(entries, _, start)) => Spanned::new(Yaml::Map(entries), start),
                None => {
                    return Err(YamlError::Syntax {
                        line,
                        message: "unbalanced collection".into(),
                    });
                }
            },
        };
        attach(&mut stack, &mut root, node)?;
    }
    Ok(root.unwrap_or_else(|| Spanned::new(Yaml::Null, first_line)))
}

enum Frame {
    Seq(Vec<Spanned>, usize),
    Map(
        Vec<(Spanned<String>, Spanned)>,
        Option<Spanned<String>>,
        usize,
    ),
}

fn attach(stack: &mut [Frame], root: &mut Option<Spanned>, node: Spanned) -> Result<(), YamlError> {
    match stack.last_mut() {
        None => {
            *root = Some(node);
            Ok(())
        }
        Some(Frame::Seq(items, _)) => {
            items.push(node);
            Ok(())
        }
        Some(Frame::Map(entries, pending, _)) => match pending.take() {
            None => {
                let key = match node.value {
                    Yaml::Str(s) => s,
                    _ => return Err(YamlError::NonStringKey { line: node.line }),
                };
                if entries.iter().any(|(k, _)| k.value == key) {
                    return Err(YamlError::Syntax {
                        line: node.line,
                        message: format!("duplicate key `{key}`"),
                    });
                }
                *pending = Some(Spanned::new(key, node.line));
                Ok(())
            }
            Some(key) => {
                entries.push((key, node));
                Ok(())
            }
        },
    }
}

/// Refuse a node that carries an anchor or a tag.
// sokf-implements: CORE-12_AC-1
fn plain(anchor: usize, tagged: bool, line: usize) -> Result<(), YamlError> {
    if anchor != 0 {
        return Err(YamlError::Anchor { line });
    }
    if tagged {
        return Err(YamlError::Tag { line });
    }
    Ok(())
}

/// Resolve a scalar per the core schema: quoted is a string; plain is
/// null, boolean, integer, float, else string.
fn scalar(value: std::borrow::Cow<'_, str>, style: ScalarStyle) -> Yaml {
    match Scalar::parse_from_cow_and_metadata(value, style, None) {
        Some(Scalar::Null) => Yaml::Null,
        Some(Scalar::Boolean(b)) => Yaml::Bool(b),
        Some(Scalar::Integer(i)) => Yaml::Int(i),
        Some(Scalar::FloatingPoint(f)) => Yaml::Float(f.into_inner()),
        Some(Scalar::String(s)) => Yaml::Str(s.into_owned()),
        None => Yaml::Null,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn parses_a_document_with_lines() {
        let doc = parse(
            "type: Note\nid: n\ntags:\n  - a\n  - b\nn: 3\nf: 1.5\nb: true\nz: ~\n",
            5,
        )
        .unwrap();
        assert_eq!(doc.line, 5);
        assert_eq!(doc.get("type").unwrap().as_str(), Some("Note"));
        assert_eq!(doc.get("id").unwrap().line, 6);
        let tags = doc.get("tags").unwrap();
        assert_eq!(tags.line, 8);
        assert_eq!(tags.as_seq().unwrap()[1].line, 9);
        assert_eq!(doc.get("n").unwrap().as_int(), Some(3));
        assert_eq!(doc.get("f").unwrap().value, Yaml::Float(1.5));
        assert_eq!(doc.get("b").unwrap().as_bool(), Some(true));
        assert_eq!(doc.get("z").unwrap().value, Yaml::Null);
        assert_eq!(doc.key_line("z"), Some(13));
        assert_eq!(
            doc.get("tags").unwrap().as_str_or_list().unwrap(),
            vec!["a", "b"]
        );
        assert_eq!(doc.get("id").unwrap().as_str_or_list().unwrap(), vec!["n"]);
    }

    #[test]
    fn quoted_scalars_stay_strings() {
        let doc = parse("a: \"0.6\"\nb: '3'\nc: 0.6\n", 1).unwrap();
        assert_eq!(doc.get("a").unwrap().as_str(), Some("0.6"));
        assert_eq!(doc.get("b").unwrap().as_str(), Some("3"));
        assert_eq!(doc.get("c").unwrap().value, Yaml::Float(0.6));
    }

    #[test]
    fn empty_text_is_null() {
        assert_eq!(parse("", 1).unwrap().value, Yaml::Null);
        assert_eq!(parse("# only a comment\n", 1).unwrap().value, Yaml::Null);
    }

    #[test]
    fn forbidden_constructs_are_errors_with_lines() {
        assert_eq!(
            parse("a: &x 1\nb: *x\n", 10).unwrap_err(),
            YamlError::Anchor { line: 10 }
        );
        assert_eq!(parse("b: *x\n", 10).unwrap_err().line(), 10);
        assert!(matches!(parse("b: *x\n", 10), Err(YamlError::Alias { .. })));
        assert_eq!(
            parse("a: 1\nb: !!str 2\n", 1).unwrap_err(),
            YamlError::Tag { line: 2 }
        );
        assert_eq!(
            parse("a: 1\n? [x]\n: 2\n", 1).unwrap_err(),
            YamlError::NonStringKey { line: 2 }
        );
        assert!(matches!(parse("a: [1\n", 1), Err(YamlError::Syntax { .. })));
        assert!(matches!(
            parse("a: 1\na: 2\n", 1),
            Err(YamlError::Syntax { line: 2, .. })
        ));
        assert!(matches!(
            parse("---\na: 1\n---\nb: 2\n", 1),
            Err(YamlError::Syntax { .. })
        ));
        let e = parse("a: [1\n", 3).unwrap_err();
        assert!(e.to_string().contains("line"), "{e}");
    }

    #[test]
    fn scalar_helpers() {
        let doc = parse("a: 1\n", 1).unwrap();
        assert_eq!(doc.type_name(), "map");
        assert_eq!(doc.get("a").unwrap().type_name(), "integer");
        assert_eq!(doc.get("a").unwrap().scalar_text(), Some("1".into()));
        assert_eq!(doc.scalar_text(), None);
        assert_eq!(parse("[1]", 1).unwrap().type_name(), "list");
        assert_eq!(parse("x", 1).unwrap().type_name(), "string");
        assert_eq!(parse("1.5", 1).unwrap().type_name(), "float");
        assert_eq!(parse("true", 1).unwrap().type_name(), "boolean");
        assert_eq!(parse("null", 1).unwrap().type_name(), "null");
        assert_eq!(parse("true", 1).unwrap().scalar_text(), Some("true".into()));
        assert_eq!(parse("1.5", 1).unwrap().scalar_text(), Some("1.5".into()));
        assert_eq!(parse("null", 1).unwrap().scalar_text(), Some("null".into()));
        assert!(parse("[1, x]", 1).unwrap().as_str_or_list().is_none());
        assert!(parse("1", 1).unwrap().as_str_or_list().is_none());
    }

    // sokf-tests: TL_13
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn anchors_aliases_and_tags_are_errors_on_their_line(
            before in 0usize..4,
            name in "[a-z][a-z0-9]{0,5}",
        ) {
            let pad: String = (0..before).map(|i| format!("k{i}: v\n")).collect();
            let line = before + 1;
            prop_assert_eq!(
                parse(&format!("{pad}a: &{name} 1\n"), 1).unwrap_err(),
                YamlError::Anchor { line }
            );
            prop_assert_eq!(
                parse(&format!("{pad}a: *{name}\n"), 1).unwrap_err(),
                YamlError::Alias { line }
            );
            prop_assert_eq!(
                parse(&format!("{pad}a: !{name} 1\n"), 1).unwrap_err(),
                YamlError::Tag { line }
            );
        }

        // sokf-tests: CORE-10_AC-1
        #[test]
        fn plain_scalars_follow_the_core_schema(n in any::<i64>(), b in any::<bool>(), s in "[a-z][a-z ]{0,10}[a-z]") {
            prop_assert_eq!(parse(&format!("v: {n}\n"), 1).unwrap().get("v").unwrap().as_int(), Some(n));
            prop_assert_eq!(parse(&format!("v: {b}\n"), 1).unwrap().get("v").unwrap().as_bool(), Some(b));
            let doc = parse(&format!("v: {s}\n"), 1).unwrap();
            prop_assert_eq!(doc.get("v").unwrap().as_str(), Some(s.as_str()));
            // Quoting makes any of them a string.
            let doc = parse(&format!("v: \"{n}\"\n"), 1).unwrap();
            let quoted = n.to_string();
            prop_assert_eq!(doc.get("v").unwrap().as_str(), Some(quoted.as_str()));
        }
    }
}
