mod keys;
mod pk;

use super::compile::{RequiredColumn, Shape};
use super::scan::{report, shape_text};
use crate::codebase::postgres::{CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog};
use crate::codebase::rules::RuleFinding;

pub(super) fn check_shape(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    table: &CatalogTable,
    shape: &Shape,
    catalog: &SchemaCatalog,
) {
    let object = CatalogObjectRef::Table(table.name.clone());
    for column in &shape.columns {
        if column.name.is_some() {
            check_named(findings, path, table, shape, &object, column);
        } else {
            check_pattern(findings, path, table, shape, &object, column);
        }
    }
    for forbidden in &shape.forbidden {
        if table.columns.iter().any(|column| column.name == *forbidden) {
            findings.push(report(
                path,
                &object,
                &shape_text(shape, &format!("forbidden column {forbidden}")),
            ));
        }
    }
    for trigger in &shape.triggers {
        if !table.triggers.iter().any(|existing| {
            existing.matches(
                &trigger.function,
                trigger.timing,
                &trigger.events,
                trigger.for_each_row,
            )
        }) {
            findings.push(report(
                path,
                &object,
                &shape_text(
                    shape,
                    &format!(
                        "no {} trigger executing {}()",
                        pk::trigger_label(trigger),
                        trigger.function
                    ),
                ),
            ));
        }
    }
    if !shape.primary_key_types.is_empty() {
        pk::check_primary_key(findings, path, table, shape, catalog, &object);
    }
}

fn check_named(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    table: &CatalogTable,
    shape: &Shape,
    object: &CatalogObjectRef,
    required: &RequiredColumn,
) {
    let name = required.name.as_deref().unwrap_or_default();
    let Some(column) = table.columns.iter().find(|column| column.name == name) else {
        findings.push(report(
            path,
            object,
            &shape_text(shape, &format!("missing required column {name}")),
        ));
        return;
    };
    if let Some(expected) = &required.data_type {
        if !column.data_type.eq_ignore_ascii_case(expected) {
            findings.push(report(
                path,
                object,
                &shape_text(
                    shape,
                    &format!(
                        "column {name} must be {expected}, found {}",
                        column.data_type
                    ),
                ),
            ));
        }
    }
    push_nullable(findings, path, object, shape, name, required, column);
    keys::push_foreign_key(findings, path, table, object, shape, name, required);
}

fn push_nullable(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    object: &CatalogObjectRef,
    shape: &Shape,
    name: &str,
    required: &RequiredColumn,
    column: &CatalogColumn,
) {
    match required.nullable {
        Some(false) if column.nullable => findings.push(report(
            path,
            object,
            &shape_text(shape, &format!("column {name} must be NOT NULL")),
        )),
        Some(true) if !column.nullable => findings.push(report(
            path,
            object,
            &shape_text(shape, &format!("column {name} must be nullable")),
        )),
        _ => {}
    }
}

fn check_pattern(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    table: &CatalogTable,
    shape: &Shape,
    object: &CatalogObjectRef,
    required: &RequiredColumn,
) {
    let Some(pattern) = &required.name_pattern else {
        return;
    };
    let matched = table.columns.iter().any(|column| {
        pattern.is_match(&column.name) && column_properties_match(table, column, required)
    });
    if !matched {
        findings.push(report(
            path,
            object,
            &shape_text(
                shape,
                &format!(
                    "no column matches {}{}",
                    required.pattern_source,
                    keys::pattern_suffix(required)
                ),
            ),
        ));
    }
}

fn column_properties_match(
    table: &CatalogTable,
    column: &CatalogColumn,
    required: &RequiredColumn,
) -> bool {
    if required
        .data_type
        .as_ref()
        .is_some_and(|expected| !column.data_type.eq_ignore_ascii_case(expected))
    {
        return false;
    }
    if required
        .nullable
        .is_some_and(|nullable| column.nullable != nullable)
    {
        return false;
    }
    if !required.foreign_key {
        return true;
    }
    keys::sole_keys(table, &column.name)
        .into_iter()
        .any(|key| keys::foreign_key_ok(key, required))
}
