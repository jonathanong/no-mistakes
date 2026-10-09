use super::compile::Compiled;
use super::RULE_ID;
use crate::codebase::postgres::{
    catalog_finding, CatalogObjectRef, CatalogTable, RelationKind, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;
use anyhow::{bail, Result};

pub(super) fn scan(
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    path: &str,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        if table.partition_of.is_some() && table.relation_kind != RelationKind::PartitionedTable {
            continue;
        }
        if compiled.check_primary_keys {
            if let Some(columns) = &table.primary_key {
                let Some(name) = table
                    .indexes
                    .iter()
                    .find(|index| index.primary)
                    .map(|index| index.name.as_str())
                else {
                    bail!("schemaCatalogPath {path}: table {} has a primary key but no primary index name; regenerate the schema catalog", table.name);
                };
                add_key_finding(
                    catalog,
                    compiled,
                    path,
                    table,
                    columns,
                    KeyKind::Primary(name),
                    &mut findings,
                );
            }
        }
        if compiled.check_foreign_keys {
            for key in &table.foreign_keys {
                add_key_finding(
                    catalog,
                    compiled,
                    path,
                    table,
                    &key.columns,
                    KeyKind::Foreign {
                        name: &key.name,
                        referenced_table: &key.referenced_table,
                    },
                    &mut findings,
                );
            }
        }
    }
    if let Some(message) = compiled
        .message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
    {
        for finding in &mut findings {
            finding.message = format!(
                "{}: {}: {message}",
                finding.file,
                finding.target.as_deref().unwrap_or_default()
            );
        }
    }
    super::super::sort_findings(&mut findings);
    let findings = compiled.allow.clone().apply(path, findings);
    let mut findings = findings;
    super::super::sort_findings(&mut findings);
    Ok(findings)
}

enum KeyKind<'a> {
    Primary(&'a str),
    Foreign {
        name: &'a str,
        referenced_table: &'a str,
    },
}

fn add_key_finding(
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    path: &str,
    table: &CatalogTable,
    names: &[String],
    key_kind: KeyKind<'_>,
    findings: &mut Vec<RuleFinding>,
) {
    let failures = names
        .iter()
        .filter_map(|name| {
            let column = table.columns.iter().find(|column| column.name == *name);
            if column.is_some_and(|column| allowed(catalog, compiled, &column.data_type)) {
                None
            } else {
                Some((name.as_str(), column))
            }
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        return;
    }
    let constraint_name = match key_kind {
        KeyKind::Primary(name) => name,
        KeyKind::Foreign { name, .. } => name,
    };
    let object = CatalogObjectRef::Constraint {
        table: table.name.clone(),
        name: constraint_name.to_string(),
    };
    let descriptions = failures
        .iter()
        .map(|(name, column)| {
            format!(
                "{} column {name}",
                column
                    .map(|column| display_type(&column.data_type))
                    .unwrap_or_else(|| "unknown".into())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let text = match key_kind {
        KeyKind::Primary(_) => format!("primary key uses {descriptions}; use one of {}, and keep any natural key as a separate unique constraint", allowed_description(compiled)),
        KeyKind::Foreign { referenced_table, .. } => format!("foreign key uses {descriptions} (references {referenced_table}); review the referenced key's type and use a compatible configured type"),
    };
    findings.push(catalog_finding(RULE_ID, path, &object, &text));
}

fn allowed(catalog: &SchemaCatalog, compiled: &Compiled, data_type: &str) -> bool {
    compiled
        .allowed_types
        .iter()
        .any(|value| value.eq_ignore_ascii_case(data_type))
        || (compiled.allow_enum_types && catalog.enum_type(data_type).is_some())
}

fn allowed_description(compiled: &Compiled) -> String {
    let mut types = compiled.allowed_types.clone();
    if compiled.allow_enum_types {
        types.push("an enum".into());
    }
    match types.as_slice() {
        [only] => only.clone(),
        [first, second] => format!("{first} or {second}"),
        _ => format!(
            "{}, or {}",
            types[..types.len() - 1].join(", "),
            types.last().unwrap()
        ),
    }
}

fn display_type(data_type: &str) -> String {
    let Some((base, suffix)) = data_type.rsplit_once('(') else {
        return data_type.to_string();
    };
    let Some(modifier) = suffix.strip_suffix(')') else {
        return data_type.to_string();
    };
    if !modifier
        .chars()
        .all(|character| character.is_ascii_digit() || matches!(character, ',' | ' '))
    {
        return data_type.to_string();
    }
    base.to_string()
}
