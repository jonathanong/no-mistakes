use crate::codebase::postgres::statements::SqlBareRead;
use crate::codebase::postgres::SchemaCatalog;

/// Columns every table has without the catalog listing them.
const SYSTEM_COLUMNS: &[&str] = &["ctid", "tableoid", "xmin", "xmax", "cmin", "cmax"];

/// Whether a subquery reads the query around it: a bare column that none of the base tables it
/// could belong to has, so PostgreSQL resolves it outward. A table the catalog does not
/// describe may have it, and is taken to.
pub(super) fn reads_outer(reads: &[SqlBareRead], catalog: &SchemaCatalog) -> bool {
    reads.iter().any(|read| {
        !SYSTEM_COLUMNS.contains(&read.column.as_str())
            && read.tables.iter().all(|table| {
                catalog.bounded_relation(table).is_some_and(|table| {
                    table
                        .columns
                        .iter()
                        .all(|column| column.name != read.column)
                })
            })
    })
}
