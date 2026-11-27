//! PostgreSQL permits VALUES directly as a quantified comparison subquery.
use super::super::{matching, word};
use sqlparser::tokenizer::Token;

pub(super) fn values(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(2) {
        if !(word(&tokens[at], "ANY") || word(&tokens[at], "ALL"))
            || tokens[at + 1] != Token::LParen
            || !word(&tokens[at + 2], "VALUES")
        {
            continue;
        }
        let Some(end) = matching(tokens, at + 1, Token::LParen, Token::RParen) else {
            continue;
        };
        tokens.splice(
            end..end,
            [
                Token::RParen,
                Token::make_keyword("AS"),
                Token::make_keyword("nm_values"),
            ],
        );
        tokens.splice(
            at + 2..at + 2,
            [
                Token::make_keyword("SELECT"),
                Token::Mul,
                Token::make_keyword("FROM"),
                Token::LParen,
            ],
        );
        return true;
    }
    false
}
