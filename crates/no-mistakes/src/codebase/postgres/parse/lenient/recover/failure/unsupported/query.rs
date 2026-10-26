//! Validate selected complete PostgreSQL query syntax absent from sqlparser.
//! Rewrites run only on parser clones and never contribute made-up schema facts.
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

#[path = "query/expressions.rs"]
mod expressions;
#[path = "query/relations.rs"]
mod relations;
use crate::codebase::postgres::parse::lenient::pg_ast;

pub(super) fn valid(original: &[TokenWithSpan]) -> bool {
    let mut tokens: Vec<_> = original
        .iter()
        .map(|item| item.token.clone())
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    if tokens.is_empty()
        || !["SELECT", "WITH", "INSERT", "UPDATE", "VALUES", "CREATE"]
            .iter()
            .any(|head| word(&tokens[0], head))
        || word(&tokens[0], "CREATE") && !tokens.get(1).is_some_and(|t| word(t, "VIEW"))
    {
        return false;
    }
    let mut changed = false;
    for _ in 0..original.len() {
        if !(expressions::rewrite(&mut tokens) || relations::rewrite(&mut tokens)) {
            break;
        }
        changed = true;
    }
    changed && parse(&tokens)
}

fn parse(tokens: &[Token]) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens.to_vec());
    match parser.parse_statement() {
        Ok(
            statement @ (Statement::Query(_)
            | Statement::Insert(_)
            | Statement::Update { .. }
            | Statement::Merge { .. }
            | Statement::CreateView(_)),
        ) => parser.peek_token().token == Token::EOF && pg_ast::valid(&statement),
        _ => false,
    }
}

pub(super) fn valid_ast(value: &impl sqlparser::ast::Visit) -> bool {
    pg_ast::valid(value)
}

fn expression(tokens: &[Token]) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens.to_vec());
    parser.parse_expr().is_ok_and(|expr| pg_ast::valid(&expr))
        && parser.peek_token().token == Token::EOF
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none()
        && word.value.eq_ignore_ascii_case(expected))
}

fn matching(tokens: &[Token], open: usize, left: Token, right: Token) -> Option<usize> {
    let mut depth = 0;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        if *token == left {
            depth += 1;
        } else if *token == right {
            depth -= 1;
            if depth == 0 {
                return Some(at);
            }
        }
    }
    None
}
