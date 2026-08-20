use super::{line_containing, relation};
use crate::codebase::postgres::types::{
    SqlForeignKeyMetadata, SqlNamedConstraint, SqlSchemaFileFacts,
};
use sqlparser::ast::{AlterTableOperation, ColumnOption, ForeignKeyConstraint, TableConstraint};

pub(super) fn collect_create_table_fks(
    sql: &str,
    table_name: &str,
    table: &sqlparser::ast::CreateTable,
    foreign_keys: &mut Vec<SqlForeignKeyMetadata>,
) {
    for column in &table.columns {
        for option in &column.options {
            if let ColumnOption::ForeignKey(fk) = &option.option {
                foreign_keys.push(fk_metadata(sql, table_name, Some(&column.name.value), fk));
            }
        }
    }
    for constraint in &table.constraints {
        if let TableConstraint::ForeignKey(fk) = constraint {
            foreign_keys.push(fk_metadata(sql, table_name, None, fk));
        }
    }
}

pub(super) fn collect_alter_table(
    sql: &str,
    alter: &sqlparser::ast::AlterTable,
    facts: &mut SqlSchemaFileFacts,
) {
    let table_name = relation(&alter.name);
    for operation in &alter.operations {
        match operation {
            AlterTableOperation::AddConstraint {
                constraint,
                not_valid,
            } => {
                match constraint {
                    TableConstraint::ForeignKey(fk) => {
                        facts
                            .foreign_keys
                            .push(fk_metadata(sql, &table_name, None, fk))
                    }
                    TableConstraint::Unique(unique) => {
                        facts
                            .indexes
                            .push(super::indexes::covering_from_index_columns(
                                &table_name,
                                &unique.columns,
                            ));
                    }
                    TableConstraint::PrimaryKey(primary) => {
                        facts
                            .indexes
                            .push(super::indexes::covering_from_index_columns(
                                &table_name,
                                &primary.columns,
                            ));
                    }
                    _ => {}
                }
                if *not_valid {
                    if let Some(name) = constraint_name(constraint) {
                        facts.not_valid_constraints.push(SqlNamedConstraint {
                            table_name: table_name.clone(),
                            name: name.clone(),
                            line: line_containing(sql, &["not valid", &name]),
                        });
                    }
                }
            }
            AlterTableOperation::ValidateConstraint { name } => {
                facts.validated_constraints.push(SqlNamedConstraint {
                    table_name: table_name.clone(),
                    name: name.value.clone(),
                    line: line_containing(sql, &["validate constraint", &name.value]),
                });
            }
            AlterTableOperation::AddColumn { column_def, .. } => {
                for option in &column_def.options {
                    if let ColumnOption::ForeignKey(fk) = &option.option {
                        facts.foreign_keys.push(fk_metadata(
                            sql,
                            &table_name,
                            Some(&column_def.name.value),
                            fk,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn fk_metadata(
    sql: &str,
    table_name: &str,
    column_name: Option<&str>,
    fk: &ForeignKeyConstraint,
) -> SqlForeignKeyMetadata {
    let column_names = if fk.columns.is_empty() {
        column_name
            .map(|name| vec![name.to_string()])
            .unwrap_or_default()
    } else {
        fk.columns.iter().map(|ident| ident.value.clone()).collect()
    };
    let line = line_containing(
        sql,
        &[
            column_names
                .first()
                .map(String::as_str)
                .unwrap_or("references"),
            "references",
        ],
    );
    SqlForeignKeyMetadata {
        table_name: table_name.to_string(),
        column_names,
        referenced_table_name: relation(&fk.foreign_table),
        delete_action: fk.on_delete.as_ref().map(ToString::to_string),
        line,
    }
}

pub(super) fn constraint_name(constraint: &TableConstraint) -> Option<String> {
    match constraint {
        TableConstraint::ForeignKey(fk) => fk.name.as_ref().map(|ident| ident.value.clone()),
        TableConstraint::Check(check) => check.name.as_ref().map(|ident| ident.value.clone()),
        TableConstraint::Unique(unique) => unique.name.as_ref().map(|ident| ident.value.clone()),
        TableConstraint::PrimaryKey(primary) => {
            primary.name.as_ref().map(|ident| ident.value.clone())
        }
        _ => None,
    }
}
