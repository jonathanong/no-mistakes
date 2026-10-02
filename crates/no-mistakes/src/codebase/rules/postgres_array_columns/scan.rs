use super::compile::Compiled;
use super::text::{excuse_text, never_text, ordinary_text};
use super::RULE_ID;
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let enums = enum_names(catalog);
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
                never_types.insert(object_ref, (object.clone(), element.to_string()));
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
    apply_message(compiled, &mut ordinary);
    apply_message(compiled, &mut never);
    super::super::sort_findings(&mut ordinary);
    let mut findings = compiled.allow.clone().apply(path, ordinary);
    let mut excused = Vec::new();
    findings.retain(|finding| {
        let Some(target) = finding.target.as_deref() else {
            return true;
        };
        let Some((object, element)) = never_types.get(target) else {
            return true;
        };
        excused.push(catalog_finding(
            RULE_ID,
            path,
            object,
            &excuse_text(&object.to_string(), element),
        ));
        false
    });
    findings.extend(never);
    findings.extend(excused);
    super::super::sort_findings(&mut findings);
    findings
}

fn skipped(element: &str, compiled: &Compiled, enums: &BTreeSet<String>) -> bool {
    if Compiled::listed(&compiled.allow_types, element) {
        return true;
    }
    if !compiled.allow_enums {
        return false;
    }
    let lower = element.to_ascii_lowercase();
    if lower.contains('.') {
        return enums.contains(&lower);
    }
    enums.contains(&lower) || enums.contains(unqualified(&lower))
}

fn enum_names(catalog: &SchemaCatalog) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for enum_type in catalog.enums() {
        let lower = enum_type.name.to_ascii_lowercase();
        names.insert(unqualified(&lower).to_string());
        names.insert(lower);
    }
    names
}

fn unqualified(name: &str) -> &str {
    name.rsplit_once('.').map(|(_, tail)| tail).unwrap_or(name)
}

fn apply_message(compiled: &Compiled, findings: &mut [RuleFinding]) {
    let Some(message) = compiled
        .message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
    else {
        return;
    };
    for finding in findings {
        let target = finding.target.as_deref().unwrap_or_default();
        finding.message = format!("{}: {target}: {message}", finding.file);
    }
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
