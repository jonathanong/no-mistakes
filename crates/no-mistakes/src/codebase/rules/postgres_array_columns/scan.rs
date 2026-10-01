use super::compile::Compiled;
use super::text::{excuse_text, never_text, ordinary_text};
use super::RULE_ID;
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let enums: Vec<&str> = catalog
        .enums()
        .map(|enum_type| enum_type.name.as_str())
        .collect();
    let mut ordinary = Vec::new();
    let mut never = Vec::new();
    let mut never_types = BTreeMap::new();
    for table in catalog.tables() {
        for column in &table.columns {
            let Some(element) = element_type(&column.data_type) else {
                continue;
            };
            if skipped(element, compiled, &enums) {
                continue;
            }
            let object = CatalogObjectRef::Column {
                table: table.name.clone(),
                column: column.name.clone(),
            };
            let object_ref = object.to_string();
            if Compiled::listed(&compiled.never, element) {
                never_types.insert(object_ref, element.to_string());
                never.push(catalog_finding(
                    RULE_ID,
                    path,
                    &object,
                    &never_text(element),
                ));
            } else {
                ordinary.push(catalog_finding(
                    RULE_ID,
                    path,
                    &object,
                    &ordinary_text(&column.data_type),
                ));
            }
        }
    }
    super::super::sort_findings(&mut ordinary);
    let mut findings = compiled.allow.clone().apply(path, ordinary);
    let mut excused = Vec::new();
    findings.retain(|finding| {
        if !finding.message.contains(": stale ") {
            return true;
        }
        let Some(object) = finding.target.as_deref() else {
            return true;
        };
        let Some(element) = never_types.get(object) else {
            return true;
        };
        let object_ref = CatalogObjectRef::Column {
            table: table_of(object),
            column: column_of(object),
        };
        excused.push(catalog_finding(
            RULE_ID,
            path,
            &object_ref,
            &excuse_text(object, element),
        ));
        false
    });
    findings.extend(never);
    findings.extend(excused);
    super::super::sort_findings(&mut findings);
    findings
}

fn skipped(element: &str, compiled: &Compiled, enums: &[&str]) -> bool {
    Compiled::listed(&compiled.allow_types, element)
        || (compiled.allow_enums && enums.iter().any(|name| name.eq_ignore_ascii_case(element)))
}

fn element_type(data_type: &str) -> Option<&str> {
    if !data_type.ends_with("[]") {
        return None;
    }
    let mut end = data_type.len();
    while end >= 2 && data_type[..end].ends_with("[]") {
        end -= 2;
    }
    Some(&data_type[..end])
}

fn table_of(object: &str) -> String {
    object
        .strip_prefix("column:")
        .and_then(|rest| rest.rsplit_once('.'))
        .map(|(table, _)| table.to_string())
        .unwrap_or_default()
}

fn column_of(object: &str) -> String {
    object
        .rsplit_once('.')
        .map(|(_, column)| column.to_string())
        .unwrap_or_default()
}
