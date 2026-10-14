//! JSON Merge Patch over YAML values, which a format entry declaring
//! `patch` is merged onto the entry it shadows with (TL_38, TL_39).

use crate::{Spanned, Yaml};

/// JSON Merge Patch, RFC 7386, over two YAML values: two maps merge key by
/// key, a null in the patch deletes the key, and anything else replaces.
/// The `patch` marker itself does not survive into the result.
// sokf-implements: TL_38
// sokf-implements: TL_39
pub(super) fn merge_patch(base: &Spanned, patch: &Spanned) -> Spanned {
    let (Yaml::Map(base_entries), Yaml::Map(patch_entries)) = (&base.value, &patch.value) else {
        return patch.clone();
    };
    let mut out = base_entries.clone();
    for (key, value) in patch_entries {
        if key.value == "patch" {
            continue;
        }
        let at = out.iter().position(|(k, _)| k.value == key.value);
        match (&value.value, at) {
            (Yaml::Null, Some(i)) => {
                out.remove(i);
            }
            (Yaml::Null, None) => {}
            (_, Some(i)) => out[i].1 = merge_patch(&out[i].1, value),
            (_, None) => out.push((key.clone(), value.clone())),
        }
    }
    Spanned::new(Yaml::Map(out), patch.line)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::yaml;

    /// Two YAML values are the same tree, whatever lines they sit on.
    fn same(a: &Spanned, b: &Spanned) -> bool {
        match (&a.value, &b.value) {
            (Yaml::Map(x), Yaml::Map(y)) => {
                x.len() == y.len()
                    && x.iter()
                        .zip(y)
                        .all(|((k, v), (k2, v2))| k.value == k2.value && same(v, v2))
            }
            (Yaml::Seq(x), Yaml::Seq(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(v, v2)| same(v, v2))
            }
            (x, y) => x == y,
        }
    }

    /// A value a generated entry may carry, `~` standing for a deletion.
    const VALUES: [&str; 5] = ["1", "text", "[1, 2]", "{ p: 1, q: 2 }", "~"];

    /// A map over `a`, `b` and `c`, each key absent or one of the values.
    fn arb_map(deletions: bool) -> impl Strategy<Value = Spanned> {
        let top = if deletions {
            VALUES.len()
        } else {
            VALUES.len() - 1
        };
        prop::collection::vec(prop::option::of(0..top), 3).prop_map(|choices| {
            let mut text = String::new();
            for (key, choice) in ["a", "b", "c"].iter().zip(&choices) {
                if let Some(i) = choice {
                    text.push_str(&format!("{key}: {}\n", VALUES[*i]));
                }
            }
            if text.is_empty() {
                text.push_str("{}\n");
            }
            yaml::parse(&text, 1).unwrap()
        })
    }

    // sokf-tests: CORE_P-2
    // sokf-tests: TL_50
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A merge is RFC 7386: a stated key replaces, an omitted key is
        /// inherited, a null deletes, and two maps merge one level down.
        #[test]
        fn a_patch_merges_by_rfc_7386(base in arb_map(false), patch in arb_map(true)) {
            let merged = merge_patch(&base, &patch);
            for key in ["a", "b", "c"] {
                let from_base = base.get(key);
                let from_patch = patch.get(key);
                let got = merged.get(key);
                match (from_base, from_patch) {
                    (_, Some(p)) if p.value == Yaml::Null => prop_assert!(got.is_none()),
                    (Some(b), Some(p)) => match (&b.value, &p.value) {
                        (Yaml::Map(_), Yaml::Map(_)) => {
                            prop_assert!(same(got.unwrap(), &merge_patch(b, p)));
                        }
                        _ => prop_assert!(same(got.unwrap(), p)),
                    },
                    (None, Some(p)) => prop_assert!(same(got.unwrap(), p)),
                    (Some(b), None) => prop_assert!(same(got.unwrap(), b)),
                    (None, None) => prop_assert!(got.is_none()),
                }
            }
        }
    }
}
