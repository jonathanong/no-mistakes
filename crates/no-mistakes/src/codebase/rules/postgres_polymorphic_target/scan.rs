use super::{compile::Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, CatalogTable, SchemaCatalog};
use crate::codebase::rules::allow_audit::AllowUsage;
use no_mistakes_rule_kernel::finding::RuleFinding;
use rayon::prelude::*;
use std::collections::BTreeSet;

const FIX_ADVICE: &str = "use one nullable foreign key per target table plus a CHECK that exactly one is set, or a separate table per target";

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog, path: &str) -> Vec<RuleFinding> {
    let tables: Vec<_> = catalog.logical_tables().collect();
    let allow_objects: Vec<&str> = compiled
        .allow
        .entry_objects()
        .map(|(_, object)| object)
        .collect();
    let used = AllowUsage::new(allow_objects.len());
    let mut findings: Vec<RuleFinding> = tables
        .par_iter()
        .flat_map_iter(|table| table_findings(&compiled, catalog, table, path, &allow_objects, &used))
        .collect();
    let normalized_path = path.replace('\\', "/");
    for index in used.unused() {
        let object = allow_objects[index];
        findings.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: normalized_path.clone(),
            line: 1,
            message: format!("{normalized_path}: stale {RULE_ID} allow entry: {object}"),
            import: None,
            target: Some(object.to_string()),
        });
    }
    super::super::sort_findings(&mut findings);
    findings
}

fn table_findings(
    compiled: &Compiled,
    catalog: &SchemaCatalog,
    table: &CatalogTable,
    path: &str,
    allow_objects: &[&str],
    used: &AllowUsage,
) -> Vec<RuleFinding> {
    let column_names: BTreeSet<&str> = table
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect();
    let fk_columns: BTreeSet<&str> = table
        .foreign_keys
        .iter()
        .flat_map(|key| key.columns.iter().map(String::as_str))
        .collect();
    let mut findings = Vec::new();
    for column in &table.columns {
        let Some(prefix) = column.name.strip_suffix(&compiled.type_suffix) else {
            continue;
        };
        if prefix.is_empty() {
            continue;
        }
        let id_name = format!("{prefix}{}", compiled.id_suffix);
        if !column_names.contains(id_name.as_str()) || fk_columns.contains(id_name.as_str()) {
            continue;
        }
        let id_ref = CatalogObjectRef::Column {
            table: table.name.clone(),
            column: id_name.clone(),
        }
        .to_string();
        let type_ref = CatalogObjectRef::Column {
            table: table.name.clone(),
            column: column.name.clone(),
        }
        .to_string();
        if used.suppress(
            allow_objects
                .iter()
                .map(|object| *object == id_ref || *object == type_ref),
        ) {
            continue;
        }
        let object = CatalogObjectRef::Table(table.name.clone());
        let text = match &compiled.message {
            Some(custom) => format!("columns {} and {}: {custom}", column.name, id_name),
            None => format!(
                "columns {} and {} form a polymorphic target without a foreign key on {}; {FIX_ADVICE}",
                column.name, id_name, id_name
            ),
        };
        let mut finding = catalog_finding(RULE_ID, path, &object, &text);
        finding.line = catalog.table_line(&table.name);
        findings.push(finding);
    }
    findings
}
