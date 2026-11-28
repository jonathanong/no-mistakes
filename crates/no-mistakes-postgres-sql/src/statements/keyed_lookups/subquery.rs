use super::function_clauses::{is_eager, visit_argument_clauses, visit_eager_functions};
use super::join::record_lookup;
use super::normalize::LookupKey;
use super::scope::ScopeSnapshot;
use super::select_exprs::SelectTargets;
use super::windows::window_expressions;
use crate::idents::{unwrap_expr, visit_child_exprs, visit_shallow_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, WindowType};
use std::collections::BTreeMap;
mod lookup;

use lookup::extract_scalar_subquery;

pub(super) fn inspect_expr(
    expr: &Expr,
    outer: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    inspect(expr, outer, targets, seen, out, false);
}

fn inspect(
    expr: &Expr,
    outer: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut super::LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    skip_eager: bool,
) {
    match unwrap_expr(expr) {
        Expr::Subquery(query) => {
            if let Some((key, key_display, line, column)) = extract_scalar_subquery(query, outer) {
                record_lookup(key, key_display, line, column, seen, out);
            }
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if !skip_eager {
                visit_eager_functions(expr, &mut |eager| {
                    inspect(eager, outer, targets, seen, out, false);
                });
            }
            if let Some(operand) = operand {
                inspect(operand, outer, targets, seen, out, true);
            }
            let mut cond_seen = seen.clone();
            for branch in conditions {
                inspect(&branch.condition, outer, targets, &mut cond_seen, out, true);
                let mut arm_seen = cond_seen.clone();
                inspect(&branch.result, outer, targets, &mut arm_seen, out, true);
                merge_history(seen, arm_seen);
            }
            if let Some(else_expr) = else_result {
                inspect(else_expr, outer, targets, &mut cond_seen, out, true);
            }
            merge_history(seen, cond_seen);
        }
        Expr::Function(function) if skip_eager && is_eager(function) => {}
        other => {
            let mut visit = |child: &Expr| inspect(child, outer, targets, seen, out, skip_eager);
            visit_child_exprs(other, &mut visit);
            if let Expr::Function(function) = other {
                for arguments in [&function.parameters, &function.args] {
                    visit_argument_clauses(arguments, &mut visit);
                }
                match &function.over {
                    Some(WindowType::WindowSpec(spec)) if !targets.reuses_window(spec) => {
                        for (expr, can_reuse) in window_expressions(spec) {
                            if !can_reuse || !targets.reuses(expr) {
                                visit(expr);
                            }
                        }
                    }
                    _ => {}
                }
                visit_shallow_exprs(&function.within_group, &mut visit);
            }
        }
    }
}

fn merge_history(seen: &mut super::LookupHistory, branch: super::LookupHistory) {
    for (key, site) in branch {
        seen.entry(key).or_insert(site);
    }
}
