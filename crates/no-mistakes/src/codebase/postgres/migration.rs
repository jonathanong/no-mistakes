use super::schema::{index_column_name, relation_name};
use super::types::SqlSchemaFileFacts;
use sqlparser::ast::{ObjectName, ObjectNamePart, ObjectType, Statement};

mod constraints;
mod dynamic;
mod dynamic_merge;
mod identifiers;
mod indexes;
mod lines;
mod predicate;
mod recovery;
mod statements;
mod table_events;
pub(crate) use recovery::recovery_failures;

pub fn extract_migration_facts(sql: &str) -> SqlSchemaFileFacts {
    let parsed = super::parse::parse_postgres_sql_with_function_sources(sql);
    let unanalyzable_statements = recovery_failures(&parsed);
    let (located, functions) = super::parse::partition_function_sources(parsed);
    let statements: Vec<_> = located
        .into_iter()
        .map(|located| located.statement)
        .collect();
    extract_from_parsed_with_failures(sql, &statements, &functions, &unanalyzable_statements)
}

pub(crate) fn extract_from_parsed(
    sql: &str,
    statements: &[Statement],
    functions: &[super::SqlFunctionCallFact],
) -> SqlSchemaFileFacts {
    extract_from_parsed_with_failures(sql, statements, functions, &[])
}

pub(crate) fn extract_from_parsed_with_failures(
    sql: &str,
    statements: &[Statement],
    functions: &[super::SqlFunctionCallFact],
    failures: &[super::types::SqlUnanalyzableStatement],
) -> SqlSchemaFileFacts {
    let mut facts = extract_parsed_migration_facts(sql, statements, failures);
    facts.function_calls.extend_from_slice(functions);
    facts
}

pub(crate) use statements::{policy_facts, SUPPORTED_KINDS};

fn extract_parsed_migration_facts(
    sql: &str,
    statements: &[Statement],
    failures: &[super::types::SqlUnanalyzableStatement],
) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts {
        unanalyzable_statements: failures.to_vec(),
        table_events_collected: true,
        tables: statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::CreateTable(table) => Some(super::schema::table_metadata(table)),
                _ => None,
            })
            .collect(),
        ..Default::default()
    };
    statements::record_new(sql, &mut facts);
    let mut create_index_n = 0usize;
    let mut drop_index_n = 0usize;
    let mut drop_table_n = 0usize;
    let mut identifier_locations = identifiers::Locations::new(sql);
    facts
        .declared_identifiers
        .extend(identifier_locations.unparsed_declarations());
    let mut table_positions = table_events::Positions::new(sql);
    for statement in statements {
        facts
            .declared_identifiers
            .extend(identifiers::collect(&mut identifier_locations, statement));
        statements::record(sql, statement, &mut facts);
        table_events::record(statement, &mut table_positions, &mut facts.table_events);
        match statement {
            Statement::CreateIndex(index) => {
                create_index_n += 1;
                facts
                    .indexes
                    .push(indexes::from_create_index(sql, create_index_n, index));
            }
            Statement::Drop {
                object_type: ObjectType::Index,
                names,
                ..
            } => {
                drop_index_n += 1;
                facts
                    .dropped_indexes
                    .extend(indexes::from_drop_index(sql, drop_index_n, names));
            }
            Statement::Drop {
                object_type: ObjectType::Table,
                names,
                ..
            } => {
                drop_table_n += 1;
                facts
                    .dropped_tables
                    .extend(indexes::from_drop_table(sql, drop_table_n, names));
            }
            Statement::CreateTable(table) => {
                let table_name = relation_name(&table.name);
                facts.indexes.extend(indexes::covering_from_table(
                    &qualified_relation(&table.name),
                    table,
                ));
                constraints::collect_create_table_fks(
                    sql,
                    &table_name,
                    table,
                    &mut facts.foreign_keys,
                );
            }
            Statement::AlterTable(alter) => {
                constraints::collect_alter_table(sql, alter, &mut facts)
            }
            _ => {}
        }
    }
    dynamic_merge::append(sql, &mut facts);
    facts.unanalyzable_statements.sort_by_key(|item| item.line);
    table_positions.apply_transaction_projection(&mut facts.table_events);
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
