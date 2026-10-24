use super::*;
use sqlparser::ast::{SetExpr, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

pub(super) fn push_query_payload(
    body: &RoutineBody,
    chunk: &[TokenWithSpan],
    chunk_end: usize,
    ordinal: usize,
    executed: bool,
    result: &mut Vec<DynamicSql>,
) {
    let code: Vec<_> = chunk
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect();
    // A complete SQL DECLARE CURSOR already owns its query's calls. A PL/pgSQL
    // declaration with cursor parameters is not necessarily a SQL DECLARE AST.
    if complete_sql_declare(&code) {
        return;
    }
    let Some(at) = code.windows(2).position(|pair| {
        (keyword(pair[0], "FOR") || keyword(pair[0], "IN") || keyword(pair[0], "IS"))
            && (keyword(pair[1], "SELECT") || keyword(pair[1], "WITH"))
    }) else {
        return;
    };
    let query = code[at + 1];
    let start = location_offset(&body.sql, query.span.start.line, query.span.start.column);
    let loop_token = keyword(code[at], "IN")
        .then(|| loop_boundary(&code[at + 2..]))
        .flatten();
    let loop_start = loop_token.and_then(|token| {
        location_offset(&body.sql, token.span.start.line, token.span.start.column)
    });
    let (Some(start), end) = (start, loop_start.unwrap_or(chunk_end)) else {
        return;
    };
    if start >= end {
        return;
    }
    // The guarded DML suffix recovery already owns function and schema facts.
    // A WITH-prefixed DML query reaches here as a Query AST, so avoid projecting it twice.
    if keyword(query, "WITH") && with_dml_body(&code[at + 1..], loop_token) {
        return;
    }
    push_schema_statement(body, start, end, ordinal, executed, false, result);
    if let Some(payload) = result.last_mut() {
        payload.query_payload = true;
        // A declared cursor's query is parsed now but runs only when opened later.
        payload.executed &= !code.iter().any(|token| keyword(token, "CURSOR"));
    }
}

fn with_dml_body(code: &[&TokenWithSpan], loop_token: Option<&TokenWithSpan>) -> bool {
    let end = loop_token.and_then(|end| code.iter().position(|token| std::ptr::eq(*token, end)));
    let tokens = code[..end.unwrap_or(code.len())]
        .iter()
        .map(|token| (*token).clone())
        .collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    matches!(parser.parse_statement(), Ok(Statement::Query(query))
        if matches!(query.body.as_ref(), SetExpr::Insert(_) | SetExpr::Update(_) | SetExpr::Delete(_)))
        && parser.peek_token().token == Token::EOF
}

fn complete_sql_declare(code: &[&TokenWithSpan]) -> bool {
    if !code.first().is_some_and(|token| keyword(token, "DECLARE")) {
        return false;
    }
    let tokens = code.iter().map(|token| (*token).clone()).collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    matches!(parser.parse_statement(), Ok(Statement::Declare { .. }))
        && parser.peek_token().token == Token::EOF
}

fn loop_boundary<'a>(code: &[&'a TokenWithSpan]) -> Option<&'a TokenWithSpan> {
    let mut depth = 0usize;
    let mut cases = 0usize;
    for token in code {
        match &token.token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            _ if keyword(token, "CASE") => cases += 1,
            _ if keyword(token, "END") => cases = cases.saturating_sub(1),
            _ if depth == 0 && cases == 0 && keyword(token, "LOOP") => return Some(token),
            _ => {}
        }
    }
    None
}

fn keyword(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
