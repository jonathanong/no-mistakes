use super::compile::Compiled;
use super::pin::pinned_literals_expr;
use super::text::{literal_text, name_text};
use super::RULE_ID;
use crate::codebase::postgres::{
    catalog_finding, CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;
use sqlparser::ast::Expr;
use std::collections::BTreeMap;

struct Seen {
    table: String,
    column: String,
    data_type: String,
    candidate: bool,
    values: Vec<String>,
}

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let seen = collect(catalog, compiled);
    let peers = peer_index(&seen);
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
                findings.push(column_finding(
                    catalog,
                    RULE_ID,
                    path,
                    &object,
                    &item.table,
                    &item.column,
                    &name_text(&item.data_type, pattern),
                ));
            }
            continue;
        }
        findings.push(column_finding(
            catalog,
            RULE_ID,
            path,
            &object,
            &item.table,
            &item.column,
            &literal_text(&item.data_type, &item.values, &peers_for(&peers, item)),
        ));
    }
    super::super::sort_findings(&mut findings);
    if let Some(message) = compiled
        .message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
    {
        for finding in &mut findings {
            finding.message = format!(
                "{}: {}: {message}",
                finding.file,
                finding.target.as_deref().unwrap_or_default()
            );
        }
    }
    let mut findings = compiled.allow.clone().apply(path, findings);
    super::super::sort_findings(&mut findings);
    findings
}

fn column_finding(
    catalog: &SchemaCatalog,
    rule: &str,
    path: &str,
    object: &CatalogObjectRef,
    table: &str,
    column: &str,
    text: &str,
) -> RuleFinding {
    let mut finding = catalog_finding(rule, path, object, text);
    finding.line = catalog.column_line(table, column);
    finding
}

fn collect(catalog: &SchemaCatalog, compiled: &Compiled) -> Vec<Seen> {
    let mut seen = Vec::new();
    for table in catalog.logical_tables() {
        if compiled.ignores(&table.name) {
            continue;
        }
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

type PeerIndex = BTreeMap<Vec<String>, Vec<(String, String)>>;

fn peer_index(seen: &[Seen]) -> PeerIndex {
    let mut index: PeerIndex = BTreeMap::new();
    for item in seen
        .iter()
        .filter(|item| item.candidate && !item.values.is_empty())
    {
        let mut key = item.values.clone();
        key.sort();
        key.dedup();
        index.entry(key).or_default().push((
            format!("column:{}.{}", item.table, item.column),
            format!("{}.{}", item.table, item.column),
        ));
    }
    for peers in index.values_mut() {
        peers.sort();
    }
    index
}

fn peers_for(index: &PeerIndex, item: &Seen) -> Vec<String> {
    let mut key = item.values.clone();
    key.sort();
    key.dedup();
    index
        .get(&key)
        .into_iter()
        .flatten()
        .filter(|(sort_key, _)| sort_key != &format!("column:{}.{}", item.table, item.column))
        .map(|(_, display)| display.clone())
        .collect()
}
