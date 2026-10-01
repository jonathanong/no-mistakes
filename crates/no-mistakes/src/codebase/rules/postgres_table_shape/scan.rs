use super::compile::{Banned, Shape};
use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        let object = CatalogObjectRef::Table(table.name.clone());
        for shape in &compiled.shapes {
            if shape.table_pattern.is_match(&table.name) {
                super::check::check_shape(
                    &mut findings,
                    &compiled.schema_catalog_path,
                    table,
                    shape,
                    catalog,
                );
            }
        }
        for banned in &compiled.banned {
            if banned.pattern.is_match(&table.name) {
                findings.push(report(
                    &compiled.schema_catalog_path,
                    &object,
                    &banned_text(banned),
                ));
            }
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

pub(super) fn report(path: &str, object: &CatalogObjectRef, text: &str) -> RuleFinding {
    catalog_finding(RULE_ID, path, object, text)
}

fn banned_text(banned: &Banned) -> String {
    format!(
        "table name matches banned pattern {}: {}",
        banned.source, banned.message
    )
}

pub(super) fn shape_text(shape: &Shape, text: &str) -> String {
    format!("(shape {}) {text}", shape.name)
}
