use super::Catalog;
use crate::codebase::postgres::statement_facts::{
    SqlConflictArbiter, SqlInsertFact, SqlOnConflictAction, SqlOnConflictFact, SqlTriggerEvent,
    SqlTriggerFact, SqlTriggerPeriod,
};

pub(super) fn judge_guarded_select(
    insert: &SqlInsertFact,
    catalog: &Catalog<'_>,
) -> Option<String> {
    catalog.triggers.iter().find_map(|trigger| {
        if !trigger.table.eq_ignore_ascii_case(&insert.table)
            || trigger.for_each_row
            || !fires_insert(trigger)
        {
            return None;
        }
        let allowlisted = catalog
            .replay_safe
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&trigger.function));
        (!allowlisted).then(|| {
            format!(
                "statement-level trigger {} fires on every replay",
                trigger.function
            )
        })
    })
}

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
    let generated_sources =
        super::generated::arbiter_source_columns(insert, conflict, catalog.schema);
    let arbiter_columns = match &conflict.arbiter {
        SqlConflictArbiter::Columns(columns) => columns.clone(),
        _ => Vec::new(),
    };
    catalog
        .triggers
        .iter()
        .filter(|trigger| trigger.table.eq_ignore_ascii_case(&insert.table))
        .find_map(|trigger| {
            unsafe_reason(
                trigger,
                conflict,
                &assigned,
                catalog,
                &generated_sources,
                &arbiter_columns,
            )
        })
}

fn unsafe_reason(
    trigger: &SqlTriggerFact,
    conflict: &SqlOnConflictFact,
    assigned: &[String],
    catalog: &Catalog<'_>,
    generated_sources: &[String],
    arbiter_columns: &[String],
) -> Option<String> {
    let allowlisted = catalog
        .replay_safe
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&trigger.function));
    let writes = written_columns(trigger, catalog);
    let applies = (trigger.for_each_row
        && (fires_insert(trigger) || fires_update(trigger, assigned)))
        || (!trigger.for_each_row
            && (fires_insert(trigger)
                || (has_update_event(trigger)
                    && conflict.action == SqlOnConflictAction::DoUpdate)));
    if !applies {
        return None;
    }
    if allowlisted
        && writes.iter().any(|column| {
            generated_sources
                .iter()
                .any(|source| source.eq_ignore_ascii_case(column))
                || arbiter_columns
                    .iter()
                    .any(|arbiter| arbiter.eq_ignore_ascii_case(column))
        })
    {
        return Some(format!(
            "allowlisted trigger {} writes a conflict-arbiter column",
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

fn fires_insert(trigger: &SqlTriggerFact) -> bool {
    trigger
        .events
        .iter()
        .any(|event| matches!(event, SqlTriggerEvent::Insert))
}

fn has_update_event(trigger: &SqlTriggerFact) -> bool {
    trigger
        .events
        .iter()
        .any(|event| matches!(event, SqlTriggerEvent::Update { .. }))
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
    assigned.iter().any(|column| {
        conflict.assignments.iter().any(|assignment| {
            assignment.column.eq_ignore_ascii_case(column)
                && super::form_is_excluded(&assignment.form, column)
                && (conflict
                    .where_proof
                    .distinct_from_excluded
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(column))
                    || conflict
                        .where_proof
                        .null_and_excluded_not_null
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(column)))
        })
    })
}
