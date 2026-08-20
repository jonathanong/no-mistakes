use super::schema::{extract_create_table_metadata, index_column_name, relation_name};
use super::types::SqlSchemaFileFacts;
use sqlparser::ast::{ObjectName, ObjectNamePart, ObjectType, Statement};

mod constraints;
mod indexes;
mod lines;
mod predicate;

pub fn extract_migration_facts(sql: &str) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts {
        tables: extract_create_table_metadata(sql),
        ..Default::default()
    };
    let mut create_index_n = 0usize;
    let mut drop_index_n = 0usize;
    let mut drop_table_n = 0usize;
    for statement in super::parse::parse_postgres_sql_lenient(sql) {
        match statement {
            Statement::CreateIndex(index) => {
                create_index_n += 1;
                facts
                    .indexes
                    .push(indexes::from_create_index(sql, create_index_n, &index));
            }
            Statement::Drop {
                object_type: ObjectType::Index,
                names,
                ..
            } => {
                drop_index_n += 1;
                facts
                    .dropped_indexes
                    .extend(indexes::from_drop_index(sql, drop_index_n, &names));
            }
            Statement::Drop {
                object_type: ObjectType::Table,
                names,
                ..
            } => {
                drop_table_n += 1;
                facts
                    .dropped_tables
                    .extend(indexes::from_drop_table(sql, drop_table_n, &names));
            }
            Statement::CreateTable(table) => {
                let table_name = relation_name(&table.name);
                facts.indexes.extend(indexes::covering_from_table(
                    &qualified_relation(&table.name),
                    &table,
                ));
                constraints::collect_create_table_fks(
                    sql,
                    &table_name,
                    &table,
                    &mut facts.foreign_keys,
                );
            }
            Statement::AlterTable(alter) => {
                constraints::collect_alter_table(sql, &alter, &mut facts)
            }
            _ => {}
        }
    }
    facts
}

pub(super) fn relation(name: &ObjectName) -> String {
    relation_name(name)
}

pub(super) fn qualified_relation(name: &ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.value.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub(super) fn leading_index_column(column: &sqlparser::ast::IndexColumn) -> Option<String> {
    index_column_name(column)
}

pub(super) fn line_containing(source: &str, parts: &[&str]) -> usize {
    source
        .lines()
        .enumerate()
        .find(|(_, line)| {
            let lower = line.to_ascii_lowercase();
            parts
                .iter()
                .all(|part| lower.contains(&part.to_ascii_lowercase()))
        })
        .map(|(index, _)| index + 1)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests;
