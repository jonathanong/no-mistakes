//! One decision for "this equality proves `column IS NOT NULL`".
//! See docs/rules/postgres-bounded-statements-proof.md.
use super::implied::text_family_column;
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::CatalogTable;

/// `arm_equated` is raw equality on this `OR` arm. It proves `IS NOT NULL` for every catalog type.
pub(super) fn proves(
    column: &str,
    item: &SqlBoundItem,
    table: &CatalogTable,
    arm_equated: bool,
) -> bool {
    arm_equated
        || listed(&item.nonnull_columns, column)
        || (listed(&item.strict_nonnull_columns, column) && text_family_column(table, column))
}

fn listed(columns: &[String], column: &str) -> bool {
    columns.iter().any(|name| name == column)
}

#[cfg(test)]
#[path = "not_null_tests.rs"]
mod tests;
