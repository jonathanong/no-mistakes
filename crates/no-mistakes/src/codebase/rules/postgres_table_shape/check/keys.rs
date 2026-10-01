use super::super::compile::{RequiredColumn, Shape};
use super::super::scan::{report, shape_text};
use crate::codebase::postgres::{CatalogForeignKey, CatalogObjectRef, CatalogTable};
use crate::codebase::rules::RuleFinding;

pub(super) fn push_foreign_key(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    table: &CatalogTable,
    object: &CatalogObjectRef,
    shape: &Shape,
    name: &str,
    required: &RequiredColumn,
) {
    if !required.foreign_key {
        return;
    }
    let keys = sole_keys(table, name);
    if keys.is_empty() {
        findings.push(report(
            path,
            object,
            &shape_text(shape, &format!("column {name} must be a foreign key")),
        ));
        return;
    }
    if keys.iter().any(|key| foreign_key_ok(key, required)) {
        return;
    }
    let first = keys[0];
    if let Some(expected) = &required.on_delete {
        if !first.on_delete.eq_ignore_ascii_case(expected) {
            findings.push(report(
                path,
                object,
                &shape_text(
                    shape,
                    &format!(
                        "foreign key on {name} must be ON DELETE {}, found {}",
                        display_action(expected),
                        display_action(&first.on_delete)
                    ),
                ),
            ));
        }
    }
    if let Some(tables) = &required.references {
        if !references_ok(first, tables) {
            findings.push(report(
                path,
                object,
                &shape_text(
                    shape,
                    &format!(
                        "foreign key on {name} must reference one of {}, found {}",
                        tables.join(", "),
                        unqualified(&first.referenced_table)
                    ),
                ),
            ));
        }
    }
}

pub(super) fn sole_keys<'a>(table: &'a CatalogTable, column: &str) -> Vec<&'a CatalogForeignKey> {
    table
        .foreign_keys
        .iter()
        .filter(|key| {
            key.columns.len() == 1 && key.columns.first().map(String::as_str) == Some(column)
        })
        .collect()
}

pub(super) fn foreign_key_ok(key: &CatalogForeignKey, required: &RequiredColumn) -> bool {
    required
        .on_delete
        .as_ref()
        .is_none_or(|expected| key.on_delete.eq_ignore_ascii_case(expected))
        && required
            .references
            .as_ref()
            .is_none_or(|tables| references_ok(key, tables))
}

fn references_ok(key: &CatalogForeignKey, tables: &[String]) -> bool {
    let found = unqualified(&key.referenced_table);
    tables
        .iter()
        .any(|table| unqualified(table).eq_ignore_ascii_case(found))
}

pub(super) fn pattern_suffix(required: &RequiredColumn) -> String {
    let mut parts = Vec::new();
    if let Some(data_type) = &required.data_type {
        parts.push(format!("type {data_type}"));
    }
    if required.nullable == Some(false) {
        parts.push("NOT NULL".to_string());
    }
    if required.foreign_key {
        parts.push("a foreign key".to_string());
    }
    if let Some(tables) = &required.references {
        parts.push(format!("references {}", tables.join(", ")));
    }
    if let Some(action) = &required.on_delete {
        parts.push(format!("ON DELETE {}", display_action(action)));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" with {}", parts.join(", "))
    }
}

fn display_action(action: &str) -> String {
    action.to_ascii_uppercase()
}

pub(super) fn unqualified(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name).trim_matches('"')
}
