use super::literal::{normalize_format, string_expression};
use super::routine::{bodies, RoutineBody};
use super::*;
use std::collections::HashMap;

/// Only static `EXECUTE` expressions become SQL facts. Runtime concatenation is
/// intentionally opaque; assigned variables are invalidated on nonstatic write.
pub(super) fn extract(sql: &str) -> Vec<DynamicSql> {
    bodies(sql).iter().flat_map(extract_body).collect()
}

fn extract_body(body: &RoutineBody) -> Vec<DynamicSql> {
    let all = tokenize(&body.sql);
    let mut variables = HashMap::<String, Option<DynamicSql>>::new();
    let mut result = Vec::new();
    let mut scope = execution::Scope::default();
    for (ordinal, statement) in statements(&all).into_iter().enumerate() {
        let code = significant(statement);
        let definite = scope.advance(&code);
        let executed = definite && body.executed;
        if code.is_empty() {
            continue;
        }
        if let Some(at) = code.iter().position(|token| word(token, "EXECUTE")) {
            let line = body_line(body, code[at]);
            if let Some(mut sql) = executed_expression(&code[at + 1..], &variables, line) {
                sql.executed &= executed;
                sql.source_order = body
                    .source_order
                    .iter()
                    .copied()
                    .chain(std::iter::once(ordinal))
                    .collect();
                result.push(sql);
            }
            continue;
        }
        if let Some(at) = assignment_at(&code) {
            if let Some(name) = assignment_name(&code, at) {
                let line = body_line(body, code[at]);
                let mut value = executed_expression(&code[at + 1..], &variables, line);
                if let Some(value) = &mut value {
                    // Retain syntactic SQL for policy facts, but invalidate definite
                    // execution when a conditional assignment can replace its value.
                    value.executed &= definite;
                }
                variables.insert(name.to_ascii_lowercase(), value);
            }
        }
    }
    result
}

fn executed_expression(
    tokens: &[&TokenWithSpan],
    variables: &HashMap<String, Option<DynamicSql>>,
    line: usize,
) -> Option<DynamicSql> {
    let expression = execution_tokens(tokens);
    if expression.len() == 1 {
        if let Some(value) = identifier(expression[0])
            .and_then(|name| variables.get(&name.to_ascii_lowercase()))
            .and_then(|value| value.as_ref())
        {
            return Some(value.clone());
        }
    }
    static_expression(expression).map(|sql| DynamicSql::anchored(sql, line))
}

fn execution_tokens<'a>(tokens: &'a [&TokenWithSpan]) -> &'a [&'a TokenWithSpan] {
    let end = tokens
        .iter()
        .position(|token| word(token, "INTO") || word(token, "USING"))
        .unwrap_or(tokens.len());
    &tokens[..end]
}

fn static_expression(tokens: &[&TokenWithSpan]) -> Option<String> {
    if tokens
        .iter()
        .any(|token| matches!(token.token, Token::StringConcat | Token::Plus))
    {
        return None;
    }
    if let Some(decoded) = string_expression(tokens, None) {
        return Some(decoded.sql);
    }
    if let Some(argument_start) = format_argument_start(tokens) {
        let comma = tokens
            .iter()
            .position(|token| matches!(token.token, Token::Comma))
            .unwrap_or(tokens.len().saturating_sub(1));
        return string_expression(&tokens[argument_start..comma], None)
            .map(|decoded| normalize_format(&decoded.sql));
    }
    None
}

fn format_argument_start(tokens: &[&TokenWithSpan]) -> Option<usize> {
    if tokens.len() >= 3 && word(tokens[0], "FORMAT") && matches!(tokens[1].token, Token::LParen) {
        return Some(2);
    }
    (tokens.len() >= 5
        && word(tokens[0], "pg_catalog")
        && matches!(tokens[1].token, Token::Period)
        && word(tokens[2], "FORMAT")
        && matches!(tokens[3].token, Token::LParen))
    .then_some(4)
}

fn assignment_name<'a>(tokens: &'a [&TokenWithSpan], assignment: usize) -> Option<&'a str> {
    let line = tokens[assignment].span.start.line;
    let identifiers = tokens[..assignment]
        .iter()
        .filter(|token| token.span.start.line == line)
        .filter_map(|token| identifier(token))
        .collect::<Vec<_>>();
    let boundary = identifiers.iter().rposition(|name| {
        matches!(
            name.to_ascii_uppercase().as_str(),
            "DECLARE" | "BEGIN" | "THEN" | "ELSE" | "LOOP"
        )
    });
    let name = identifiers.get(boundary.map_or(0, |index| index + 1))?;
    (!is_control_word(name)).then_some(*name)
}

fn assignment_at(tokens: &[&TokenWithSpan]) -> Option<usize> {
    tokens
        .iter()
        .position(|token| matches!(token.token, Token::Assignment))
        .or_else(|| {
            tokens.iter().enumerate().find_map(|(index, token)| {
                if !matches!(token.token, Token::Eq) {
                    return None;
                }
                assignment_name(tokens, index).map(|_| index)
            })
        })
}

fn is_control_word(word: &str) -> bool {
    matches!(
        word.to_ascii_uppercase().as_str(),
        "IF" | "ELSIF" | "WHILE" | "WHEN" | "EXIT" | "RETURN" | "BEGIN" | "LOOP"
    )
}
