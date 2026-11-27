use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn keyword(source: &str) -> &str {
    source
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ';' | '|' | '&'))
        .next()
        .unwrap_or_default()
}

pub(super) fn unsupported_control(source: &str, in_double_bracket: bool) -> bool {
    let source =
        source.trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '|' | '&' | '!'));
    let word = keyword(source);
    (!in_double_bracket && source.starts_with(['(', '{']))
        || matches!(
            word,
            "if" | "then"
                | "elif"
                | "else"
                | "fi"
                | "case"
                | "esac"
                | "while"
                | "until"
                | "for"
                | "do"
                | "done"
                | "select"
                | "function"
                | "time"
                | "coproc"
        )
        || word.ends_with("()")
}

pub(super) fn drop_leading_bangs(mut words: &[Word]) -> &[Word] {
    while words.first().is_some_and(|word| word.text == "!") {
        words = &words[1..];
    }
    words
}

pub(super) fn drop_leading_word<'a>(words: &'a [Word], kw: &str) -> &'a [Word] {
    &words[usize::from(words.first().is_some_and(|word| word.text == kw))..]
}

pub(super) fn update_double_bracket(words: &[Word], in_double_bracket: &mut bool) {
    for word in words {
        if word.text == "[[" {
            *in_double_bracket = true;
        } else if word.text == "]]" {
            *in_double_bracket = false;
        }
    }
}
