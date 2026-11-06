use super::compile::Compiled;
use super::pin::pinned_literals_expr;
use super::scan::Seen;
use crate::codebase::postgres::{CatalogColumn, CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;

pub(super) fn collect(catalog: &SchemaCatalog, compiled: &Compiled) -> Vec<Seen> {
    let mut seen = Vec::new();
    for table in catalog.logical_tables() {
        let mut checks: Vec<_> = table
            .check_constraints
            .iter()
            .filter(|check| check.validated)
            .collect();
        checks.sort_by(|left, right| left.name.cmp(&right.name));
        let parsed_checks: Vec<_> = checks
            .into_iter()
            .filter_map(|check| super::parse::expression(&check.definition))
            .collect();
        for column in &table.columns {
            if !compiled.type_matches(&column.data_type) {
                continue;
            }
            seen.push(Seen {
                table: table.name.clone(),
                column: column.name.clone(),
                data_type: column.data_type.clone(),
                candidate: candidate(column, table, compiled),
                ignored: compiled.ignores(&table.name),
                values: values_for(&parsed_checks, &column.name),
            });
        }
    }
    seen
}

fn candidate(column: &CatalogColumn, table: &CatalogTable, compiled: &Compiled) -> bool {
    if compiled.skip_generated && column.generated.is_some() {
        return false;
    }
    !sole_foreign_key(table, &column.name)
}

fn sole_foreign_key(table: &CatalogTable, column: &str) -> bool {
    table
        .foreign_keys
        .iter()
        .any(|foreign_key| foreign_key.columns.len() == 1 && foreign_key.columns[0] == column)
}

fn values_for(checks: &[Expr], column: &str) -> Vec<String> {
    let mut pinned_checks = Vec::new();
    for check in checks {
        let Some(values) = pinned_literals_expr(check, column) else {
            continue;
        };
        pinned_checks.push(values);
    }
    let Some(mut values) = pinned_checks.first().cloned() else {
        return Vec::new();
    };
    for check_values in pinned_checks.iter().skip(1) {
        values.retain(|value| check_values.contains(value));
    }
    values
}
