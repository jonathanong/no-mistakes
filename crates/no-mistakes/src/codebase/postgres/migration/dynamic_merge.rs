use super::{dynamic, extract_migration_facts};
use crate::codebase::postgres::types::SqlSchemaFileFacts;

/// Merge the already discovered routine body SQL and static EXECUTE SQL.
pub(super) fn append(sql: &str, facts: &mut SqlSchemaFileFacts) {
    for source in dynamic::schema_bodies(sql) {
        if source.clear_outer_wrapper {
            let outer = source.source_order.first().copied();
            facts.unanalyzable_statements.retain(|marker| {
                marker.source_order.first().copied() != outer
                    || marker.source_order.len() != 1
                    || !marker.head.split_whitespace().next().is_some_and(|head| {
                        head.eq_ignore_ascii_case("DO") || head.eq_ignore_ascii_case("CREATE")
                    })
            });
        }
        append_one(facts, source, true);
    }
    for source in dynamic::extract(sql) {
        // A peeled DO body can leave an opaque PL/pgSQL EXECUTE wrapper marker.
        // The exact statement path proves that the static expression was projected.
        facts.unanalyzable_statements.retain(|marker| {
            marker.source_order != source.source_order || !execute_wrapper(&marker.head)
        });
        append_one(facts, source, false);
    }
}

fn execute_wrapper(head: &str) -> bool {
    let upper = head.to_ascii_uppercase();
    upper.starts_with("EXECUTE ") || upper.starts_with("BEGIN EXECUTE ")
}

fn append_one(facts: &mut SqlSchemaFileFacts, source: dynamic::DynamicSql, body: bool) {
    let mut nested = extract_migration_facts(&source.sql);
    if body && source.procedural_control {
        // The source is a recognized body-control chunk. Keep any nested SQL failure.
        nested
            .unanalyzable_statements
            .retain(|marker| marker.source_order.len() != 1);
    }
    if !source.executed {
        nested.table_events.clear();
    }
    dynamic::remap_fact_lines(&mut nested, &source);
    merge(facts, nested);
}

fn merge(facts: &mut SqlSchemaFileFacts, dynamic: SqlSchemaFileFacts) {
    facts
        .unanalyzable_statements
        .extend(dynamic.unanalyzable_statements);
    facts.tables.extend(dynamic.tables);
    facts.table_events.extend(dynamic.table_events);
    facts.indexes.extend(dynamic.indexes);
    facts.dropped_indexes.extend(dynamic.dropped_indexes);
    facts.dropped_tables.extend(dynamic.dropped_tables);
    facts.foreign_keys.extend(dynamic.foreign_keys);
    facts.add_columns.extend(dynamic.add_columns);
    facts
        .unnamed_constraints
        .extend(dynamic.unnamed_constraints);
    facts.statement_kinds.extend(dynamic.statement_kinds);
    facts.setting_uses.extend(dynamic.setting_uses);
    facts.function_calls.extend(dynamic.function_calls);
    facts
        .not_valid_constraints
        .extend(dynamic.not_valid_constraints);
    facts
        .validated_constraints
        .extend(dynamic.validated_constraints);
    facts
        .declared_identifiers
        .extend(dynamic.declared_identifiers);
}
