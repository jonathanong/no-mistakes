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
    let Some(at) = code.windows(2).position(|pair| {
        (word(pair[0], "FOR") || word(pair[0], "IN") || word(pair[0], "IS"))
            && (word(pair[1], "SELECT") || word(pair[1], "WITH"))
    }) else {
        return;
    };
    let query = code[at + 1];
    let start = location_offset(&body.sql, query.span.start.line, query.span.start.column);
    let loop_start = code[at + 2..]
        .iter()
        .rev()
        .find(|token| word(token, "LOOP"))
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
        payload.executed &= !code.iter().any(|token| word(token, "CURSOR"));
    }
}
