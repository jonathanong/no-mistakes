use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::{visit_expressions, Expr};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

pub(super) fn scan(opts: &CompiledOptions, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let Some(pattern) = &opts.fingerprint_name_pattern else {
        return Vec::new();
    };
    let file = opts.schema_catalog_path.replace('\\', "/");
    let mut findings = Vec::new();
    for table in catalog.tables() {
        for column in &table.columns {
            if !matches!(
                column.data_type.to_ascii_lowercase().as_str(),
                "bytea" | "text" | "pg_catalog.bytea" | "pg_catalog.text"
            ) || !pattern.is_match(&column.name)
                || opts.allows(&table.name, &column.name)
            {
                continue;
            }
            for key in unique_keys(catalog, table, &column.name) {
                let line = catalog.column_line(&table.name, &column.name);
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.clone(),
                    line,
                    message: format!("{file}:{line}: column {}.{} matches fingerprintNamePattern and is part of UNIQUE key {key}; key on foreign key columns instead", table.name, column.name),
                    import: None,
                    target: Some(format!("column:{}.{}", table.name, column.name)),
                });
            }
        }
    }
    findings
}

fn unique_keys<'a>(
    catalog: &'a SchemaCatalog,
    table: &'a CatalogTable,
    column: &str,
) -> BTreeSet<&'a str> {
    table
        .unique_constraints
        .iter()
        .filter(|key| key.columns.iter().any(|name| name == column))
        .map(|key| key.name.as_str())
        .chain(
            table
                .indexes
                .iter()
                .filter(|index| {
                    index.unique
                        && index
                            .keys
                            .iter()
                            .any(|key| key.column.as_deref() == Some(column))
                })
                .map(|index| index.name.as_str()),
        )
        .chain(
            catalog
                .bounded_unique_indexes(&table.name)
                .iter()
                .filter(|index| {
                    index
                        .keys
                        .iter()
                        .any(|expr| references_column(expr, catalog, table, column))
                })
                .map(|index| index.name.as_str()),
        )
        .collect()
}

fn references_column(
    expr: &Expr,
    catalog: &SchemaCatalog,
    table: &CatalogTable,
    column: &str,
) -> bool {
    visit_expressions(expr, |expr| {
        let refers = match expr {
            Expr::Identifier(ident) => {
                crate::codebase::postgres::idents::ident_key(ident) == column
            }
            Expr::CompoundIdentifier(parts) => parts.last().is_some_and(|ident| {
                crate::codebase::postgres::idents::ident_key(ident) == column
                    && catalog
                        .relation(
                            &parts[..parts.len() - 1]
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join("."),
                        )
                        .is_some_and(|owner| owner.name == table.name)
            }),
            // Function names and literal text are not column references.
            _ => false,
        };
        if refers {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_break()
}
