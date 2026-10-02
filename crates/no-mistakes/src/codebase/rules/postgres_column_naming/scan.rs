use super::compile::{lists_type, Compiled};
use super::foreign::foreign_key_texts;
use super::require::{require_text, stale_exempt};
use super::reserved::reserved_texts;
use super::singular::join_or;
use super::RULE_ID;
use crate::codebase::postgres::{
    catalog_finding, CatalogColumn, CatalogObjectRef, CatalogTable, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        if ignored(&table.name, compiled) {
            continue;
        }
        for column in &table.columns {
            if compiled.skip_generated && column.generated.is_some() {
                continue;
            }
            let object = CatalogObjectRef::Column {
                table: table.name.clone(),
                column: column.name.clone(),
            };
            for text in column_texts(column, table, catalog, compiled) {
                findings.push(catalog_finding(RULE_ID, path, &object, &text));
            }
        }
    }
    super::super::sort_findings(&mut findings);
    apply_message(compiled, &mut findings);
    let mut findings = compiled.allow.clone().apply(path, findings);
    findings.extend(stale_exempt(catalog, compiled, path));
    super::super::sort_findings(&mut findings);
    findings
}

fn column_texts(
    column: &CatalogColumn,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    compiled: &Compiled,
) -> Vec<String> {
    let mut texts = Vec::new();
    texts.extend(type_texts(column, compiled));
    texts.extend(name_type_texts(column, compiled));
    if let Some(text) = forbidden_text(column, compiled) {
        texts.push(text);
    }
    texts.extend(foreign_key_texts(column, table, compiled));
    let reserved = reserved_texts(column, table, catalog, compiled);
    let reserved_hit = !reserved.is_empty();
    texts.extend(reserved);
    if !reserved_hit {
        texts.extend(require_text(column, table, compiled));
    }
    texts
}

fn type_texts(column: &CatalogColumn, compiled: &Compiled) -> Vec<String> {
    compiled
        .type_rules
        .iter()
        .filter(|rule| {
            lists_type(&rule.types, &column.data_type) && !rule.pattern.is_match(&column.name)
        })
        .map(|rule| {
            let mut text = format!(
                "{} column name does not match {}",
                column.data_type, rule.raw
            );
            if let Some(hint) = &rule.hint {
                text.push_str(&format!(" ({hint})"));
            }
            text
        })
        .collect()
}

fn name_type_texts(column: &CatalogColumn, compiled: &Compiled) -> Vec<String> {
    compiled
        .name_type_rules
        .iter()
        .filter(|rule| {
            rule.pattern.is_match(&column.name) && !lists_type(&rule.types, &column.data_type)
        })
        .map(|rule| {
            let mut text = format!(
                "column name matches {} but its type is {}; use {}",
                rule.raw,
                column.data_type,
                join_or(&rule.types)
            );
            if let Some(hint) = &rule.hint {
                text.push_str(&format!(" ({hint})"));
            }
            text
        })
        .collect()
}

fn forbidden_text(column: &CatalogColumn, compiled: &Compiled) -> Option<String> {
    compiled
        .forbidden
        .iter()
        .find(|rule| rule.pattern.is_match(&column.name))
        .map(|rule| {
            format!(
                "column name matches forbidden pattern {}; {}",
                rule.raw, rule.hint
            )
        })
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

fn ignored(name: &str, compiled: &Compiled) -> bool {
    compiled.ignore.iter().any(|pattern| pattern.is_match(name))
}
