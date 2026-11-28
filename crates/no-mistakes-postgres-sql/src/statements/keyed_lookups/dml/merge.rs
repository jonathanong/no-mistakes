use super::super::conditions::literal_condition;
use super::super::join::register_table_factor;
use super::super::scope::ScopeStack;
use super::super::{inspect_exprs, returning_exprs, LookupHistory};
use super::can_affect_rows;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Expr, Merge, MergeAction, MergeClauseKind, MergeInsertKind, MergeUpdateKind, OutputClause,
};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::statements::keyed_lookups) fn walk_merge(
    merge: &Merge,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut base_seen = LookupHistory::new();
    register_table_factor(&merge.table, scopes, &mut base_seen, out);
    register_table_factor(&merge.source, scopes, &mut base_seen, out);
    inspect_exprs(std::iter::once(&*merge.on), scopes, &mut base_seen, out);
    let mut category_seen = BTreeMap::new();
    let mut closed_categories = BTreeSet::new();
    if !can_affect_rows(Some(&merge.on)) {
        closed_categories.insert(MergeClauseKind::Matched);
    }
    let mut modifies_rows = false;
    for clause in &merge.clauses {
        let category = match clause.clause_kind {
            MergeClauseKind::NotMatchedByTarget => MergeClauseKind::NotMatched,
            kind => kind,
        };
        let predicate = clause.predicate.as_ref();
        if closed_categories.contains(&category) || !can_affect_rows(predicate) {
            continue;
        }
        let pred_seen = category_seen
            .entry(category)
            .or_insert_with(|| base_seen.clone());
        if !pred_seen.is_reachable() {
            continue;
        }
        if let Some(predicate) = predicate {
            inspect_exprs(std::iter::once(predicate), scopes, pred_seen, out);
        }
        let mut action_seen = pred_seen.clone();
        if let Some(predicate) = predicate {
            let outer = scopes.snapshot();
            action_seen.restrict(predicate, &outer, true);
            pred_seen.restrict(predicate, &outer, false);
        }
        if action_seen.is_reachable() {
            modifies_rows |= !matches!(clause.action, MergeAction::DoNothing { .. });
            let mut exprs = Vec::new();
            collect_merge_action_exprs(&clause.action, &mut exprs);
            inspect_exprs(exprs, scopes, &mut action_seen, out);
        }
        if predicate.is_none() || predicate.and_then(literal_condition) == Some(true) {
            closed_categories.insert(category);
        }
    }
    if modifies_rows {
        if let Some(OutputClause::Returning { select_items, .. }) = &merge.output {
            scopes.add_returning_aliases(None);
            let mut returning_seen = LookupHistory::new();
            inspect_exprs(
                returning_exprs(Some(select_items)),
                scopes,
                &mut returning_seen,
                out,
            );
        }
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
