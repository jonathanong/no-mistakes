use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlKind, SqlStatementFileFacts};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) struct PreparedSqlFragment {
    pub line: u32,
    pub statements: Arc<SqlStatementFileFacts>,
}

/// Parse each unexecuted recovered builder text once within its projection.
pub(super) fn collect(file: &EmbeddedSqlFileFacts) -> Vec<PreparedSqlFragment> {
    let executed: HashSet<_> = file
        .calls
        .iter()
        .filter(|call| call.kind != EmbeddedSqlKind::Dynamic)
        .filter_map(|call| call.sql_text.as_deref())
        .collect();
    let mut parsed = HashMap::new();
    file.fragments
        .iter()
        .filter_map(|fragment| {
            let sql = fragment.sql_text.as_deref()?;
            if executed.contains(sql) {
                return None;
            }
            let statements = parsed
                .entry(sql)
                .or_insert_with(|| Arc::new(statement_facts(sql)));
            Some(PreparedSqlFragment {
                line: fragment.line,
                statements: Arc::clone(statements),
            })
        })
        .collect()
}

fn statement_facts(sql: &str) -> SqlStatementFileFacts {
    use crate::codebase::postgres::statements::extract_sql_statement_facts;
    let direct = extract_sql_statement_facts(sql);
    if !direct.selects.is_empty() {
        return direct;
    }
    // Predicate-only fragments retain the legacy synthetic SELECT context.
    let prefix = sql.trim_start();
    let wrapper = if prefix.starts_with("AND ") || prefix.starts_with("OR ") {
        format!("SELECT 1 WHERE true {sql}")
    } else {
        format!("SELECT 1 WHERE {sql}")
    };
    extract_sql_statement_facts(&wrapper)
}

#[cfg(test)]
mod tests;
