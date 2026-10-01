use super::compile::Compiled;
use super::pin::pinned_literals;
use super::text::{literal_text, name_text};
use super::RULE_ID;
use crate::codebase::postgres::{
    catalog_finding, CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;

struct Seen {
    table: String,
    column: String,
    data_type: String,
    candidate: bool,
    values: Vec<String>,
}

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let seen = collect(catalog, compiled);
    let mut findings = Vec::new();
    for item in &seen {
        if !item.candidate {
            continue;
        }
        let object = CatalogObjectRef::Column {
            table: item.table.clone(),
            column: item.column.clone(),
        };
        if item.values.is_empty() {
            if let Some(pattern) = compiled.matching_pattern(&item.column) {
                findings.push(catalog_finding(
                    RULE_ID,
                    path,
                    &object,
                    &name_text(&item.data_type, pattern),
                ));
            }
            continue;
        }
        findings.push(catalog_finding(
            RULE_ID,
            path,
            &object,
            &literal_text(&item.data_type, &item.values, &peers(&seen, item)),
        ));
    }
    super::super::sort_findings(&mut findings);
    let mut findings = compiled.allow.clone().apply(path, findings);
    super::super::sort_findings(&mut findings);
    findings
}

fn collect(catalog: &SchemaCatalog, compiled: &Compiled) -> Vec<Seen> {
    let mut seen = Vec::new();
    for table in catalog.tables() {
        if compiled.ignores(&table.name) {
            continue;
        }
        for column in &table.columns {
            if !compiled.type_matches(&column.data_type) {
                continue;
            }
            seen.push(Seen {
                table: table.name.clone(),
                column: column.name.clone(),
                data_type: column.data_type.clone(),
                candidate: candidate(column, table, compiled),
                values: values_for(table, &column.name),
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

fn values_for(table: &CatalogTable, column: &str) -> Vec<String> {
    let mut checks: Vec<_> = table.check_constraints.iter().collect();
    checks.sort_by(|left, right| left.name.cmp(&right.name));
    let mut values = Vec::new();
    for check in checks {
        let Some(pinned) = pinned_literals(&check.definition, column) else {
            continue;
        };
        for value in pinned {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        }
    }
    values
}

fn peers(seen: &[Seen], item: &Seen) -> Vec<String> {
    let mut peers: Vec<(String, String)> = seen
        .iter()
        .filter(|other| other.table != item.table || other.column != item.column)
        .filter(|other| !other.values.is_empty() && same_set(&other.values, &item.values))
        .map(|other| {
            (
                format!("column:{}.{}", other.table, other.column),
                format!("{}.{}", other.table, other.column),
            )
        })
        .collect();
    peers.sort();
    peers.into_iter().map(|(_, display)| display).collect()
}

fn same_set(left: &[String], right: &[String]) -> bool {
    let mut left: Vec<&str> = left.iter().map(String::as_str).collect();
    let mut right: Vec<&str> = right.iter().map(String::as_str).collect();
    left.sort_unstable();
    right.sort_unstable();
    left == right
}
