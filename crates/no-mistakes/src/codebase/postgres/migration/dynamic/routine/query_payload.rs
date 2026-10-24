use super::*;
use sqlparser::ast::Statement;
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
    let start = location_offset(&body.sql, query.span.start.line, query.span.start.column)
        .expect("query token belongs to the routine body");
    let loop_token = keyword(code[at], "IN").then(|| loop_boundary(&code[at + 2..]));
    let loop_start = loop_token.map(|token| {
        location_offset(&body.sql, token.span.start.line, token.span.start.column)
            .expect("loop token belongs to the routine body")
    });
    let end = loop_start.unwrap_or(chunk_end);
    assert!(start < end, "validated query starts before its chunk end");
    push_schema_statement(body, start, end, ordinal, executed, false, result);
    let payload = result.last_mut().expect("query payload was just added");
    payload.query_payload = true;
    payload.call_range = Some(CallRange {
        start,
        end,
        physical: body.peeled_column_offset.map(|column| {
            (
                physical_position(body, column, start),
                physical_position(body, column, end),
            )
        }),
    });
    // A declared cursor's query is parsed now but runs only when opened later.
    payload.executed &= !code.iter().any(|token| keyword(token, "CURSOR"));
}

fn physical_position(body: &RoutineBody, first_column_offset: usize, at: usize) -> (usize, usize) {
    let prefix = &body.sql[..at];
    let line = body.line + prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |at| at + 1);
    let column = prefix[line_start..].chars().count() + 1;
    (
        line,
        column
            + if line_start == 0 {
                first_column_offset
            } else {
                0
            },
    )
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

fn loop_boundary<'a>(code: &[&'a TokenWithSpan]) -> &'a TokenWithSpan {
    let mut depth = 0usize;
    let mut cases = 0usize;
    code.iter()
        .find(|token| match &token.token {
            Token::LParen | Token::LBracket => {
                depth += 1;
                false
            }
            Token::RParen | Token::RBracket => {
                depth = depth.saturating_sub(1);
                false
            }
            _ if keyword(token, "CASE") => {
                cases += 1;
                false
            }
            _ if keyword(token, "END") => {
                cases = cases.saturating_sub(1);
                false
            }
            _ => depth == 0 && cases == 0 && keyword(token, "LOOP"),
        })
        .copied()
        .expect("validated FOR header contains LOOP")
}

fn keyword(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
