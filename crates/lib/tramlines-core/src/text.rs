//! Checks of a scalar value that every format shares, and the English
//! that a finding is written in: the article before a word, a count, and
//! the label before a limit's `when-at-limit`.

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

/// Whether `s` is a language tag in the syntax of BCP 47 (RFC 5646): a
/// language, then an optional script, region, variants, extensions, and
/// private use, or private use alone. It checks the form of each subtag,
/// not whether a registry lists it, and gives a grandfathered tag no
/// special treatment: an irregular one, such as `i-klingon`, fails, and a
/// regular one, such as `zh-min-nan`, passes.
// sokf-implements: TL_102
pub fn is_language_tag(s: &str) -> bool {
    let alpha = |t: &str| t.bytes().all(|b| b.is_ascii_alphabetic());
    let alnum = |t: &str| t.bytes().all(|b| b.is_ascii_alphanumeric());
    let private = |rest: &[&str]| {
        !rest.is_empty() && rest.iter().all(|t| (1..=8).contains(&t.len()) && alnum(t))
    };
    let tags: Vec<&str> = s.split('-').collect();
    let Some((first, mut rest)) = tags.split_first() else {
        return false;
    };
    if first.eq_ignore_ascii_case("x") {
        return private(rest);
    }
    if !(2..=8).contains(&first.len()) || !alpha(first) {
        return false;
    }
    let take = |rest: &mut &[&str], ok: &dyn Fn(&str) -> bool| match rest.first() {
        Some(t) if ok(t) => {
            *rest = &rest[1..];
            true
        }
        _ => false,
    };
    // Up to three extended language subtags follow a short language.
    if first.len() <= 3 {
        for _ in 0..3 {
            if !take(&mut rest, &|t| t.len() == 3 && alpha(t)) {
                break;
            }
        }
    }
    take(&mut rest, &|t| t.len() == 4 && alpha(t));
    take(&mut rest, &|t| {
        (t.len() == 2 && alpha(t)) || (t.len() == 3 && t.bytes().all(|b| b.is_ascii_digit()))
    });
    while take(&mut rest, &|t| {
        alnum(t)
            && ((5..=8).contains(&t.len()) || (t.len() == 4 && t.as_bytes()[0].is_ascii_digit()))
    }) {}
    while let Some(t) = rest.first() {
        if t.len() != 1 || !alnum(t) || t.eq_ignore_ascii_case("x") {
            break;
        }
        rest = &rest[1..];
        let n = rest
            .iter()
            .take_while(|t| (2..=8).contains(&t.len()) && alnum(t))
            .count();
        if n == 0 {
            return false;
        }
        rest = &rest[n..];
    }
    match rest.split_first() {
        None => true,
        Some((x, tail)) => x.eq_ignore_ascii_case("x") && private(tail),
    }
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
    const YOU: [&str; 13] = [
        "uni", "uin", "uid", "uli", "use", "usu", "uti", "utf", "uri", "url", "uk", "uu", "eu",
    ];
    const SILENT: [&str; 5] = ["hour", "honest", "honor", "heir", "html"];
    let w = word
        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_ascii_lowercase();
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

/// The label between the engine's words of a finding on a count above a
/// hard limit and the limit's `when-at-limit`.
pub const AT_LIMIT: &str = ". When at the limit: ";

/// The words of a finding on a count above a hard limit, then
/// [`AT_LIMIT`] and the limit's `when-at-limit`, its whitespace squashed so
/// that the finding stays one line: "…; it has 3. When at the limit: Put
/// more in the list that follows." The words end with no full stop,
/// because the label begins with one. A text that holds no word adds
/// nothing.
// sokf-implements: TL_77
pub fn at_limit(words: String, when_at_limit: &str) -> String {
    let text = crate::grammar::squash(when_at_limit);
    if text.is_empty() {
        words
    } else {
        format!("{words}{AT_LIMIT}{text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // sokf-tests: TL_102
    #[test]
    fn a_language_tag_has_the_syntax_of_bcp_47() {
        for tag in [
            "en",
            "de",
            "de-CH",
            "zh-Hant-TW",
            "sr-Latn-RS",
            "es-419",
            "de-CH-1996",
            "zh-yue-HK",
            "en-US-x-twain",
            "x-house",
            "en-a-bbb-x-a",
            "EN-gb",
        ] {
            assert!(is_language_tag(tag), "{tag}");
        }
        for tag in [
            "",
            "e",
            "en_US",
            "en-",
            "-en",
            "en--US",
            "123",
            "en-US-x",
            "toolongtag",
            "en-a",
            "en-US-a-x-b",
            "x",
            "en GB",
        ] {
            assert!(!is_language_tag(tag), "{tag}");
        }
    }

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

    // sokf-tests: TL_77
    #[test]
    fn a_finding_at_a_limit_ends_with_its_text_on_one_line() {
        assert_eq!(
            at_limit("it has 2".into(), "Put more\n  in a list below.\n"),
            "it has 2. When at the limit: Put more in a list below."
        );
        assert_eq!(at_limit("it has 2".into(), " \n"), "it has 2");
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
