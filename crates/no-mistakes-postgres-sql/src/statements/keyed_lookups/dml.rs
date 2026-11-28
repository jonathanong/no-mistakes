mod merge;

use super::conditions::literal_condition;
use super::join::register_table_with_joins;
use super::query::has_zero_limit;
use super::scope::ScopeStack;
use super::{inspect_exprs, returning_exprs, LookupHistory};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
pub(super) use merge::walk_merge;
use sqlparser::ast::{
    Delete, Expr, FromTable, Insert, OnConflict, OnConflictAction, OnInsert, TableObject, Update,
};

pub(super) fn walk_insert(
    insert: &Insert,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some(source) = insert.source.as_deref() {
        super::walk_query(source, scopes, out);
        if has_zero_limit(source.limit_clause.as_ref()) {
            return;
        }
    }
    let update = match &insert.on {
        Some(OnInsert::OnConflict(OnConflict {
            action: OnConflictAction::DoUpdate(update),
            ..
        })) if can_affect_rows(update.selection.as_ref()) => Some(update),
        _ => None,
    };
    let returning = insert.returning.as_deref();
    if update.is_none() && returning.is_none() {
        return;
    }
    scopes.push_frame();
    if let TableObject::TableName(name) = &insert.table {
        scopes.add_table_alias(name, insert.table_alias.as_ref().map(|a| &a.alias));
    }
    if let Some(update) = update {
        scopes.push_frame();
        scopes.add_named_alias("excluded".to_owned());
        let exprs = update
            .assignments
            .iter()
            .map(|assignment| &assignment.value)
            .chain(update.selection.iter());
        inspect_exprs(exprs, scopes, &mut LookupHistory::new(), out);
        scopes.pop_frame();
    }
    if let Some(returning) = returning {
        scopes.add_returning_aliases(Some("new"));
        inspect_exprs(
            returning_exprs(Some(returning)),
            scopes,
            &mut LookupHistory::new(),
            out,
        );
    }
    scopes.pop_frame();
}

pub(super) fn walk_update(
    update: &Update,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if !can_affect_rows(update.selection.as_ref()) {
        return;
    }
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    register_table_with_joins(&update.table, scopes, &mut seen, out);
    if let Some(from) = &update.from {
        let tables = match from {
            sqlparser::ast::UpdateTableFromKind::BeforeSet(t)
            | sqlparser::ast::UpdateTableFromKind::AfterSet(t) => t,
        };
        for table in tables {
            register_table_with_joins(table, scopes, &mut seen, out);
        }
    }
    let exprs = update
        .assignments
        .iter()
        .map(|assignment| &assignment.value)
        .chain(update.selection.iter());
    inspect_exprs(exprs, scopes, &mut seen, out);
    if let Some(returning) = update.returning.as_deref() {
        scopes.add_returning_aliases(Some("new"));
        let mut returning_seen = LookupHistory::new();
        inspect_exprs(
            returning_exprs(Some(returning)),
            scopes,
            &mut returning_seen,
            out,
        );
    }
    scopes.pop_frame();
}

pub(super) fn walk_delete(
    delete: &Delete,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if !can_affect_rows(delete.selection.as_ref()) {
        return;
    }
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    let from_tables = match &delete.from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables,
    };
    for table in from_tables.iter().chain(delete.using.iter().flatten()) {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    inspect_exprs(delete.selection.iter(), scopes, &mut seen, out);
    if let Some(returning) = delete.returning.as_deref() {
        scopes.add_returning_aliases(Some("old"));
        inspect_exprs(returning_exprs(Some(returning)), scopes, &mut seen, out);
    }
    scopes.pop_frame();
}

fn can_affect_rows(predicate: Option<&Expr>) -> bool {
    predicate.and_then(literal_condition) != Some(false)
}
