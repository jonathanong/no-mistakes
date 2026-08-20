use super::schema::{extract_create_table_metadata, index_column_name, relation_name};
use super::types::SqlSchemaFileFacts;
use sqlparser::ast::{ObjectType, Statement};

mod constraints;
mod indexes;

pub fn extract_migration_facts(sql: &str) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts {
        tables: extract_create_table_metadata(sql),
        ..Default::default()
    };
    for statement in super::parse::parse_postgres_sql_lenient(sql) {
        match statement {
            Statement::CreateIndex(index) => {
                facts.indexes.push(indexes::from_create_index(sql, &index))
            }
            Statement::Drop {
                object_type: ObjectType::Index,
                names,
                ..
            } => facts
                .dropped_indexes
                .extend(indexes::from_drop_index(sql, &names)),
            Statement::CreateTable(table) => {
                let table_name = relation_name(&table.name);
                facts
                    .indexes
                    .extend(indexes::covering_from_table(&table_name, &table));
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

pub(super) fn relation(name: &sqlparser::ast::ObjectName) -> String {
    relation_name(name)
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
