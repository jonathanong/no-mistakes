use super::{
    compile::{integer_type, Compiled},
    proof, RULE_ID,
};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, CatalogTable, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use rayon::prelude::*;
use std::collections::BTreeSet;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog, path: &str) -> Vec<RuleFinding> {
    let tables: Vec<_> = catalog.logical_tables().collect();
    let findings = tables
        .par_iter()
        .flat_map_iter(|table| table_findings(&compiled, catalog, table, path))
        .collect();
    let mut findings = compiled.allow.apply(path, findings);
    super::super::sort_findings(&mut findings);
    findings
}

fn table_findings(
    compiled: &Compiled,
    catalog: &SchemaCatalog,
    table: &CatalogTable,
    path: &str,
) -> Vec<RuleFinding> {
    let selected: Vec<_> = table
        .columns
        .iter()
        .filter(|column| {
            integer_type(&column.data_type).is_some_and(|data_type| {
                compiled.requirements.iter().any(|requirement| {
                    requirement.types.contains(&data_type)
                        && requirement.pattern.is_match(&column.name)
                })
            })
        })
        .collect();
    if selected.is_empty() {
        return Vec::new();
    }
    let proven: BTreeSet<_> = table
        .check_constraints
        .iter()
        .filter(|check| check.validated)
        .flat_map(|check| proof::proven_columns(&check.definition))
        .collect();
    selected.into_iter().filter(|column| !proven.contains(&column.name)).map(|column| {
        let object = CatalogObjectRef::Column { table: table.name.clone(), column: column.name.clone() };
        let sql_column = sqlparser::ast::Ident::with_quote('"', column.name.as_str());
        let default = format!("{}.{} lacks a validated CHECK proving {} >= 0; add and validate CHECK ({sql_column} >= 0) to enforce non-negative values", table.name, column.name, column.name);
        let mut finding = catalog_finding(RULE_ID, path, &object, compiled.message.as_deref().unwrap_or(&default));
        finding.line = catalog.column_line(&table.name, &column.name);
        finding
    }).collect()
}
