//! One decision for "this equality proves `column IS NOT NULL`".
//! See docs/rules/postgres-bounded-statements-proof.md.
use super::implied::text_family_column;
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::CatalogTable;

pub(super) fn proves(column: &str, item: &SqlBoundItem, table: &CatalogTable) -> bool {
    proves_arm(column, item, table, false)
}

/// `arm_equated` is a raw equality, `IN`, or parameter on this `OR` arm only.
/// Raw equality proves `IS NOT NULL` for every catalog type. A strict-function
/// equality still has to pass [`text_family_column`].
pub(super) fn proves_arm(
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
