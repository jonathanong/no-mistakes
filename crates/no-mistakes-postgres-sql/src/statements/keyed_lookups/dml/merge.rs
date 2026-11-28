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
    let target_ids = scopes.binding_ids(0..scopes.binding_count());
    register_table_factor(&merge.source, scopes, &mut base_seen, out);
    inspect_exprs(std::iter::once(&*merge.on), scopes, &mut base_seen, out);
    let outer = scopes.snapshot();
    let mut category_seen = BTreeMap::new();
    let mut closed_categories = BTreeSet::new();
    if !can_affect_rows(Some(&merge.on)) {
        closed_categories.insert(MergeClauseKind::Matched);
    }
    let mut returning = Vec::new();
    let mut deletes_only = true;
    for clause in &merge.clauses {
        let category = match clause.clause_kind {
            MergeClauseKind::NotMatchedByTarget => MergeClauseKind::NotMatched,
            kind => kind,
        };
        let predicate = clause.predicate.as_ref();
        if closed_categories.contains(&category) || !can_affect_rows(predicate) {
            continue;
        }
        let pred_seen = category_seen.entry(category).or_insert_with(|| {
            let mut seen = base_seen.clone();
            if category == MergeClauseKind::Matched {
                seen.restrict(&merge.on, &outer, true);
            }
            seen
        });
        if !pred_seen.is_reachable() {
            continue;
        }
        if let Some(predicate) = predicate {
            inspect_exprs(std::iter::once(predicate), scopes, pred_seen, out);
        }
        let mut action_seen = pred_seen.clone();
        if let Some(predicate) = predicate {
            action_seen.restrict(predicate, &outer, true);
            pred_seen.restrict(predicate, &outer, false);
        }
        if action_seen.is_reachable() {
            if merge.output.is_some() {
                record_returning_context(
                    &clause.action,
                    &action_seen,
                    &target_ids,
                    &mut returning,
                    &mut deletes_only,
                );
            }
            let mut exprs = Vec::new();
            collect_merge_action_exprs(&clause.action, &mut exprs);
            inspect_exprs(exprs, scopes, &mut action_seen, out);
        }
        if predicate.is_none() || predicate.and_then(literal_condition) == Some(true) {
            closed_categories.insert(category);
        }
    }
    if let Some(OutputClause::Returning { select_items, .. }) = &merge.output {
        if !returning.is_empty() {
            let old_target = deletes_only.then(|| scopes.first_binding_id()).flatten();
            scopes.add_returning_aliases(old_target, None);
            inspect_returning(select_items, returning, scopes, out);
        }
    }
    scopes.pop_frame();
}

fn record_returning_context(
    action: &MergeAction,
    action_seen: &LookupHistory,
    target_ids: &[usize],
    returning: &mut Vec<LookupHistory>,
    deletes_only: &mut bool,
) {
    match action {
        MergeAction::DoNothing { .. } => {}
        MergeAction::Delete { .. } => returning.push(action_seen.guard_only()),
        _ => {
            returning.push(action_seen.without_bindings(target_ids));
            *deletes_only = false;
        }
    }
}

fn inspect_returning(
    items: &[sqlparser::ast::SelectItem],
    contexts: Vec<LookupHistory>,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let first_finding = out.len();
    for mut guards in contexts {
        let mut findings = Vec::new();
        inspect_exprs(
            returning_exprs(Some(items)),
            scopes,
            &mut guards,
            &mut findings,
        );
        for finding in findings {
            if !out[first_finding..].contains(&finding) {
                out.push(finding);
            }
        }
    }
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
