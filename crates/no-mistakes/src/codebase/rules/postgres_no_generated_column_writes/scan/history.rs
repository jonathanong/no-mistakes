use super::CompiledOptions;
use crate::codebase::postgres::dml::GeneratedTableColumns;
use crate::codebase::postgres::{SqlSchemaFileFacts, SqlStatementFileFacts};
use std::collections::BTreeMap;

pub(super) struct Catalogs {
    pub(super) catalog: GeneratedTableColumns,
    pub(super) trigger: GeneratedTableColumns,
    pub(super) combined: GeneratedTableColumns,
    pub(super) known_tables: BTreeMap<String, bool>,
}

impl Catalogs {
    pub(super) fn build(
        tables: &super::super::catalog::LiveTables<'_>,
        opts: &CompiledOptions,
    ) -> Self {
        let catalog =
            super::super::catalog::catalog_from_tables(tables, &opts.extra_generated_columns);
        let trigger = super::super::catalog::trigger_catalog_from_tables(
            tables,
            &opts.trigger_maintained_columns,
        );
        let mut combined = trigger.clone();
        combined.extend_from(&catalog);
        let known_tables = tables
            .iter()
            .map(|(name, table)| (name.clone(), table.complete))
            .collect();
        Self {
            catalog,
            trigger,
            combined,
            known_tables,
        }
    }

    pub(super) fn known_complete(&self, name: &str) -> bool {
        let key = crate::codebase::postgres::idents::resolve_relation_key(
            self.known_tables.keys().map(String::as_str),
            name,
        );
        key.and_then(|key| self.known_tables.get(&key)) == Some(&true)
    }
}

/// Number of this file's schema statements that run before a write on `line`.
pub(super) fn events_before(file: &SqlSchemaFileFacts, line: usize) -> usize {
    file.table_events
        .iter()
        .filter(|event| event.precedes_write(line))
        .count()
}

/// Catalogs keyed by `events_before` for every distinct write position in one migration.
pub(super) fn snapshots(
    schema: &[&SqlSchemaFileFacts],
    file: &SqlSchemaFileFacts,
    statements: &[SqlStatementFileFacts],
    opts: &CompiledOptions,
) -> BTreeMap<usize, Catalogs> {
    let mut out = BTreeMap::new();
    for line in statements
        .iter()
        .flat_map(|statement| statement.writes.iter().map(|write| write.line))
    {
        out.entry(events_before(file, line)).or_insert_with(|| {
            let tables = super::super::catalog::tables_before(schema, &file.path, line);
            Catalogs::build(&tables, opts)
        });
    }
    out
}
