use super::SqlInsertFact;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{Insert, OnConflictAction, OnInsert, Statement, TableObject};

pub(super) fn from_statement(sql: &str, statement: &Statement, n: usize) -> Option<SqlInsertFact> {
    match statement {
        Statement::Insert(insert) => Some(from_insert(sql, insert, n, true)),
        _ => None,
    }
}

pub(super) fn from_insert(sql: &str, insert: &Insert, n: usize, executed: bool) -> SqlInsertFact {
    let table = match &insert.table {
        TableObject::TableName(name) => relation_name(name),
        _ => String::new(),
    };
    SqlInsertFact {
        table,
        line: super::lines::nth_insert_line(sql, n),
        executed,
        guarded_select: insert
            .source
            .as_deref()
            .is_some_and(super::not_exists::query_is_guarded),
        on_conflict: match &insert.on {
            Some(OnInsert::OnConflict(conflict)) => Some(from_conflict(conflict)),
            _ => None,
        },
        assignments: insert_assignments(sql, insert),
    }
}

fn from_conflict(conflict: &sqlparser::ast::OnConflict) -> super::SqlOnConflictFact {
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
                .map(super::value::from_assignment)
                .collect(),
            where_proof: super::conflict::where_proof(update.selection.as_ref()),
        },
    }
}

fn insert_assignments(sql: &str, insert: &Insert) -> Vec<super::SqlAssignmentFact> {
    if has_overriding_user_value(sql) {
        return Vec::new();
    }
    let set: Vec<_> = insert
        .assignments
        .iter()
        .map(super::value::from_assignment)
        .collect();
    if set.is_empty() {
        super::insert_source::from_insert(insert)
    } else {
        set
    }
}

fn has_overriding_user_value(sql: &str) -> bool {
    super::fallback::mask_quoted_sql(sql)
        .to_ascii_lowercase()
        .contains("overriding user value")
}
