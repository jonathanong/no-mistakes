use super::super::{finding, trigger_finding, unanalyzable_finding};
use super::history::Catalogs;
use crate::codebase::postgres::dml::writes::positional_insert_hits;
use crate::codebase::postgres::{
    recovered_sql_may_write_columns, EmbeddedSqlFileFacts, SqlStatementFileFacts, SqlWriteColumns,
};

/// An opaque tail may introduce another write even when recovered targets
/// are literal and untracked. Only an empty applicable catalog is safe.
pub(super) fn extend_unanalyzable<'a>(
    findings: &mut crate::codebase::rules::PostgresFindings,
    file: &str,
    embedded: &EmbeddedSqlFileFacts,
    _statements: &[SqlStatementFileFacts],
    lookup: &dyn Fn(usize) -> &'a Catalogs,
) {
    for call in &embedded.calls {
        let line = call.line.max(1) as usize;
        if call.is_unanalyzable()
            && recovered_sql_may_write_columns(call.sql_text.as_deref())
            && !lookup(line).combined.is_empty()
        {
            findings.push(unanalyzable_finding(file, line));
        }
    }
}

pub(super) fn extend_writes<'a>(
    findings: &mut crate::codebase::rules::PostgresFindings,
    file: &str,
    statements: &[SqlStatementFileFacts],
    dedup: &mut crate::codebase::rules::VariantFindingDedup,
    source: Option<&str>,
    lookup: &dyn Fn(usize) -> &'a Catalogs,
) {
    for statement in statements {
        let mut hits = Vec::new();
        for (write_index, write) in statement.writes.iter().enumerate() {
            let catalog = &lookup(write.line).combined;
            let Some(meta) = catalog.get(&write.table) else {
                continue;
            };
            let columns: Vec<String> = match &write.columns {
                SqlWriteColumns::Named(columns) => columns
                    .iter()
                    .filter(|column| meta.generated.contains(&column.to_ascii_lowercase()))
                    .cloned()
                    .collect(),
                SqlWriteColumns::Positional(width) => {
                    positional_insert_hits(&write.table, meta, *width)
                        .into_iter()
                        .map(|hit| hit.column)
                        .collect()
                }
                SqlWriteColumns::All => meta.generated.iter().cloned().collect(),
            };
            for column in columns {
                hits.push((write.line, meta.name.as_str(), column, write_index));
            }
        }
        hits.sort();
        hits.dedup();
        for (line, table, column, write_index) in hits {
            let generated = &lookup(line).catalog;
            let render = if generated
                .get_exact(table)
                .or_else(|| generated.get(table))
                .is_some_and(|meta| meta.generated.contains(&column.to_ascii_lowercase()))
            {
                finding
            } else {
                trigger_finding
            };
            dedup.push(
                findings,
                statement,
                write_site(statement, write_index, &column),
                source,
                render(file, line, table, &column),
            );
        }
    }
}
fn write_site(
    statement: &SqlStatementFileFacts,
    index: usize,
    column: &str,
) -> crate::codebase::postgres::statements::SqlFactSite {
    use crate::codebase::postgres::statements::SqlFactSite;
    let named = SqlFactSite::WriteColumn(index, column.to_string());
    if statement
        .variant_locations
        .as_ref()
        .is_some_and(|locations| locations.position(named.clone()).is_some())
    {
        named
    } else {
        SqlFactSite::Write(index)
    }
}
