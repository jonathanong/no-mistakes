use super::compile::Compiled;
use super::policy::{min_words_text, tokens, NameFlags};
use super::scan_name::{consider, push, NameCheck};
use crate::codebase::postgres::{CatalogIndexInfo, CatalogObjectRef, CatalogTable, SchemaCatalog};
use crate::codebase::rules::RuleFinding;

pub(super) fn scan(catalog: &SchemaCatalog, compiled: &Compiled, path: &str) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        scan_table(table, compiled, path, &mut findings);
    }
    for function in catalog.functions() {
        let object = CatalogObjectRef::Function(function.key.clone());
        consider(
            named(
                "function",
                &function.name,
                object.clone(),
                None,
                NameFlags::tokens(),
            ),
            compiled,
            path,
            &mut findings,
        );
        if function.returns_trigger {
            consider(
                named(
                    "triggerFunction",
                    &function.name,
                    object,
                    None,
                    NameFlags::default(),
                ),
                compiled,
                path,
                &mut findings,
            );
        }
    }
    for view in catalog.views() {
        let (kind, object) = if view.materialized {
            (
                "materializedView",
                CatalogObjectRef::MaterializedView(view.name.clone()),
            )
        } else {
            ("view", CatalogObjectRef::View(view.name.clone()))
        };
        consider(
            named(kind, &view.name, object, None, NameFlags::underscore()),
            compiled,
            path,
            &mut findings,
        );
    }
    for enum_type in catalog.enums() {
        consider(
            named(
                "enum",
                &enum_type.name,
                CatalogObjectRef::Enum(enum_type.name.clone()),
                None,
                NameFlags::enum_name(),
            ),
            compiled,
            path,
            &mut findings,
        );
    }
    super::super::sort_findings(&mut findings);
    let mut findings = compiled.allow.clone().apply(path, findings);
    super::super::sort_findings(&mut findings);
    findings
}

fn scan_table(
    table: &CatalogTable,
    compiled: &Compiled,
    path: &str,
    findings: &mut Vec<RuleFinding>,
) {
    consider(
        named(
            "table",
            &table.name,
            CatalogObjectRef::Table(table.name.clone()),
            None,
            NameFlags::table(),
        ),
        compiled,
        path,
        findings,
    );
    if let Some(minimum) = compiled.table_min_words {
        let count = tokens(&table.name).len();
        if count < minimum {
            push(
                findings,
                path,
                CatalogObjectRef::Table(table.name.clone()),
                &min_words_text(count, minimum, &table.name),
            );
        }
    }
    for column in &table.columns {
        consider(
            named(
                "column",
                &column.name,
                CatalogObjectRef::Column {
                    table: table.name.clone(),
                    column: column.name.clone(),
                },
                None,
                NameFlags::tokens(),
            ),
            compiled,
            path,
            findings,
        );
    }
    for index in &table.indexes {
        if index.constraint_backed && !compiled.check_constraint_backed_indexes {
            continue;
        }
        scan_index(table, index, compiled, path, findings);
    }
    for trigger in &table.triggers {
        consider(
            named(
                "trigger",
                &trigger.name,
                CatalogObjectRef::Trigger {
                    table: table.name.clone(),
                    trigger: trigger.name.clone(),
                },
                Some(table.name.as_str()),
                NameFlags::tokens(),
            ),
            compiled,
            path,
            findings,
        );
    }
}

fn scan_index(
    table: &CatalogTable,
    index: &CatalogIndexInfo,
    compiled: &Compiled,
    path: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let kind = if index.unique && compiled.patterns.contains_key("uniqueIndex") {
        "uniqueIndex"
    } else {
        "index"
    };
    consider(
        named(
            kind,
            &index.name,
            CatalogObjectRef::Index {
                table: table.name.clone(),
                index: index.name.clone(),
            },
            Some(table.name.as_str()),
            NameFlags::tokens(),
        ),
        compiled,
        path,
        findings,
    );
}

fn named<'a>(
    kind: &'a str,
    name: &'a str,
    object: CatalogObjectRef,
    table: Option<&'a str>,
    flags: NameFlags,
) -> NameCheck<'a> {
    NameCheck {
        kind,
        name,
        object,
        table,
        flags,
    }
}
