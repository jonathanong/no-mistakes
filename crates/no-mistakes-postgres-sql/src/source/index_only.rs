use sqlparser::tokenizer::{Location, Token, TokenWithSpan, Whitespace};

/// Remove PostgreSQL's CREATE INDEX relation modifier from sqlparser's clone,
/// keeping source locations and a marker for the public typed fact.
pub(super) fn prepare(tokens: &mut [TokenWithSpan]) -> Vec<Location> {
    let mut only_starts = Vec::new();
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_) | Token::EOF)).then_some(index)
        })
        .collect::<Vec<_>>();
    for (position, &create) in significant.iter().enumerate() {
        if !word(&tokens[create].token, "CREATE") {
            continue;
        }
        let mut cursor_position = position + 1;
        let Some(mut cursor) = significant.get(cursor_position).copied() else {
            continue;
        };
        if word(&tokens[cursor].token, "UNIQUE") {
            cursor_position += 1;
            let Some(next) = significant.get(cursor_position).copied() else {
                continue;
            };
            cursor = next;
            cursor_position += 1;
        } else {
            cursor_position += 1;
        }
        if !word(&tokens[cursor].token, "INDEX") {
            continue;
        }
        let Some(on) = significant[cursor_position..]
            .iter()
            .position(|index| {
                matches!(tokens[*index].token, Token::SemiColon)
                    || word(&tokens[*index].token, "ON")
            })
            .map(|offset| cursor_position + offset)
        else {
            continue;
        };
        if matches!(tokens[significant[on]].token, Token::SemiColon) {
            continue;
        }
        let Some(&only) = significant.get(on + 1) else {
            continue;
        };
        if !word(&tokens[only].token, "ONLY") {
            continue;
        }
        only_starts.push(tokens[create].span.start);
        tokens[only].token = Token::Whitespace(Whitespace::Space);
    }
    only_starts.sort_unstable();
    only_starts.dedup();
    only_starts
}

pub(super) fn contains(starts: &[Location], statement_start: Location) -> bool {
    starts.binary_search(&statement_start).is_ok()
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
