//! Link destinations, classified into the forms a body link takes.

use super::{LinkTarget, is_url};
use crate::RepoPath;

/// Marks a URL as a reference label the document did not define.
pub(super) const REF: &str = "\u{1}ref:";

/// Classify a link destination. `source` is the link as written, used to
/// tell a full or collapsed reference from a shortcut one; `scheme` is the
/// host's reference scheme.
pub(super) fn target(url: &str, source: &str, scheme: Option<&str>) -> Option<LinkTarget> {
    if let Some(label) = url.strip_prefix(REF) {
        if let Some(rest) = scheme.and_then(|s| label.strip_prefix(s)?.strip_prefix(':')) {
            return Some(LinkTarget::Host(rest.to_string()));
        }
        // `[text][label]` and `[label][]` are references; `[label]` alone is
        // text under CommonMark when nothing defines it.
        let full = source.ends_with("][]")
            || source
                .strip_suffix(']')
                .is_some_and(|s| s.rfind('[').is_some_and(|i| s[..i].ends_with(']')));
        return full.then(|| LinkTarget::Reference(label.to_string()));
    }
    if let Some(fragment) = url.strip_prefix('#') {
        return Some(LinkTarget::Anchor(fragment.to_string()));
    }
    if url.starts_with('/') {
        let (path, fragment) = split_fragment(url);
        return Some(match RepoPath::parse(path) {
            Some(path) => LinkTarget::Repo { path, fragment },
            None => LinkTarget::Invalid(url.to_string()),
        });
    }
    if is_url(url) {
        return Some(LinkTarget::Url(url.to_string()));
    }
    let (path, fragment) = split_fragment(url);
    Some(LinkTarget::Relative {
        target: path.to_string(),
        fragment,
    })
}

/// The host reference an image unit's text carries, `![alt][<scheme>:<rest>]`:
/// the label after `<scheme>:`; `None` for any other destination.
// sokf-implements: TLMD_15
pub fn host_ref(unit: &str, scheme: Option<&str>) -> Option<String> {
    match target(unit, "[x][y]", scheme)? {
        LinkTarget::Host(rest) => Some(rest),
        _ => None,
    }
}

fn split_fragment(s: &str) -> (&str, Option<String>) {
    match s.split_once('#') {
        Some((p, f)) => (p, Some(f.to_string())),
        None => (s, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::{self, BlockKind, Body, Dialect, segments};

    const HOST: Dialect = Dialect {
        namespace: "host",
        region: Some("include"),
        trailer: Some("links"),
        scheme: Some("host"),
    };

    fn parse_body(body: &str) -> Body {
        let segments = segments(body, 1, &HOST);
        markdown::parse(body, 1, &segments, &HOST)
    }

    #[test]
    fn links_are_classified() {
        let body = "See [config][host:config] and [crit][host:req#CLI-1_AC-2] and [bare][host:#K] and [self][host:config].\n\nA [typo][nope] and [collapsed][] and a [shortcut] word.\n\n[Path](/k/a.md#x) [rel](../b.md) [url](https://x.y/z) [mail](mailto:a@b) [anchor](#top) [bad](/../x)\n\n![img](/img.png)\n\n[defined][d]\n\n[d]: /d.md\n";
        let b = parse_body(body);
        let targets: Vec<&LinkTarget> = b.links.iter().map(|l| &l.target).collect();
        assert_eq!(
            targets,
            vec![
                &LinkTarget::Host("config".into()),
                &LinkTarget::Host("req#CLI-1_AC-2".into()),
                &LinkTarget::Host("#K".into()),
                &LinkTarget::Host("config".into()),
                &LinkTarget::Reference("nope".into()),
                &LinkTarget::Reference("collapsed".into()),
                &LinkTarget::Repo {
                    path: RepoPath::parse("/k/a.md").unwrap(),
                    fragment: Some("x".into())
                },
                &LinkTarget::Relative {
                    target: "../b.md".into(),
                    fragment: None
                },
                &LinkTarget::Url("https://x.y/z".into()),
                &LinkTarget::Url("mailto:a@b".into()),
                &LinkTarget::Anchor("top".into()),
                &LinkTarget::Invalid("/../x".into()),
                &LinkTarget::Repo {
                    path: RepoPath::parse("/img.png").unwrap(),
                    fragment: None
                },
                &LinkTarget::Repo {
                    path: RepoPath::parse("/d.md").unwrap(),
                    fragment: None
                },
            ]
        );
        assert_eq!(b.ref_defs, vec![("d".to_string(), "/d.md".to_string(), 11)]);
        let refs = b
            .blocks
            .iter()
            .find(|x| x.kind == BlockKind::References)
            .unwrap();
        assert_eq!(refs.line, 11);
        assert!(is_url("https://a"));
        assert!(!is_url("a:b"));
        assert!(!is_url("no"));
        assert!(!is_url("1http://x"));
    }
}
