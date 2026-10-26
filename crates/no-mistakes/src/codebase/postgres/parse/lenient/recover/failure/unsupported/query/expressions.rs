use super::{expression, matching, word};
use sqlparser::tokenizer::Token;
#[path = "expressions/json_object.rs"]
mod json_object;
#[path = "expressions/quantified.rs"]
mod quantified;

pub(super) fn rewrite(tokens: &mut Vec<Token>) -> bool {
    json(tokens)
        || xml(tokens)
        || collation(tokens)
        || window_range(tokens)
        || at_local(tokens)
        || quantified::values(tokens)
}

fn json(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(1) {
        if tokens[at + 1] != Token::LParen {
            continue;
        }
        let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
            continue;
        };
        if word(&tokens[at], "JSON_QUERY") {
            let default = (at + 2..end).find(|pos| word(&tokens[*pos], "DEFAULT"));
            let error = (at + 2..end.saturating_sub(1))
                .find(|pos| word(&tokens[*pos], "ON") && word(&tokens[*pos + 1], "ERROR"));
            if let (Some(default), Some(error)) = (default, error) {
                if default + 1 < error
                    && (at + 2..default)
                        .find(|pos| word(&tokens[*pos], "RETURNING"))
                        .is_some_and(|returning| {
                            super::super::super::ddl_foreign::valid_type(
                                &tokens[returning + 1..default],
                            )
                        })
                    && expression(&tokens[default + 1..error])
                {
                    tokens.drain(default..error + 2);
                    return true;
                }
            }
        }
        if word(&tokens[at], "JSON_EXISTS") {
            let passing = (at + 2..end).find(|pos| word(&tokens[*pos], "PASSING"));
            if let Some(passing) = passing {
                if passing_bindings(&tokens[passing + 1..end]) {
                    tokens.drain(passing..end);
                    return true;
                }
            }
        }
        if word(&tokens[at], "JSON_OBJECT") && json_object::rewrite(tokens, at, end) {
            return true;
        }
    }
    false
}

fn passing_bindings(tokens: &[Token]) -> bool {
    let mut start = 0;
    loop {
        let end = tokens[start..]
            .iter()
            .position(|token| *token == Token::Comma)
            .map_or(tokens.len(), |offset| start + offset);
        if end < start + 3
            || !word(&tokens[end - 2], "AS")
            || !matches!(tokens[end - 1], Token::Word(_))
            || !expression(&tokens[start..end - 2])
        {
            return false;
        }
        if end == tokens.len() {
            return true;
        }
        start = end + 1;
    }
}

fn xml(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len() {
        if word(&tokens[at], "XMLPI") && tokens.get(at + 1) == Some(&Token::LParen) {
            let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
                continue;
            };
            if end == at + 4
                && word(&tokens[at + 2], "NAME")
                && matches!(tokens[at + 3], Token::Word(_))
            {
                tokens.splice(
                    at + 1..=end,
                    [
                        Token::LParen,
                        Token::SingleQuotedString("name".into()),
                        Token::RParen,
                    ],
                );
                return true;
            }
        }
        if word(&tokens[at], "XMLSERIALIZE") && tokens.get(at + 1) == Some(&Token::LParen) {
            let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
                continue;
            };
            if end > at + 5 && word(&tokens[at + 2], "DOCUMENT") {
                if let Some(as_at) = (at + 3..end).find(|pos| word(&tokens[*pos], "AS")) {
                    if expression(&tokens[at + 3..as_at])
                        && as_at + 2 == end
                        && super::super::super::ddl_foreign::valid_type(&tokens[as_at + 1..end])
                    {
                        tokens.splice(at..=end, [Token::make_keyword("NULL")]);
                        return true;
                    }
                }
            }
        }
        if word(&tokens[at], "IS")
            && tokens.get(at + 1).is_some_and(|t| word(t, "DOCUMENT"))
            && at > 0
            && expression(&tokens[1..at])
        {
            tokens[at + 1] = Token::make_keyword("NULL");
            return true;
        }
    }
    false
}

fn collation(tokens: &mut Vec<Token>) -> bool {
    if let Some(at) = tokens.windows(3).position(|pair| {
        word(&pair[0], "COLLATION") && word(&pair[1], "FOR") && pair[2] == Token::LParen
    }) {
        let Some(end) = matching(tokens, at + 2, Token::LParen, Token::RParen) else {
            return false;
        };
        if end > at + 3 && expression(&tokens[at + 3..end]) {
            tokens.remove(at + 1);
            return true;
        }
    }
    false
}

fn window_range(tokens: &mut Vec<Token>) -> bool {
    let has_range = tokens
        .windows(2)
        .any(|pair| word(&pair[0], "RANGE") && word(&pair[1], "BETWEEN"));
    if !has_range {
        return false;
    }
    for at in 1..tokens.len().saturating_sub(2) {
        if tokens[at] == Token::DoubleColon
            && word(&tokens[at + 1], "INTERVAL")
            && matches!(tokens[at - 1], Token::SingleQuotedString(_))
            && (word(&tokens[at + 2], "PRECEDING") || word(&tokens[at + 2], "FOLLOWING"))
        {
            tokens.drain(at..at + 2);
            return true;
        }
    }
    false
}

fn at_local(tokens: &mut Vec<Token>) -> bool {
    for at in 1..tokens.len().saturating_sub(1) {
        if word(&tokens[at], "AT")
            && word(&tokens[at + 1], "LOCAL")
            && matches!(tokens[at - 1], Token::SingleQuotedString(_))
            && tokens[..at].iter().any(|token| word(token, "TIMESTAMP"))
        {
            tokens.drain(at..at + 2);
            return true;
        }
    }
    false
}
