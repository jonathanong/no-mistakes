use crate::codebase::postgres::types::SqlDeclaredIdentifier;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn names(
    tokens: &[TokenWithSpan],
    name: String,
    mut next: usize,
    line: usize,
) -> Vec<SqlDeclaredIdentifier> {
    let mut names = vec![SqlDeclaredIdentifier { name, line }];
    if !matches!(
        tokens.get(next).map(|token| &token.token),
        Some(Token::LParen)
    ) {
        return names;
    }
    next += 1;
    while let Some((name, after)) = super::procedures::identifier(tokens, next) {
        names.push(SqlDeclaredIdentifier { name, line });
        if !matches!(
            tokens.get(after).map(|token| &token.token),
            Some(Token::Comma)
        ) {
            break;
        }
        next = after + 1;
    }
    names
}
