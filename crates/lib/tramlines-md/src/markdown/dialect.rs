//! The marker names a host writes: the namespace of the engine's own
//! markers, the host's region and trailer, and its reference scheme.

use std::collections::BTreeMap;

// sokf:begin dialect
/// How a host spells the markers the parser recognises. The engine's own
/// markers are `<ns>:only`, `<ns>:guidance` and `<ns>:optional`; a host
/// may add one region, one trailer and one reference scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialect {
    /// The namespace every marker is written in, as `host`.
    pub namespace: &'static str,
    /// The host's region: `<!-- <ns>:<region> target -->` to
    /// `<!-- /<ns>:<region> -->`, masked as one opaque block, as `include`.
    pub region: Option<&'static str>,
    /// The host's trailer: `<!-- <ns>:<trailer> -->`, the generated block
    /// of reference definitions that ends a document, as `links`.
    pub trailer: Option<&'static str>,
    /// The host's reference scheme: a link label `<scheme>:<rest>` is a
    /// reference the host resolves, as `host`.
    pub scheme: Option<&'static str>,
}
// sokf:end dialect

// sokf-implements: TLMD_17
impl Default for Dialect {
    /// The engine's markers alone, in the namespace `tramlines`.
    fn default() -> Self {
        Dialect {
            namespace: "tramlines",
            region: None,
            trailer: None,
            scheme: None,
        }
    }
}

/// The marker on a line, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    /// `<!-- <ns>:<region> target -->`.
    Open(String),
    /// `<!-- /<ns>:<region> -->`.
    Close,
    /// `<!-- <ns>:<trailer> -->`.
    Trailer,
}

/// The text inside an HTML comment line, if the line is one.
pub(super) fn comment(line: &str) -> Option<&str> {
    line.trim()
        .strip_prefix("<!--")
        .and_then(|s| s.strip_suffix("-->"))
        .map(str::trim)
}

/// `text` starts with `<ns>:<name>`.
fn names(text: &str, ns: &str, name: &str) -> bool {
    text.strip_prefix(ns)
        .and_then(|t| t.strip_prefix(':'))
        .is_some_and(|t| t.starts_with(name))
}

impl Dialect {
    /// `<ns>:<name>`, as a marker writes it.
    pub fn spell(&self, name: &str) -> String {
        format!("{}:{name}", self.namespace)
    }

    /// Recognise a region or trailer marker line. Whitespace inside the
    /// comment is free; the line carries nothing else.
    pub fn marker(&self, line: &str) -> Option<Marker> {
        let inner = comment(line)?;
        if let Some(region) = self.region {
            if inner == format!("/{}", self.spell(region)) {
                return Some(Marker::Close);
            }
            if let Some(target) = inner.strip_prefix(&self.spell(region)) {
                let target = target.trim();
                if target.is_empty() || target.contains(char::is_whitespace) {
                    return None;
                }
                return Some(Marker::Open(target.to_string()));
            }
        }
        if self.trailer.is_some_and(|t| inner == self.spell(t)) {
            return Some(Marker::Trailer);
        }
        None
    }

    /// Whether a line is an `only` marker of grammar `example`, as an HTML
    /// comment or a YAML comment.
    pub fn is_only_marker(&self, line: &str) -> bool {
        let only = format!("{} ", self.spell("only"));
        if let Some(inner) = comment(line) {
            return inner.starts_with(&only);
        }
        let t = line.trim();
        match t.rfind('#') {
            Some(i) => t[i + 1..].trim().starts_with(&only),
            None => false,
        }
    }

    /// Whether a line is a `guidance` marker: one HTML comment line, or one
    /// YAML comment line, whose text starts `<ns>:guidance`. A scaffold
    /// writes one before the part it guides; a governed document that keeps
    /// one still has that part to write.
    // sokf-implements: TLMD_158
    pub fn is_guidance_marker(&self, line: &str) -> bool {
        if let Some(inner) = comment(line) {
            return names(inner, self.namespace, "guidance");
        }
        line.trim()
            .strip_prefix('#')
            .is_some_and(|rest| names(rest.trim_start(), self.namespace, "guidance"))
    }

    /// Whether a line opens an `optional` region: what lies between it and
    /// its close is a part the schema does not demand, so a reader of the
    /// scaffold keeps the part and drops the markers, or drops both. The
    /// region is written open and closed, because its reader is a person
    /// and nothing else tells them where it ends.
    pub fn is_optional_marker(&self, line: &str) -> bool {
        comment(line) == Some(self.spell("optional").as_str())
    }

