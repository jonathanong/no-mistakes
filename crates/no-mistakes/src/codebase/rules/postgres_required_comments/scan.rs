use super::{Compiled, ObjectKind, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let tables = compiled.objects.contains(&ObjectKind::Table);
    let columns = compiled.objects.contains(&ObjectKind::Column);
    if tables || columns {
        for table in catalog.tables() {
            if tables {
                push(
                    &mut findings,
                    &compiled,
                    CatalogObjectRef::Table(table.name.clone()),
                    "table",
                    "TABLE",
                    table.comment.as_deref(),
                );
            }
            if columns {
                for column in &table.columns {
                    if column_selected(&compiled, &column.name) {
                        push(
                            &mut findings,
                            &compiled,
                            CatalogObjectRef::Column {
                                table: table.name.clone(),
                                column: column.name.clone(),
                            },
                            "column",
                            "COLUMN",
                            column.comment.as_deref(),
                        );
                    }
                }
            }
        }
    }
    if compiled.objects.contains(&ObjectKind::View) {
        for view in catalog.views().filter(|view| !view.materialized) {
            push(
                &mut findings,
                &compiled,
                CatalogObjectRef::View(view.name.clone()),
                "view",
                "VIEW",
                view.comment.as_deref(),
            );
        }
    }
    if compiled.objects.contains(&ObjectKind::MaterializedView) {
        for view in catalog.views().filter(|view| view.materialized) {
            push(
                &mut findings,
                &compiled,
                CatalogObjectRef::MaterializedView(view.name.clone()),
                "materialized view",
                "MATERIALIZED VIEW",
                view.comment.as_deref(),
            );
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn column_selected(compiled: &Compiled, name: &str) -> bool {
    let included = compiled.column_name_patterns.is_empty()
        || compiled
            .column_name_patterns
            .iter()
            .any(|pattern| pattern.is_match(name));
    included
        && !compiled
            .exempt_column_name_patterns
            .iter()
            .any(|pattern| pattern.is_match(name))
}

fn push(
    findings: &mut Vec<RuleFinding>,
    compiled: &Compiled,
    object: CatalogObjectRef,
    subject: &str,
    sql_target: &str,
    comment: Option<&str>,
) {
    let text = match comment_text(comment, compiled.min_length) {
        None => return,
        Some(text) => text,
    };
    findings.push(catalog_finding(
        RULE_ID,
        &compiled.schema_catalog_path,
        &object,
        &text
            .replace("{subject}", subject)
            .replace("{target}", sql_target),
    ));
}

fn comment_text(comment: Option<&str>, min_length: u64) -> Option<String> {
    let trimmed = comment.unwrap_or("").trim();
    let length = trimmed.chars().count() as u64;
    if length == 0 {
        Some("{subject} has no COMMENT ON {target}".to_string())
    } else if length < min_length {
        Some(format!(
            "{{subject}} comment is shorter than {min_length} characters"
        ))
    } else {
        None
    }
}
