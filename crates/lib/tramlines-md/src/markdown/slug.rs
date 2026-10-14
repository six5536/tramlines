//! Heading slugs, formed as repository hosts form theirs (TLMD_101).

/// Lowercase; keep letters, digits, spaces, hyphens and underscores; drop
/// every other character; spaces to hyphens.
// sokf-implements: TLMD_101
pub fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::markdown::is_slug;

    #[test]
    fn forms_slugs_like_a_repository_host() {
        assert_eq!(slug("Acceptance Criteria"), "acceptance-criteria");
        assert_eq!(slug("CLI-1_AC-2"), "cli-1_ac-2");
        assert_eq!(slug("What's `this`? (v2)"), "whats-this-v2");
        assert_eq!(slug("  two  spaces "), "--two--spaces-");
        assert_eq!(slug("Ünïcode Ok"), "ünïcode-ok");
        assert_eq!(slug(""), "");
    }

    #[test]
    fn recognizes_spec_5_slugs() {
        for ok in ["a", "issue-004-retry", "x1-2y"] {
            assert!(is_slug(ok), "{ok}");
        }
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "a b", "é"] {
            assert!(!is_slug(bad), "{bad}");
        }
    }

    // sokf-tests: TLMD_101
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn slug_is_idempotent_and_well_formed(heading in "\\PC{0,40}") {
            let once = slug(&heading);
            prop_assert_eq!(slug(&once), once.clone());
            for c in once.chars() {
                prop_assert!(c.is_alphanumeric() || c == '-' || c == '_');
            }
            prop_assert_eq!(once.to_lowercase(), once.clone());
        }
    }
}
