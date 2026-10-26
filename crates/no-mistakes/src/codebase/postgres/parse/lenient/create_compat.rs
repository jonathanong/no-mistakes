//! Preserve typed CREATE TABLE facts across PostgreSQL syntax gaps.
use super::pg_ident;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};
#[path = "create_compat/clauses.rs"]
mod clauses;
#[path = "create_compat/original.rs"]
mod original;
#[cfg(test)]
#[path = "create_compat/tests.rs"]
mod tests;
use clauses::*;

pub(super) fn valid_original(source: Option<&[TokenWithSpan]>) -> bool {
    original::valid(source)
}

pub(super) fn rewrite(tokens: &mut Vec<TokenWithSpan>) {
    if !valid_original(Some(tokens)) {
        return;
    }
    let mut clone = tokens.clone();
    let code = significant(&clone);
    let plain: Vec<_> = code.iter().map(|at| clone[*at].token.clone()).collect();
    if pg_ident::table_head(&plain).is_none() {
        return;
    }
    let mut changed = false;
    for _ in 0..code.len() {
        let code = significant(&clone);
        let plain: Vec<_> = code.iter().map(|at| clone[*at].token.clone()).collect();
        let body = pg_ident::table_head(&plain).filter(|at| plain.get(*at) == Some(&Token::LParen));
        if !(body.is_some_and(|open| column_modifiers(&mut clone, &code, open))
            || partition_columns(&mut clone, &code)
            || generated_default(&mut clone, &code)
            || table_options(&mut clone, &code))
        {
            break;
        }
        changed = true;
    }
    super::rewrite::rewrite_referential_set_column_lists_located(&mut clone);
    if clone != *tokens {
        changed = true;
    }
    if changed && parsed_table(&clone) {
        *tokens = clone;
    }
}

fn significant(tokens: &[TokenWithSpan]) -> Vec<usize> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, item)| !matches!(item.token, Token::Whitespace(_)))
        .map(|(at, _)| at)
        .collect()
}

fn parsed_table(tokens: &[TokenWithSpan]) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens.to_vec());
    matches!(parser.parse_statement(), Ok(statement @ Statement::CreateTable(_)) if super::pg_ast::valid(&statement))
        && parser.peek_token().token == Token::EOF
}
