use super::Lexeme;
use sqlparser::tokenizer::{Span, Token};
use std::sync::LazyLock;

static NO_KEY: LazyLock<[Token; 2]> =
    LazyLock::new(|| [Token::make_keyword("NO"), Token::make_keyword("KEY")]);
static KEY: LazyLock<[Token; 1]> = LazyLock::new(|| [Token::make_keyword("KEY")]);
static COMMA: [Token; 1] = [Token::Comma];

pub(super) fn restore<'a>(
    mut tokens: Vec<Lexeme<'a>>,
    source_at: &dyn Fn(Span) -> Option<&'a str>,
) -> Vec<Lexeme<'a>> {
    let reordered = tokens.windows(2).any(|pair| pair[0].at() > pair[1].at());
    if reordered {
        // The parser copies the prefix and wait policy for each target in an OF list.
        tokens.sort_by_key(Lexeme::at);
        tokens.dedup_by(|left, right| left.span == right.span && left.token == right.token);
    } else if !tokens.iter().any(|token| token.word("for")) {
        return tokens;
    }

    let mut restored: Vec<Lexeme<'a>> = Vec::with_capacity(tokens.len());
    for token in tokens {
        if let Some(previous) = restored.last() {
            let span = Span::new(previous.span.end, token.span.start);
            let words = missing(previous, &token, span, reordered, source_at);
            restored.extend(words.iter().map(|token| Lexeme {
                token,
                span,
                depth: 0,
            }));
        }
        restored.push(token);
    }
    restored
}

fn missing<'sql>(
    left: &Lexeme<'_>,
    right: &Lexeme<'_>,
    gap: Span,
    reordered: bool,
    source_at: &dyn Fn(Span) -> Option<&'sql str>,
) -> &'static [Token] {
    if left.word("for") && source_word(left, "for", source_at) {
        if right.word("update")
            && source_word(right, "update", source_at)
            && source_at(gap).is_some_and(|text| matches_gap(text, &["no", "key"]))
        {
            return &*NO_KEY;
        }
        if right.word("share")
            && source_word(right, "share", source_at)
            && source_at(gap).is_some_and(|text| matches_gap(text, &["key"]))
        {
            return &*KEY;
        }
    }
    if reordered && source_at(gap).is_some_and(|text| matches_gap(text, &[","])) {
        return &COMMA;
    }
    &[]
}

fn source_word<'sql>(
    token: &Lexeme<'_>,
    expected: &str,
    source_at: &dyn Fn(Span) -> Option<&'sql str>,
) -> bool {
    source_at(token.span).is_some_and(|text| text.eq_ignore_ascii_case(expected))
}

fn matches_gap(mut text: &str, words: &[&str]) -> bool {
    for (index, word) in words.iter().enumerate() {
        let Some(rest) = skip_trivia(text) else {
            return false;
        };
        if (index > 0 && rest.len() == text.len())
            || !rest
                .get(..word.len())
                .is_some_and(|part| part.eq_ignore_ascii_case(word))
        {
            return false;
        }
        text = &rest[word.len()..];
    }
    skip_trivia(text) == Some("")
}

fn skip_trivia(mut text: &str) -> Option<&str> {
    loop {
        text = text.trim_ascii_start();
        if let Some(comment) = text.strip_prefix("--") {
            text = comment.find(['\n', '\r']).map_or("", |end| &comment[end..]);
        } else if text.starts_with("/*") {
            text = text.get(block_comment_end(text.as_bytes())?..)?;
        } else {
            return Some(text);
        }
    }
}

fn block_comment_end(bytes: &[u8]) -> Option<usize> {
    let mut depth = 1;
    let mut at = 2;
    while let Some(pair) = bytes.get(at..at + 2) {
        match pair {
            b"/*" => {
                depth += 1;
                at += 2;
            }
            b"*/" => {
                depth -= 1;
                at += 2;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => at += 1,
        }
    }
    None
}
