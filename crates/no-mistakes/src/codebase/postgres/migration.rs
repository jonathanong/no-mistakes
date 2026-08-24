use super::schema::{extract_create_table_metadata, index_column_name, relation_name};
use super::types::SqlSchemaFileFacts;
use sqlparser::ast::{ObjectName, ObjectNamePart, ObjectType, Statement};

mod constraints;
mod dynamic;
mod indexes;
mod lines;
mod predicate;
mod statements;

pub fn extract_migration_facts(sql: &str) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts {
        tables: extract_create_table_metadata(sql),
        ..Default::default()
    };
    let mut create_index_n = 0usize;
    let mut drop_index_n = 0usize;
    let mut drop_table_n = 0usize;
    for statement in super::parse::parse_postgres_sql_lenient(sql) {
        statements::record(sql, &statement, &mut facts);
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
    // Analyze direct body DDL and static EXECUTE SQL through the same extractor,
    // so every downstream PostgreSQL rule consumes identical schema facts.
    for dynamic_sql in dynamic::schema_bodies(sql)
        .into_iter()
        .chain(dynamic::extract(sql))
    {
        let mut dynamic_facts = extract_migration_facts(&dynamic_sql.sql);
        remap_dynamic_fact_lines(&mut dynamic_facts, &dynamic_sql);
        merge_dynamic_facts(&mut facts, dynamic_facts);
    }
    facts
}

fn remap_dynamic_fact_lines(facts: &mut SqlSchemaFileFacts, dynamic: &dynamic::DynamicSql) {
    for index in &mut facts.indexes {
        index.line = dynamic.source_line(index.line);
    }
    for index in &mut facts.dropped_indexes {
        index.line = dynamic.source_line(index.line);
    }
    for table in &mut facts.dropped_tables {
        table.line = dynamic.source_line(table.line);
    }
    for key in &mut facts.foreign_keys {
        key.line = dynamic.source_line(key.line);
    }
    for column in &mut facts.add_columns {
        column.line = dynamic.source_line(column.line);
    }
    for constraint in &mut facts.unnamed_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
    for statement in &mut facts.statement_kinds {
        statement.line = dynamic.source_line(statement.line);
    }
    for constraint in &mut facts.not_valid_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
    for constraint in &mut facts.validated_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
}

fn merge_dynamic_facts(facts: &mut SqlSchemaFileFacts, dynamic: SqlSchemaFileFacts) {
    facts.tables.extend(dynamic.tables);
    facts.indexes.extend(dynamic.indexes);
    facts.dropped_indexes.extend(dynamic.dropped_indexes);
    facts.dropped_tables.extend(dynamic.dropped_tables);
    facts.foreign_keys.extend(dynamic.foreign_keys);
    facts.add_columns.extend(dynamic.add_columns);
    facts
        .unnamed_constraints
        .extend(dynamic.unnamed_constraints);
    facts.statement_kinds.extend(dynamic.statement_kinds);
    facts
        .not_valid_constraints
        .extend(dynamic.not_valid_constraints);
    facts
        .validated_constraints
        .extend(dynamic.validated_constraints);
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
