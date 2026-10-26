//! Parser-clone rewrites for complete PostgreSQL regression query forms.
use super::super::{expression, matching, word};
use sqlparser::tokenizer::Token;

pub(super) fn xml_table(tokens: &mut Vec<Token>) -> bool {
    let Some(at) = tokens.iter().enumerate().position(|(at, token)| {
        word(token, "XMLTABLE") && tokens.get(at + 1) == Some(&Token::LParen)
    }) else {
        return false;
    };
    // XMLTABLE is a table reference here, never a scalar SELECT expression.
    if at == 0 || !word(&tokens[at - 1], "LATERAL") {
        return false;
    }
    let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
        return false;
    };
    let args = &tokens[at + 2..end];
    if args.len() < 6
        || !matches!(args[0], Token::SingleQuotedString(_))
        || !word(&args[1], "PASSING")
    {
        return false;
    }
    let Some(columns) = args.iter().position(|token| word(token, "COLUMNS")) else {
        return false;
    };
    if columns <= 2 || !expression(&args[2..columns]) || !columns_valid(&args[columns + 1..]) {
        return false;
    }
    tokens.splice(
        at + 1..=end,
        [
            Token::LParen,
            Token::SingleQuotedString("path".into()),
            Token::RParen,
        ],
    );
    true
}

fn columns_valid(tokens: &[Token]) -> bool {
    let mut start = 0;
    loop {
        let end = tokens[start..]
            .iter()
            .position(|token| *token == Token::Comma)
            .map_or(tokens.len(), |offset| start + offset);
        let item = &tokens[start..end];
        let type_end = item
            .iter()
            .position(|token| word(token, "PATH"))
            .unwrap_or(item.len());
        if type_end < 2
            || !super::super::super::super::ddl_foreign::valid_identifier(&item[0])
            || !super::super::super::super::ddl_foreign::valid_type(&item[1..type_end])
            || type_end < item.len()
                && !(type_end + 2 == item.len()
                    && matches!(item[type_end + 1], Token::SingleQuotedString(_)))
        {
            return false;
        }
        if end == tokens.len() {
            return true;
        }
        start = end + 1;
    }
}

pub(super) fn sample_null(tokens: &mut [Token]) -> bool {
    let Some(sample) = tokens.iter().position(|token| word(token, "TABLESAMPLE")) else {
        return false;
    };
    let open = sample + 2;
    if !matches!(tokens.get(sample + 1), Some(Token::Word(_)))
        || tokens.get(open) != Some(&Token::LParen)
    {
        return false;
    }
    let Some(close) = matching(tokens, open, Token::LParen, Token::RParen) else {
        return false;
    };
    let null_at = if close == open + 2 && word(&tokens[open + 1], "NULL") {
        Some(open + 1)
    } else if tokens
        .get(close + 1)
        .is_some_and(|token| word(token, "REPEATABLE"))
        && tokens.get(close + 2) == Some(&Token::LParen)
        && tokens
            .get(close + 3)
            .is_some_and(|token| word(token, "NULL"))
        && tokens.get(close + 4) == Some(&Token::RParen)
    {
        Some(close + 3)
    } else {
        None
    };
    if let Some(at) = null_at {
        tokens[at] = Token::Number("0".into(), false);
    }
    null_at.is_some()
}

pub(super) fn cte_search(tokens: &mut Vec<Token>) -> bool {
    let Some(body_end) = first_cte_body_end(tokens) else {
        return false;
    };
    for at in 1..tokens.len().saturating_sub(6) {
        if at != body_end + 1
            || tokens[at - 1] != Token::RParen
            || !word(&tokens[at], "SEARCH")
            || !(word(&tokens[at + 1], "DEPTH") || word(&tokens[at + 1], "BREADTH"))
            || !word(&tokens[at + 2], "FIRST")
            || !word(&tokens[at + 3], "BY")
        {
            continue;
        }
        let mut end = at + 4;
        loop {
            if !tokens
                .get(end)
                .is_some_and(super::super::super::super::ddl_foreign::valid_identifier)
            {
                return false;
            }
            end += 1;
            if tokens.get(end) != Some(&Token::Comma) {
                break;
            }
            end += 1;
        }
        if !tokens.get(end).is_some_and(|token| word(token, "SET"))
            || !tokens
                .get(end + 1)
                .is_some_and(super::super::super::super::ddl_foreign::valid_identifier)
        {
            return false;
        }
        tokens.drain(at..end + 2);
        return true;
    }
    false
}

fn first_cte_body_end(tokens: &[Token]) -> Option<usize> {
    if !tokens.first().is_some_and(|token| word(token, "WITH")) {
        return None;
    }
    let mut at = 1;
    if tokens.get(at).is_some_and(|token| word(token, "RECURSIVE")) {
        at += 1;
    }
    if !tokens
        .get(at)
        .is_some_and(super::super::super::super::ddl_foreign::valid_identifier)
    {
        return None;
    }
    at += 1;
    if tokens.get(at) == Some(&Token::LParen) {
        at = matching(tokens, at, Token::LParen, Token::RParen)? + 1;
    }
    if !tokens.get(at).is_some_and(|token| word(token, "AS")) {
        return None;
    }
    at += 1;
    if tokens.get(at).is_some_and(|token| word(token, "NOT")) {
        at += 1;
    }
    if tokens
        .get(at)
        .is_some_and(|token| word(token, "MATERIALIZED"))
    {
        at += 1;
    }
    (tokens.get(at) == Some(&Token::LParen))
        .then(|| matching(tokens, at, Token::LParen, Token::RParen))
        .flatten()
}

pub(super) fn whole_row_cast(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(4) {
        if super::super::super::super::ddl_foreign::valid_identifier(&tokens[at])
            && tokens[at + 1] == Token::Period
            && tokens[at + 2] == Token::Mul
            && tokens[at + 3] == Token::DoubleColon
            && super::super::super::super::ddl_foreign::valid_type(&tokens[at + 4..at + 5])
        {
            tokens.drain(at + 1..at + 3);
            return true;
        }
    }
    false
}