    /// Whether a line closes an `optional` region.
    pub fn is_optional_close(&self, line: &str) -> bool {
        comment(line).and_then(|c| c.strip_prefix('/')) == Some(self.spell("optional").as_str())
    }

    /// Whether a line is either end of an `optional` region.
    pub fn is_optional_mark(&self, line: &str) -> bool {
        self.is_optional_marker(line) || self.is_optional_close(line)
    }

    /// Parse an `only k=v,v2 k2=v` marker line into its map, dimension to
    /// states. `None` when the line is no marker or is malformed.
    pub fn only_marker(&self, line: &str) -> Option<BTreeMap<String, Vec<String>>> {
        if !self.is_only_marker(line) {
            return None;
        }
        let only = format!("{} ", self.spell("only"));
        let i = line.find(&only)?;
        let rest = line[i + only.len()..].trim_end_matches("-->").trim();
        let mut map = BTreeMap::new();
        for pair in rest.split_whitespace() {
            let (k, v) = pair.split_once('=')?;
            if k.is_empty() || v.is_empty() {
                return None;
            }
            map.insert(k.to_string(), v.split(',').map(str::to_string).collect());
        }
        Some(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: Dialect = Dialect {
        namespace: "host",
        region: Some("include"),
        trailer: Some("links"),
        scheme: Some("host"),
    };

    // sokf-tests: MD-11_AC-2
    #[test]
    fn markers() {
        let m = |l: &str| HOST.marker(l);
        assert_eq!(m("<!-- host:include x -->"), Some(Marker::Open("x".into())));
        assert_eq!(
            m("  <!--host:include /a/b.rs#r-->"),
            Some(Marker::Open("/a/b.rs#r".into()))
        );
        assert_eq!(m("<!-- /host:include -->"), Some(Marker::Close));
        assert_eq!(m("<!-- host:links -->"), Some(Marker::Trailer));
        assert_eq!(m("<!-- host:include -->"), None);
        assert_eq!(m("<!-- host:include a b -->"), None);
        assert_eq!(m("<!-- host:includes x -->"), None);
        assert_eq!(m("text <!-- host:links -->"), None);
        let plain = Dialect::default();
        assert_eq!(plain.marker("<!-- host:include x -->"), None);
        assert_eq!(plain.marker("<!-- tramlines:links -->"), None);
    }

    #[test]
    fn only_markers() {
        assert!(HOST.is_only_marker("<!-- host:only state=closed -->"));
        assert!(HOST.is_only_marker("resolution: none # host:only state=closed"));
        assert!(!HOST.is_only_marker("<!-- host:links -->"));
        assert!(!HOST.is_only_marker("plain # comment"));
        assert!(!HOST.is_only_marker("plain"));
        assert!(HOST.is_optional_marker("<!-- host:optional -->"));
        assert!(HOST.is_optional_close("<!-- /host:optional -->"));
        assert!(HOST.is_optional_mark("<!-- /host:optional -->"));
        assert!(!HOST.is_optional_marker("<!-- /host:optional -->"));
        assert!(!HOST.is_optional_close("<!-- host:optional -->"));
        assert!(!HOST.is_optional_marker("<!-- host:only state=closed -->"));
        assert!(!HOST.is_optional_marker("<!-- host:optionally -->"));
        assert!(!HOST.is_only_marker("<!-- host:optional -->"));
        assert!(!Dialect::default().is_only_marker("<!-- host:only state=closed -->"));
        assert!(Dialect::default().is_only_marker("<!-- tramlines:only state=closed -->"));
        assert_eq!(
            HOST.only_marker("<!-- host:only state=open,done kind=bug -->"),
            Some(BTreeMap::from([
                ("kind".to_string(), vec!["bug".to_string()]),
                (
                    "state".to_string(),
                    vec!["open".to_string(), "done".to_string()]
                ),
            ]))
        );
    }

    #[test]
    fn guidance_markers() {
        assert!(HOST.is_guidance_marker("<!-- host:guidance Say what the part is for. -->"));
        assert!(HOST.is_guidance_marker("  # host:guidance The states, in order."));
        assert!(!HOST.is_guidance_marker("<!-- host:guidance spans"));
        assert!(!HOST.is_guidance_marker("title: x # host:guidance trailing"));
        assert!(!HOST.is_guidance_marker("<!-- host:only state=done -->"));
    }
}
