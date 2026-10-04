use super::SqlInsertFact;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{Insert, OnConflictAction, OnInsert, Statement, TableObject};

pub(super) fn from_statement_at(
    sources: &super::lines::InsertSources<'_>,
    statement: &Statement,
    n: usize,
    positions: super::value::PlaceholderPositions<'_>,
) -> Option<SqlInsertFact> {
    match statement {
        Statement::Insert(insert) => {
            Some(from_insert_prepared(sources, insert, n, true, positions))
        }
        _ => None,
    }
}

pub(super) fn from_insert_prepared(
    sources: &super::lines::InsertSources<'_>,
    insert: &Insert,
    n: usize,
    executed: bool,
    positions: super::value::PlaceholderPositions<'_>,
) -> SqlInsertFact {
    let table = match &insert.table {
        TableObject::TableName(name) => relation_name(name),
        _ => String::new(),
    };
    SqlInsertFact {
        table,
        line: sources.line(n),
        executed,
        guarded_select: insert
            .source
            .as_deref()
            .is_some_and(super::not_exists::query_is_guarded),
        on_conflict: match &insert.on {
            Some(OnInsert::OnConflict(conflict)) => Some(from_conflict(conflict, positions)),
            _ => None,
        },
        assignments: insert_assignments(sources, insert, n, positions),
    }
}

fn from_conflict(
    conflict: &sqlparser::ast::OnConflict,
    positions: super::value::PlaceholderPositions<'_>,
) -> super::SqlOnConflictFact {
    let arbiter = super::conflict::arbiter(&conflict.conflict_target);
    match &conflict.action {
        OnConflictAction::DoNothing => super::SqlOnConflictFact {
            action: super::SqlOnConflictAction::DoNothing,
            arbiter,
            assignments: Vec::new(),
            where_proof: Default::default(),
        },
        OnConflictAction::DoUpdate(update) => super::SqlOnConflictFact {
            action: super::SqlOnConflictAction::DoUpdate,
            arbiter,
            assignments: update
                .assignments
                .iter()
                .map(|assignment| super::value::from_assignment_at(assignment, positions))
                .collect(),
            where_proof: super::conflict::where_proof_at(update.selection.as_ref(), positions),
        },
    }
}

fn insert_assignments(
    sources: &super::lines::InsertSources<'_>,
    insert: &Insert,
    n: usize,
    positions: super::value::PlaceholderPositions<'_>,
) -> Vec<super::SqlAssignmentFact> {
    if has_overriding_user_value(sources, n) {
        return Vec::new();
    }
    let set: Vec<_> = insert
        .assignments
        .iter()
        .map(|assignment| super::value::from_assignment_at(assignment, positions))
        .collect();
    if set.is_empty() {
        super::insert_source::from_insert_at(insert, positions)
    } else {
        set
    }
}

fn has_overriding_user_value(sources: &super::lines::InsertSources<'_>, n: usize) -> bool {
    let masked = super::fallback::mask_quoted_sql(sources.source(n));
    let tokens = masked
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ");
    tokens.contains("overriding user value")
}
