use super::relation;
mod names;
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::types::{
    SqlAddColumnMetadata, SqlForeignKeyMetadata, SqlNamedConstraint, SqlSchemaFileFacts,
    SqlUnnamedConstraint,
};
pub(super) use names::constraint_name;
use sqlparser::ast::{
    AlterTableOperation, ColumnOption, ForeignKeyConstraint, Spanned, TableConstraint,
};

pub(super) fn collect_create_table_fks(
    line: usize,
    table_name: &str,
    table: &sqlparser::ast::CreateTable,
    foreign_keys: &mut Vec<SqlForeignKeyMetadata>,
) {
    for column in &table.columns {
        for option in &column.options {
            if let ColumnOption::ForeignKey(fk) = &option.option {
                // Inline FK directives belong to the declaration, not the CREATE head.
                let declaration_line = fk.span().start.line as usize;
                foreign_keys.push(fk_metadata(
                    table_name,
                    Some(&column.name.value),
                    fk,
                    if declaration_line == 0 {
                        line
                    } else {
                        declaration_line
                    },
                ));
            }
        }
    }
    for constraint in &table.constraints {
        if let TableConstraint::ForeignKey(fk) = constraint {
            foreign_keys.push(fk_metadata(table_name, None, fk, line));
        }
    }
}

pub(super) fn collect_alter_table(
    line: usize,
    alter: &sqlparser::ast::AlterTable,
    facts: &mut SqlSchemaFileFacts,
) {
    let table_name = relation(&alter.name);
    let constraint_table = names::table_key(&alter.name);
    let index_table = super::qualified_relation(&alter.name);
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
                            .push(fk_metadata(&table_name, None, fk, line))
                    }
                    TableConstraint::Unique(unique) => {
                        facts
                            .indexes
                            .push(super::indexes::covering_from_index_columns(
                                &index_table,
                                &unique.columns,
                            ));
                    }
                    TableConstraint::PrimaryKey(primary) => {
                        facts
                            .indexes
                            .push(super::indexes::covering_from_index_columns(
                                &index_table,
                                &primary.columns,
                            ));
                    }
                    _ => {}
                }
                if let Some(kind) = unnamed_fk_or_check_kind(constraint) {
                    facts.unnamed_constraints.push(SqlUnnamedConstraint {
                        table_name: table_name.clone(),
                        kind: kind.to_string(),
                        line,
                    });
                }
                if *not_valid {
                    if let Some(name) = constraint_name(constraint)
                        .or_else(|| names::default_check_name(&constraint_table, constraint))
                    {
                        facts.not_valid_constraints.push(SqlNamedConstraint {
                            table_name: constraint_table.clone(),
                            name: name.clone(),
                            line,
                        });
                    }
                }
            }
            AlterTableOperation::ValidateConstraint { name } => {
                facts.validated_constraints.push(SqlNamedConstraint {
                    table_name: constraint_table.clone(),
                    name: ident_key(name),
                    line,
                });
            }
            AlterTableOperation::AddColumn { column_def, .. } => {
                facts.add_columns.push(SqlAddColumnMetadata {
                    table_name: index_table.clone(),
                    unqualified_table_name: table_name.clone(),
                    column_name: column_def.name.value.clone(),
                    data_type: column_def.data_type.to_string(),
                    is_generated: column_def.options.iter().any(|option| {
                        matches!(
                            &option.option,
                            ColumnOption::Generated {
                                generation_expr: Some(_),
                                ..
                            }
                        )
                    }),
                    nullable: !column_def
                        .options
                        .iter()
                        .any(|option| matches!(&option.option, ColumnOption::NotNull)),
                    default: column_def.options.iter().find_map(|option| {
                        if let ColumnOption::Default(default) = &option.option {
                            Some(default.to_string())
                        } else {
                            None
                        }
                    }),
                    line,
                });
                for option in &column_def.options {
                    if let ColumnOption::ForeignKey(fk) = &option.option {
                        facts.foreign_keys.push(fk_metadata(
                            &table_name,
                            Some(&column_def.name.value),
                            fk,
                            line,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn fk_metadata(
    table_name: &str,
    column_name: Option<&str>,
    fk: &ForeignKeyConstraint,
    line: usize,
) -> SqlForeignKeyMetadata {
    let column_names = if fk.columns.is_empty() {
        column_name
            .map(|name| vec![name.to_string()])
            .unwrap_or_default()
    } else {
        fk.columns.iter().map(|ident| ident.value.clone()).collect()
    };
    SqlForeignKeyMetadata {
        table_name: table_name.to_string(),
        column_names,
        referenced_table_name: relation(&fk.foreign_table),
        delete_action: fk.on_delete.as_ref().map(ToString::to_string),
        line,
    }
}

fn unnamed_fk_or_check_kind(constraint: &TableConstraint) -> Option<&'static str> {
    match constraint {
        TableConstraint::ForeignKey(fk) if fk.name.is_none() => Some("FOREIGN KEY"),
        TableConstraint::Check(check) if check.name.is_none() => Some("CHECK"),
        _ => None,
    }
}
