use super::Catalog;
use crate::codebase::postgres::statement_facts::{
    SqlConflictArbiter, SqlInsertFact, SqlOnConflictFact, SqlTriggerEvent, SqlTriggerFact,
    SqlTriggerPeriod, SqlValueForm,
};

pub(super) fn judge(
    insert: &SqlInsertFact,
    conflict: &SqlOnConflictFact,
    catalog: &Catalog<'_>,
) -> Option<String> {
    let assigned: Vec<String> = conflict
        .assignments
        .iter()
        .map(|assignment| assignment.column.clone())
        .collect();
    let generated_sources = generated_arbiter_sources(insert, conflict, catalog);
    catalog
        .triggers
        .iter()
        .filter(|trigger| trigger.table.eq_ignore_ascii_case(&insert.table))
        .find_map(|trigger| {
            unsafe_reason(trigger, conflict, &assigned, catalog, &generated_sources)
        })
}

fn unsafe_reason(
    trigger: &SqlTriggerFact,
    conflict: &SqlOnConflictFact,
    assigned: &[String],
    catalog: &Catalog<'_>,
    generated_sources: &[String],
) -> Option<String> {
    let allowlisted = catalog
        .replay_safe
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&trigger.function));
    let writes = written_columns(trigger, catalog);
    if allowlisted
        && writes.iter().any(|column| {
            generated_sources
                .iter()
                .any(|source| source.eq_ignore_ascii_case(column))
        })
    {
        return Some(format!(
            "allowlisted trigger {} writes a generated-arbiter source",
            trigger.function
        ));
    }
    if !trigger.for_each_row {
        return if allowlisted {
            None
        } else {
            Some(format!(
                "statement-level trigger {} fires on every replay",
                trigger.function
            ))
        };
    }
    if trigger.period == SqlTriggerPeriod::Before && fires_insert(trigger) {
        return if allowlisted {
            None
        } else {
            Some(format!(
                "BEFORE INSERT trigger {} fires before conflict checks",
                trigger.function
            ))
        };
    }
    if !fires_update(trigger, assigned) {
        return None;
    }
    if allowlisted {
        return None;
    }
    if trigger.period == SqlTriggerPeriod::After && where_proves_noop(conflict, assigned) {
        return None;
    }
    Some(format!(
        "ON CONFLICT DO UPDATE can re-fire trigger {}",
        trigger.function
    ))
}

fn written_columns(trigger: &SqlTriggerFact, catalog: &Catalog<'_>) -> Vec<String> {
    if let Some((_, columns)) = catalog
        .trigger_writes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&trigger.function))
    {
        return columns.clone();
    }
    trigger
        .events
        .iter()
        .filter_map(|event| match event {
            SqlTriggerEvent::Update { columns } if !columns.is_empty() => Some(columns.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn generated_arbiter_sources(
    insert: &SqlInsertFact,
    conflict: &SqlOnConflictFact,
    catalog: &Catalog<'_>,
) -> Vec<String> {
    let SqlConflictArbiter::Columns(arbiter) = &conflict.arbiter else {
        return Vec::new();
    };
    catalog
        .schema
        .iter()
        .flat_map(|file| file.tables.iter())
        .filter(|table| table.table_name.eq_ignore_ascii_case(&insert.table))
        .flat_map(|table| table.columns.iter())
        .filter(|column| {
            column.is_generated
                && arbiter
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&column.name))
        })
        .flat_map(|column| {
            if column.generated_source_columns.is_empty() {
                column.generated_function_arg_columns.clone()
            } else {
                column.generated_source_columns.clone()
            }
        })
        .collect()
}

fn fires_insert(trigger: &SqlTriggerFact) -> bool {
    trigger
        .events
        .iter()
        .any(|event| matches!(event, SqlTriggerEvent::Insert))
}

fn fires_update(trigger: &SqlTriggerFact, assigned: &[String]) -> bool {
    trigger.events.iter().any(|event| match event {
        SqlTriggerEvent::Update { columns } if columns.is_empty() => true,
        SqlTriggerEvent::Update { columns } => columns.iter().any(|column| {
            assigned
                .iter()
                .any(|assigned| assigned.eq_ignore_ascii_case(column))
        }),
        _ => false,
    })
}

fn where_proves_noop(conflict: &SqlOnConflictFact, assigned: &[String]) -> bool {
    if conflict.where_proof.disjunctive || assigned.is_empty() {
        return false;
    }
    assigned.iter().all(|column| {
        conflict
            .where_proof
            .distinct_from_excluded
            .iter()
            .any(|name| name.eq_ignore_ascii_case(column))
            || conflict
                .where_proof
                .null_and_excluded_not_null
                .iter()
                .any(|name| name.eq_ignore_ascii_case(column))
            || conflict.assignments.iter().any(|assignment| {
                assignment.column.eq_ignore_ascii_case(column)
                    && matches!(assignment.form, SqlValueForm::SelfRef { .. })
            })
    })
}
