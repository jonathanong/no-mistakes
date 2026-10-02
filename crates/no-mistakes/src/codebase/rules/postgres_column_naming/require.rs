use super::compile::{lists_type, Compiled};
use crate::codebase::postgres::{CatalogColumn, CatalogTable, SchemaCatalog};
use crate::codebase::rules::RuleFinding;

pub(super) fn require_text(
    column: &CatalogColumn,
    table: &CatalogTable,
    compiled: &Compiled,
) -> Option<String> {
    let require = compiled.require.as_ref()?;
    if column.generated.is_some() || !lists_type(&require.types, &column.data_type) {
        return None;
    }
    if !require.pattern.is_match(&column.name) {
        return None;
    }
    if require
        .exempt
        .iter()
        .any(|entry| entry.pattern.is_match(&column.name))
    {
        return None;
    }
    let referenced = table
        .foreign_keys
        .iter()
        .any(|foreign_key| foreign_key.columns.iter().any(|name| name == &column.name));
    if referenced {
        return None;
    }
    Some(format!(
        "{} column name matches {} but the column is not part of any foreign key; add a foreign key, or an exempt pattern or allow entry with a reason",
        column.data_type, require.raw
    ))
}

pub(super) fn stale_exempt(
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    path: &str,
) -> Vec<RuleFinding> {
    let Some(require) = &compiled.require else {
        return Vec::new();
    };
    let path = path.replace('\\', "/");
    require
        .exempt
        .iter()
        .filter(|entry| !exempt_matches(catalog, compiled, require, &entry.pattern))
        .map(|entry| RuleFinding {
            rule: super::RULE_ID.to_string(),
            file: path.clone(),
            line: 1,
            message: format!(
                "{path}: stale postgres-column-naming requireForeignKey exempt entry: {}",
                entry.raw
            ),
            import: None,
            target: Some(entry.raw.clone()),
        })
        .collect()
}

fn exempt_matches(
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    require: &super::compile::RequireCheck,
    pattern: &regex::Regex,
) -> bool {
    catalog.tables().any(|table| {
        !ignored(&table.name, compiled)
            && table.columns.iter().any(|column| {
                column.generated.is_none()
                    && lists_type(&require.types, &column.data_type)
                    && require.pattern.is_match(&column.name)
                    && pattern.is_match(&column.name)
                    && !in_foreign_key(table, &column.name)
            })
    })
}

fn in_foreign_key(table: &CatalogTable, column: &str) -> bool {
    table
        .foreign_keys
        .iter()
        .any(|foreign_key| foreign_key.columns.iter().any(|name| name == column))
}

fn ignored(name: &str, compiled: &Compiled) -> bool {
    compiled.ignore.iter().any(|pattern| pattern.is_match(name))
}
