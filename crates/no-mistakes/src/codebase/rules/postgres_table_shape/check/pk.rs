use super::super::compile::{Shape, TriggerNeed};
use super::super::scan::{report, shape_text};
use super::keys::unqualified;
use crate::codebase::postgres::{
    CatalogObjectRef, CatalogTable, SchemaCatalog, TriggerEvent, TriggerTiming,
};
use crate::codebase::rules::RuleFinding;

pub(super) fn check_primary_key(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    table: &CatalogTable,
    shape: &Shape,
    catalog: &SchemaCatalog,
    object: &CatalogObjectRef,
) {
    let Some(columns) = &table.primary_key else {
        findings.push(report(
            path,
            object,
            &shape_text(shape, "table has no primary key"),
        ));
        return;
    };
    for name in columns {
        let data_type = table
            .columns
            .iter()
            .find(|column| column.name == *name)
            .map(|column| column.data_type.as_str())
            .unwrap_or("");
        if !type_allowed(data_type, &shape.primary_key_types, catalog) {
            findings.push(report(
                path,
                object,
                &shape_text(
                    shape,
                    &format!(
                        "primary key column {name} must be {}, found {data_type}",
                        primary_key_phrase(&shape.primary_key_types)
                    ),
                ),
            ));
        }
    }
}

fn type_allowed(data_type: &str, allowed: &[String], catalog: &SchemaCatalog) -> bool {
    allowed.iter().any(|kind| {
        if kind.eq_ignore_ascii_case("enum") {
            is_enum(catalog, data_type)
        } else {
            kind.eq_ignore_ascii_case(data_type)
        }
    })
}

fn is_enum(catalog: &SchemaCatalog, data_type: &str) -> bool {
    let name = unqualified(data_type);
    catalog.enums().any(|enum_type| {
        enum_type.name.eq_ignore_ascii_case(name) || enum_type.name.eq_ignore_ascii_case(data_type)
    })
}

fn primary_key_phrase(types: &[String]) -> String {
    types
        .iter()
        .map(|kind| {
            if kind.eq_ignore_ascii_case("enum") {
                "an enum".to_string()
            } else {
                kind.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" or ")
}

pub(super) fn trigger_label(trigger: &TriggerNeed) -> String {
    let timing = match trigger.timing {
        TriggerTiming::Before => "BEFORE",
        TriggerTiming::After => "AFTER",
        TriggerTiming::InsteadOf => "INSTEAD OF",
    };
    let events = trigger
        .events
        .iter()
        .map(|event| match event {
            TriggerEvent::Insert => "INSERT",
            TriggerEvent::Update => "UPDATE",
            TriggerEvent::Delete => "DELETE",
            TriggerEvent::Truncate => "TRUNCATE",
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let each = if trigger.for_each_row {
        "FOR EACH ROW"
    } else {
        "FOR EACH STATEMENT"
    };
    format!("{timing} {events} {each}")
}
