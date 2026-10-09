use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{
    catalog_finding, CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;

const ADVICE: &str = "keep one source of truth: make status a GENERATED ALWAYS AS (...) STORED column computed from the timestamps, or, if rows return to earlier states, record each attempt or change in a history table";

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    if compiled.lifecycle_verbs.is_empty() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for table in catalog.logical_tables() {
        if let Some(finding) = table_finding(&compiled, table) {
            findings.push(finding);
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
    let data_type = data_type.trim().to_ascii_lowercase();
    if is_array(&data_type) {
        return false;
    }
    let rest = if let Some(rest) = data_type.strip_prefix("timestamptz") {
        rest
    } else if let Some(rest) = data_type.strip_prefix("timestamp") {
        rest
    } else {
        return false;
    };
    let rest = match precision(rest.trim_start()) {
        Some(after) => after.trim_start(),
        None => rest.trim_start(),
    };
    rest.is_empty() || rest == "with time zone" || rest == "without time zone"
}

fn precision(rest: &str) -> Option<&str> {
    let rest = rest.strip_prefix('(')?;
    let (digits, after) = rest.split_once(')')?;
    digits
        .chars()
        .all(|character| character.is_ascii_digit())
        .then_some(after)
        .filter(|_| !digits.is_empty())
}

fn is_array(data_type: &str) -> bool {
    data_type.contains('[')
}
