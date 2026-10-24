use super::*;

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
    // A leading SQL DECLARE CURSOR wrapper is parsed directly with its query. Its calls
    // already belong to that AST; only later PL/pgSQL declaration chunks need projection.
    if code.first().is_some_and(|token| keyword(token, "DECLARE")) {
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
    let loop_start = keyword(code[at], "IN")
        .then(|| loop_boundary(&code[at + 2..]))
        .flatten()
        .and_then(|token| {
            location_offset(&body.sql, token.span.start.line, token.span.start.column)
        });
    let (Some(start), end) = (start, loop_start.unwrap_or(chunk_end)) else {
        return;
    };
    if start >= end {
        return;
    }
    push_schema_statement(body, start, end, ordinal, executed, false, result);
    if let Some(payload) = result.last_mut() {
        payload.query_payload = true;
        // A declared cursor's query is parsed now but runs only when opened later.
        payload.executed &= !code.iter().any(|token| keyword(token, "CURSOR"));
    }
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
