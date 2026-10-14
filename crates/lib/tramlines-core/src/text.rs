//! Checks of a scalar value that every format shares, and the English
//! that a finding is written in: the article before a word and a count.

/// Whether `s` is a slug: lower-case letters and digits joined by single
/// hyphens, `^[a-z0-9]+(-[a-z0-9]+)*$`.
pub fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Whether `s` has a URL scheme.
pub fn is_url(s: &str) -> bool {
    let Some(i) = s.find(':') else {
        return false;
    };
    let scheme = &s[..i];
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        && (s[i + 1..].starts_with("//") || scheme.eq_ignore_ascii_case("mailto"))
}

/// Each of `items` in code font, joined by `sep`: "`a`, `b`".
pub fn code_list<T: std::fmt::Display>(items: impl IntoIterator<Item = T>, sep: &str) -> String {
    items
        .into_iter()
        .map(|i| format!("`{i}`"))
        .collect::<Vec<_>>()
        .join(sep)
}

/// The indefinite article before `word`, as English says the word: `an`
/// before a vowel sound, judged from its spelling, which reads a `u` that
/// sounds as "you" and an `h` that is silent.
pub fn article(word: &str) -> &'static str {
    let w = word
        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_ascii_lowercase();
    const YOU: [&str; 13] = [
        "uni", "uin", "uid", "uli", "use", "usu", "uti", "utf", "uri", "url", "uk", "uu", "eu",
    ];
    const SILENT: [&str; 5] = ["hour", "honest", "honor", "heir", "html"];
    let vowel = w.starts_with(['a', 'e', 'i', 'o', 'u'])
        && !YOU.iter().any(|p| w.starts_with(p))
        && !w.starts_with("one");
    // A number reads as its first word: eight, eleven, eighteen.
    let eight = w.starts_with('8') || w.starts_with("11") || w.starts_with("18");
    if vowel || eight || SILENT.iter().any(|p| w.starts_with(p)) {
        "an"
    } else {
        "a"
    }
}

/// `n` with `one` or `many`, as its number takes: `1 item`, `2 items`.
pub fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// `is` for one, `are` for any other number.
pub fn is_are(n: usize) -> &'static str {
    if n == 1 { "is" } else { "are" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_takes_its_number() {
        assert_eq!(count(1, "item", "items"), "1 item");
        assert_eq!(count(0, "item", "items"), "0 items");
        assert_eq!((is_are(1), is_are(2)), ("is", "are"));
    }

    #[test]
    fn the_article_follows_the_sound() {
        for (word, said) in [
            ("image", "an"),
            ("int", "an"),
            ("`amount`", "an"),
            ("item", "an"),
            ("html", "an"),
            ("integer", "an"),
            ("unit", "a"),
            ("url", "a"),
            ("slug", "a"),
            ("paragraph", "a"),
            ("row", "a"),
            ("bullet-list", "a"),
            ("uint", "a"),
            ("ulid", "a"),
            ("utf8", "a"),
            ("8bit", "an"),
            ("one-of", "a"),
        ] {
            assert_eq!(article(word), said, "{word}");
        }
    }

    #[test]
    fn a_slug_is_lower_case_words_joined_by_hyphens() {
        assert!(is_slug("issue-1"));
        assert!(!is_slug("Issue"));
        assert!(!is_slug("a--b"));
        assert!(!is_slug(""));
    }

    #[test]
    fn a_url_has_a_scheme() {
        assert!(is_url("https://a"));
        assert!(is_url("mailto:a@b"));
        assert!(!is_url("a:b"));
        assert!(!is_url("1http://x"));
    }
}
