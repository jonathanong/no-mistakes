use super::{ExtraGeneratedColumn, RULE_ID};
use anyhow::{bail, Result};

pub(super) fn clauses(values: Option<&[String]>) -> Result<(bool, bool, bool)> {
    let Some(values) = values else {
        return Ok((true, true, true));
    };
    if values.is_empty() {
        bail!("{RULE_ID} option clauses: must not be empty");
    }
    let mut where_clause = false;
    let mut join_clause = false;
    let mut order_clause = false;
    let mut seen = Vec::new();
    for value in values {
        let key = value.trim().to_ascii_lowercase();
        if key.is_empty() {
            bail!("{RULE_ID} option clauses: empty clause name");
        }
        if seen.contains(&key) {
            bail!("{RULE_ID} option clauses: duplicate entry {key}");
        }
        seen.push(key.clone());
        match key.as_str() {
            "where" => where_clause = true,
            "join" => join_clause = true,
            "order-by" => order_clause = true,
            _ => bail!("{RULE_ID} option clauses: expected where, join, or order-by"),
        }
    }
    Ok((where_clause, join_clause, order_clause))
}

pub(super) fn extras(values: &[ExtraGeneratedColumn]) -> Result<Vec<ExtraGeneratedColumn>> {
    let mut seen = Vec::new();
    for extra in values {
        if extra.table.trim().is_empty() {
            bail!("{RULE_ID} option extraGeneratedColumns: empty table name");
        }
        if extra.column.trim().is_empty() {
            bail!("{RULE_ID} option extraGeneratedColumns: empty column name");
        }
        if extra.source_column.trim().is_empty() {
            bail!("{RULE_ID} option extraGeneratedColumns: empty sourceColumn");
        }
        let key = format!(
            "{}.{}",
            extra.table.trim().to_ascii_lowercase(),
            extra.column.trim().to_ascii_lowercase()
        );
        if seen.contains(&key) {
            bail!("{RULE_ID} option extraGeneratedColumns: duplicate entry {key}");
        }
        seen.push(key);
    }
    Ok(values.to_vec())
}

pub(super) fn unique_names(values: &[String], option: &str, noun: &str) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for value in values {
        let key = value.trim().to_ascii_lowercase();
        if key.is_empty() {
            bail!("{RULE_ID} option {option}: empty {noun} name");
        }
        if names.contains(&key) {
            bail!("{RULE_ID} option {option}: duplicate entry {key}");
        }
        names.push(key);
    }
    Ok(names)
}
