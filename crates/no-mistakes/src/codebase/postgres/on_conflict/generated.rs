use super::form_is_self;
use crate::codebase::postgres::statement_facts::{
    SqlConflictArbiter, SqlInsertFact, SqlOnConflictFact,
};
use crate::codebase::postgres::types::SqlSchemaFileFacts;

pub(super) fn judge(
    insert: &SqlInsertFact,
    conflict: &SqlOnConflictFact,
    schema: &[SqlSchemaFileFacts],
) -> Option<String> {
    let arbiter_cols = match &conflict.arbiter {
        SqlConflictArbiter::Columns(columns) => columns.clone(),
        _ => return None,
    };
    if schema.iter().all(|file| file.tables.is_empty()) {
        return None;
    }
    let Some(table) = schema
        .iter()
        .flat_map(|file| file.tables.iter())
        .find(|table| table.table_name.eq_ignore_ascii_case(&insert.table))
    else {
        return Some(format!(
            "target table {} is missing from schema facts; generated-arbiter cannot be proven",
            insert.table
        ));
    };
    for column in table.columns.iter().filter(|column| column.is_generated) {
        if !arbiter_cols
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&column.name))
        {
            continue;
        }
        let sources = if column.generated_source_columns.is_empty() {
            &column.generated_function_arg_columns
        } else {
            &column.generated_source_columns
        };
        for assignment in &conflict.assignments {
            if sources
                .iter()
                .any(|source| source.eq_ignore_ascii_case(&assignment.column))
                && !form_is_self(&assignment.form, &assignment.column)
            {
                return Some(format!(
                    "ON CONFLICT DO UPDATE assigns a source column of generated arbiter {}",
                    column.name
                ));
            }
        }
    }
    None
}
