//! Validate cursor declarations whose VALUES subquery exceeds sqlparser's expression grammar.
use super::{Cursor, Token};
use sqlparser::{
    ast::{SetExpr, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if !cursor.word("DECLARE")
        || !cursor.identifier()
        || !cursor.words(&["SCROLL", "CURSOR", "FOR"])
    {
        return false;
    }
    let mut query: Vec<_> = cursor.tokens[cursor.at..]
        .iter()
        .map(|token| (**token).clone())
        .collect();
    let Some(start) = (0..query.len().saturating_sub(2)).find(|at| {
        word(&query[*at], "ALL")
            && query[*at + 1] == Token::LParen
            && word(&query[*at + 2], "VALUES")
    }) else {
        return false;
    };
    let Some(end) = closing(&query, start + 1) else {
        return false;
    };
    let mut values = Parser::new(&PostgreSqlDialect {}).with_tokens(query[start + 2..end].to_vec());
    let valid_values = values.parse_statement().is_ok_and(|statement| {
        super::super::query::valid_ast(&statement)
            && matches!(&statement,
                Statement::Query(query) if matches!(query.body.as_ref(),
                    SetExpr::Values(rows) if !rows.rows.is_empty()
                        && rows.rows.iter().all(|row| row.content.len() == 1)))
    }) && values.peek_token().token == Token::EOF;
    if !valid_values {
        return false;
    }
    query.splice(
        start + 2..end,
        [
            Token::make_keyword("SELECT"),
            Token::Number("1".into(), false),
        ],
    );
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(query);
    if !matches!(parser.parse_statement(), Ok(statement @ Statement::Query(_))
        if super::super::query::valid_ast(&statement))
        || parser.peek_token().token != Token::EOF
    {
        return false;
    }
    cursor.at = cursor.tokens.len();
    true
}

fn closing(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
    }
    None
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
