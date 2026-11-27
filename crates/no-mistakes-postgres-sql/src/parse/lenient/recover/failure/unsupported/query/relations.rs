use super::{matching, word};
use sqlparser::tokenizer::Token;
#[path = "relations/extra.rs"]
mod extra;

pub(super) fn rewrite(tokens: &mut Vec<Token>) -> bool {
    table_star(tokens)
        || rows_from(tokens)
        || json_table(tokens)
        || extra::xml_table(tokens)
        || extra::sample_null(tokens)
        || extra::cte_search(tokens)
        || extra::whole_row_cast(tokens)
}

fn table_star(tokens: &mut Vec<Token>) -> bool {
    for at in 2..tokens.len() {
        if tokens[at] != Token::Mul || !matches!(tokens[at - 1], Token::Word(_)) {
            continue;
        }
        if !word(&tokens[at - 2], "FROM") && !word(&tokens[at - 2], "JOIN") {
            continue;
        }
        if tokens.get(at + 1).is_none_or(|next| {
            matches!(next, Token::Comma)
                || ["WHERE", "GROUP", "ORDER", "LIMIT", "OFFSET", "JOIN"]
                    .iter()
                    .any(|name| word(next, name))
        }) {
            tokens.remove(at);
            return true;
        }
    }
    false
}

fn rows_from(tokens: &mut Vec<Token>) -> bool {
    for at in 1..tokens.len().saturating_sub(3) {
        if !word(&tokens[at], "ROWS")
            || !word(&tokens[at + 1], "FROM")
            || tokens[at + 2] != Token::LParen
            || !word(&tokens[at - 1], "FROM")
        {
            continue;
        }
        let Some(end) = matching(tokens, at + 2, Token::LParen, Token::RParen) else {
            continue;
        };
        if end <= at + 3
            || !matches!(tokens[at + 3], Token::Word(_))
            || tokens.get(at + 4) != Some(&Token::LParen)
        {
            continue;
        }
        // One function is safely representable as an ordinary table function.
        if matching(tokens, at + 4, Token::LParen, Token::RParen) != Some(end - 1) {
            continue;
        }
        tokens.remove(end);
        tokens.drain(at..at + 3);
        return true;
    }
    false
}

fn json_table(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(1) {
        if !word(&tokens[at], "JSON_TABLE") || tokens[at + 1] != Token::LParen {
            continue;
        }
        let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
            continue;
        };
        let Some(columns) = (at + 2..end).find(|pos| word(&tokens[*pos], "COLUMNS")) else {
            continue;
        };
        if tokens.get(columns + 1) != Some(&Token::LParen) {
            continue;
        }
        let Some(close) = matching(tokens, columns + 1, Token::LParen, Token::RParen) else {
            continue;
        };
        if close + 1 != end
            || close != columns + 4
            || !super::super::super::ddl_foreign::valid_identifier(&tokens[columns + 2])
            || !super::super::super::ddl_foreign::valid_type(&tokens[columns + 3..close])
        {
            continue;
        }
        tokens.splice(
            close..close,
            [
                Token::make_keyword("PATH"),
                Token::SingleQuotedString("$".into()),
            ],
        );
        return true;
    }
    false
}
