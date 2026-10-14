use super::super::{leading_query_annotation, EmbeddedSqlCall, EmbeddedSqlKind};
use super::ExecutedPostgresSql;

pub(super) fn project_calls(
    file_name: &str,
    calls: Vec<EmbeddedSqlCall>,
) -> Vec<ExecutedPostgresSql> {
    let mut statements = Vec::new();
    for call in calls {
        if call.variants.is_empty() {
            statements.push(statement(
                file_name,
                call.line,
                call.sql_text,
                call.kind != EmbeddedSqlKind::Dynamic
                    && call.recovered_placeholder_positions.is_empty(),
                false,
                call.kind,
            ));
        } else {
            for variant in call.variants {
                statements.push(statement(
                    file_name,
                    call.line,
                    Some(variant.sql_text),
                    variant.recovered_placeholder_positions.is_empty(),
                    false,
                    EmbeddedSqlKind::Composed,
                ));
            }
        }
    }
    statements
}

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
