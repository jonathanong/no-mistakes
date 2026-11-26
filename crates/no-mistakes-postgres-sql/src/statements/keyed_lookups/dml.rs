use super::join::{register_table_factor, register_table_with_joins};
use super::scope::ScopeStack;
use super::{inspect_exprs, returning_exprs};
use crate::codebase::postgres::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Delete, Expr, FromTable, Insert, Merge, MergeAction, MergeInsertKind, MergeUpdateKind,
    OutputClause, TableObject, Update,
};
use std::collections::BTreeMap;

pub(super) fn walk_insert(
    insert: &Insert,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some(source) = insert.source.as_deref() {
        super::walk_query(source, scopes, out);
    }
    if let Some(returning) = insert.returning.as_deref() {
        scopes.push_frame();
        if let TableObject::TableName(name) = &insert.table {
            scopes.add_table_alias(name, insert.table_alias.as_ref().map(|a| &a.alias));
        }
        let mut seen = BTreeMap::new();
        inspect_exprs(returning_exprs(Some(returning)), scopes, &mut seen, out);
        scopes.pop_frame();
    }
}

pub(super) fn walk_update(
    update: &Update,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut seen = BTreeMap::new();
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
        let mut returning_seen = BTreeMap::new();
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
    scopes.push_frame();
    let mut seen = BTreeMap::new();
    let from_tables = match &delete.from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables,
    };
    for table in from_tables.iter().chain(delete.using.iter().flatten()) {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    let exprs = delete
        .selection
        .iter()
        .chain(returning_exprs(delete.returning.as_deref()));
    inspect_exprs(exprs, scopes, &mut seen, out);
    scopes.pop_frame();
}

pub(super) fn walk_merge(
    merge: &Merge,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut base_seen = BTreeMap::new();
    register_table_factor(&merge.table, scopes, out);
    register_table_factor(&merge.source, scopes, out);
    inspect_exprs(std::iter::once(&*merge.on), scopes, &mut base_seen, out);
    let mut category_seen: BTreeMap<String, _> = BTreeMap::new();
    for clause in &merge.clauses {
        let pred_seen = category_seen
            .entry(clause.clause_kind.to_string())
            .or_insert_with(|| base_seen.clone());
        if let Some(predicate) = &clause.predicate {
            inspect_exprs(std::iter::once(predicate), scopes, pred_seen, out);
        }
        let mut action_seen = pred_seen.clone();
        let mut exprs = Vec::new();
        collect_merge_action_exprs(&clause.action, &mut exprs);
        inspect_exprs(exprs, scopes, &mut action_seen, out);
    }
    if let Some(OutputClause::Returning { select_items, .. }) = &merge.output {
        let mut returning_seen = BTreeMap::new();
        inspect_exprs(
            returning_exprs(Some(select_items)),
            scopes,
            &mut returning_seen,
            out,
        );
    }
    scopes.pop_frame();
}

fn collect_merge_action_exprs<'a>(action: &'a MergeAction, out: &mut Vec<&'a Expr>) {
    match action {
        MergeAction::Update(update) => {
            if let MergeUpdateKind::Set(assignments) = &update.kind {
                out.extend(assignments.iter().map(|a| &a.value));
            }
        }
        MergeAction::Insert(insert) => {
            if let MergeInsertKind::Values(values) = &insert.kind {
                for row in &values.rows {
                    out.extend(&row.content);
                }
            }
        }
        MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
    }
}
