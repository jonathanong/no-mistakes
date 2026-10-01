use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{
    catalog_finding, CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;

const ADVICE: &str = "keep one source of truth: make status a GENERATED ALWAYS AS (...) STORED column computed from the timestamps, or, if rows return to earlier states, record each attempt or change in a history table";

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if !compiled.lifecycle_verbs.is_empty() {
        for table in catalog.tables() {
            if let Some(finding) = table_finding(&compiled, table) {
                findings.push(finding);
            }
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn table_finding(compiled: &Compiled, table: &CatalogTable) -> Option<RuleFinding> {
    let status = stored_status(table, &compiled.status_columns)?;
    let timestamps = lifecycle_names(table, &compiled.lifecycle_verbs);
    if timestamps.len() < compiled.min_lifecycle_columns {
        return None;
    }
    let text = compiled
        .message
        .clone()
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "table stores status column {} next to lifecycle timestamps {}; {ADVICE}",
                status.name,
                timestamps.join(", ")
            )
        });
    Some(catalog_finding(
        RULE_ID,
        &compiled.schema_catalog_path,
        &CatalogObjectRef::Table(table.name.clone()),
        &text,
    ))
}

fn stored_status<'a>(table: &'a CatalogTable, names: &[String]) -> Option<&'a CatalogColumn> {
    table.columns.iter().find(|column| {
        column.generated.is_none()
            && !is_array(&column.data_type)
            && names.iter().any(|name| name == &column.name)
    })
}

fn lifecycle_names(table: &CatalogTable, verbs: &[String]) -> Vec<String> {
    table
        .columns
        .iter()
        .filter(|column| is_lifecycle(column, verbs))
        .map(|column| column.name.clone())
        .collect()
}

fn is_lifecycle(column: &CatalogColumn, verbs: &[String]) -> bool {
    if column.generated.is_some() || !is_scalar_timestamp(&column.data_type) {
        return false;
    }
    verbs.iter().any(|verb| {
        let exact = format!("{verb}_at");
        column.name == exact || column.name.ends_with(&format!("_{verb}_at"))
    })
}

fn is_scalar_timestamp(data_type: &str) -> bool {
    let data_type = data_type.to_ascii_lowercase();
    !is_array(&data_type)
        && (data_type.starts_with("timestamp") || data_type.starts_with("timestamptz"))
}

fn is_array(data_type: &str) -> bool {
    data_type.contains('[')
}
