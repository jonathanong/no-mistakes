use super::literal::{leading_string_expression, string_expression};
use super::*;
mod wrapper;

#[derive(Clone, Debug)]
pub(super) struct RoutineBody {
    pub(super) sql: String,
    pub(super) line: usize,
    pub(super) source_bytes: Vec<usize>,
    direct_facts_already_recovered: bool,
    pub(super) source_order: Vec<usize>,
    pub(super) executed: bool,
    clear_outer_wrapper: bool,
}

/// Executable PL/pgSQL DO/function/procedure bodies, including dollar, plain,
/// E, U&, and PostgreSQL newline-concatenated string literals.
pub(super) fn bodies(sql: &str) -> Vec<RoutineBody> {
    let normalized = super::super::super::parse::normalize_copy_data_for_tokens(sql);
    let all = tokenize(&normalized);
    statements(&all)
        .into_iter()
        .enumerate()
        .filter_map(|(ordinal, statement)| {
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
                if code.get(1).is_some_and(|token| word(token, "LANGUAGE")) {
                    3
                } else {
                    1
                }
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
                source_order: vec![ordinal],
                executed: do_block,
                clear_outer_wrapper: if routine {
                    wrapper::complete_routine(&code)
                } else {
                    do_block && !decoded.dollar_quoted && complete_body_suffix(&code, start, sql)
                },
            })
        })
        .collect()
}

fn complete_body_suffix(code: &[&TokenWithSpan], start: usize, sql: &str) -> bool {
    let end = if code.len() >= start + 3
        && word(code[code.len() - 2], "LANGUAGE")
        && crate::codebase::postgres::parse::is_plpgsql_language(&code[code.len() - 1].token)
    {
        code.len() - 2
    } else {
        code.len()
    };
    string_expression(&code[start..end], Some(sql)).is_some()
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

pub(in super::super) fn peeled_schema_bodies(sql: &str) -> Vec<DynamicSql> {
    bodies(sql)
        .into_iter()
        .filter(|body| body.direct_facts_already_recovered)
        .flat_map(|body| schema_statements(&body))
        .collect()
}

fn schema_statements(body: &RoutineBody) -> Vec<DynamicSql> {
    let tokens = tokenize(&body.sql);
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut start_token = 0usize;
    let mut ordinal = 0usize;
    let mut scope = execution::Scope::default();
    let mut procedural = super::super::super::parse::ProceduralBodyState::default();
    for (at, token) in tokens.iter().enumerate() {
        if !matches!(token.token, Token::SemiColon) {
            continue;
        }
        let end = location_offset(&body.sql, token.span.start.line, token.span.start.column)
            .map_or(body.sql.len(), |offset| offset + 1);
        let chunk = &tokens[start_token..at];
        let executed = scope.advance(&significant(chunk)) && body.executed;
        let control = procedural.control_chunk(chunk);
        push_schema_statement(body, start, end, ordinal, executed, control, &mut result);
        start_token = at + 1;
        start = end;
        ordinal += 1;
    }
    let control = procedural.control_chunk(&tokens[start_token..]);
    push_schema_statement(
        body,
        start,
        body.sql.len(),
        ordinal,
        scope.advance(&significant(&tokens[start_token..])) && body.executed,
        control,
        &mut result,
    );
    result
}

fn push_schema_statement(
    body: &RoutineBody,
    start: usize,
    end: usize,
    ordinal: usize,
    executed: bool,
    procedural_control: bool,
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
        executed,
        procedural_control,
        clear_outer_wrapper: body.clear_outer_wrapper,
        sql: sql.to_owned(),
        line: source_lines.first().copied().unwrap_or(body.line),
        source_lines,
        source_order: body
            .source_order
            .iter()
            .copied()
            .chain(std::iter::once(ordinal))
            .collect(),
    });
}
