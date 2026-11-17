use super::super::{leading_query_annotation, EmbeddedSqlCall, EmbeddedSqlKind};
use super::ExecutedPostgresSql;

pub(super) fn project_calls(
    file_name: &str,
    calls: Vec<EmbeddedSqlCall>,
) -> Vec<ExecutedPostgresSql> {
    let mut statements = Vec::new();
    for call in calls {
        if call.variants.is_empty() {
            let values_only = call.kind != EmbeddedSqlKind::Dynamic
                && only_value_binds(
                    call.sql_text.as_deref(),
                    &call.recovered_placeholder_positions,
                );
            statements.push(statement(
                file_name,
                call.line,
                call.sql_text,
                call.kind != EmbeddedSqlKind::Dynamic
                    && call.recovered_placeholder_positions.is_empty(),
                values_only,
                call.kind,
            ));
        } else {
            let truncated = call.variants_truncated();
            for variant in call.variants {
                let values_only = !truncated
                    && only_value_binds(
                        Some(&variant.sql_text),
                        &variant.recovered_placeholder_positions,
                    );
                let mut projected = statement(
                    file_name,
                    call.line,
                    Some(variant.sql_text),
                    !truncated && variant.recovered_placeholder_positions.is_empty(),
                    values_only,
                    if truncated {
                        EmbeddedSqlKind::Dynamic
                    } else {
                        EmbeddedSqlKind::Composed
                    },
                );
                if truncated {
                    projected.incomplete_reason = Some("sql-variants-truncated".into());
                }
                statements.push(projected);
            }
        }
    }
    statements
}

fn only_value_binds(text: Option<&str>, positions: &[(u32, u32)]) -> bool {
    let Some(text) = text.filter(|_| !positions.is_empty()) else {
        return false;
    };
    let (mut line, mut column, mut found) = (1, 1, 0);
    for character in text.chars() {
        if positions.get(found) == Some(&(line, column)) {
            if character != '$' {
                return false;
            }
            found += 1;
            if found == positions.len() {
                return true;
            }
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    false
}

#[cfg(test)]
mod tests;

pub(super) fn statement(
    file_name: &str,
    line: u32,
    sql_text: Option<String>,
    complete: bool,
    only_value_placeholders: bool,
    kind: EmbeddedSqlKind,
) -> ExecutedPostgresSql {
    let annotation = sql_text
        .as_deref()
        .and_then(leading_query_annotation)
        .map(str::to_owned);
    ExecutedPostgresSql {
        file_name: file_name.to_string(),
        line,
        annotation,
        sql_text,
        complete,
        only_value_placeholders,
        kind,
        incomplete_reason: (!complete && !only_value_placeholders)
            .then(|| "unresolved-sql-fragment".into()),
    }
}
