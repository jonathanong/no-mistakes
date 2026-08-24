use super::literal::leading_string_expression;
use super::*;

#[derive(Clone, Debug)]
pub(super) struct RoutineBody {
    pub(super) sql: String,
    pub(super) line: usize,
    pub(super) source_bytes: Vec<usize>,
    direct_facts_already_recovered: bool,
}

/// Executable PL/pgSQL DO/function/procedure bodies, including dollar, plain,
/// E, U&, and PostgreSQL newline-concatenated string literals.
pub(super) fn bodies(sql: &str) -> Vec<RoutineBody> {
    let all = tokenize(sql);
    statements(&all)
        .into_iter()
        .filter_map(|statement| {
            let code = significant(statement);
            let first = code.first()?;
            let do_block = word(first, "DO");
            let routine = word(first, "CREATE")
                && code
                    .iter()
                    .any(|token| word(token, "FUNCTION") || word(token, "PROCEDURE"));
            if !do_block && !routine
                || routine && !plpgsql(&code)
                || do_block && code.iter().any(|token| word(token, "LANGUAGE")) && !plpgsql(&code)
            {
                return None;
            }
            let start = if do_block {
                1
            } else {
                code.iter()
                    .position(|token| word(token, "AS"))
                    .map_or(code.len(), |index| index + 1)
            };
            leading_string_expression(&code[start..], Some(sql)).map(|decoded| RoutineBody {
                line: decoded.source_bytes.first().copied().unwrap_or(1),
                sql: decoded.sql,
                source_bytes: decoded.source_bytes,
                direct_facts_already_recovered: do_block && decoded.dollar_quoted,
            })
        })
        .collect()
}

/// Routine bodies whose direct schema statements are not already peeled by
/// the lenient parser. Dollar-quoted `DO` bodies are excluded to avoid
/// duplicate facts; function/procedure and quoted `DO` bodies still need the
/// shared migration extractor.
pub(super) fn schema_bodies(sql: &str) -> Vec<DynamicSql> {
    bodies(sql)
        .into_iter()
        .filter(|body| !body.direct_facts_already_recovered)
        .flat_map(|body| schema_statements(&body))
        .collect()
}

fn schema_statements(body: &RoutineBody) -> Vec<DynamicSql> {
    let tokens = tokenize(&body.sql);
    let mut result = Vec::new();
    let mut start = 0usize;
    for token in &tokens {
        if !matches!(token.token, Token::SemiColon) {
            continue;
        }
        let end = location_offset(&body.sql, token.span.start.line, token.span.start.column)
            .map_or(body.sql.len(), |offset| offset + 1);
        push_schema_statement(body, start, end, &mut result);
        start = end;
    }
    push_schema_statement(body, start, body.sql.len(), &mut result);
    result
}

fn push_schema_statement(
    body: &RoutineBody,
    start: usize,
    end: usize,
    result: &mut Vec<DynamicSql>,
) {
    let Some(sql) = body.sql.get(start..end) else {
        return;
    };
    if sql.trim().is_empty() {
        return;
    }
    let source_bytes = body.source_bytes.get(start..end).unwrap_or_default();
    let source_lines = source_lines(sql, source_bytes, body.line);
    result.push(DynamicSql {
        sql: sql.to_owned(),
        line: source_lines.first().copied().unwrap_or(body.line),
        source_lines,
    });
}
