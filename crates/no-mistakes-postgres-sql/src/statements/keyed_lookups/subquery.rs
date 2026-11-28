use super::expr_norm::ExprNormalizer;
use super::function_clauses::{
    coalesce_arguments, is_eager, split_coalesce_arguments, visit_eager_functions,
    visit_function_inputs,
};
use super::join::record_lookup;
use super::normalize::LookupKey;
use super::scope::ScopeSnapshot;
use super::select_exprs::{target_identity, SelectTargets};
use super::windows::window_expressions;
use crate::idents::{function_arg_expr, unwrap_expr, visit_child_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, FunctionArg, WindowType};
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
        Expr::Function(function) => {
            if let Some(arguments) = coalesce_arguments(function) {
                inspect_coalesce(expr, arguments, outer, targets, seen, out, skip_eager);
                return;
            }
            let input_scope = targets.function_input_scope(function, outer);
            if let Some(filter) = &function.filter {
                inspect(filter, input_scope, targets, seen, out, skip_eager);
                let key = ExprNormalizer::new(None, true, input_scope)
                    .normalize(filter)
                    .map(|_| target_identity(filter));
                let mut filtered = key
                    .as_ref()
                    .map_or_else(BTreeMap::new, |key| targets.take_filter_history(key));
                visit_function_inputs(function, &mut |expr| {
                    inspect(expr, input_scope, targets, &mut filtered, out, skip_eager);
                });
                if let Some(key) = key {
                    targets.save_filter_history(key, filtered);
                }
            } else {
                visit_function_inputs(function, &mut |expr| {
                    inspect(expr, input_scope, targets, seen, out, skip_eager);
                });
            }
            if let Some(WindowType::WindowSpec(spec)) = &function.over {
                if let Some(needs_frame) = targets.take_inline_frame(spec) {
                    for (expr, can_reuse) in window_expressions(spec, needs_frame) {
                        if !can_reuse {
                            inspect(expr, outer, targets, seen, out, skip_eager);
                        } else if !targets.reuses(expr) {
                            let expr = targets.filter_grouped(expr);
                            inspect(&expr, outer, targets, seen, out, skip_eager);
                        }
                    }
                }
            }
        }
        other => visit_child_exprs(other, &mut |expr| {
            inspect(expr, outer, targets, seen, out, skip_eager);
        }),
    }
}

fn inspect_coalesce(
    expr: &Expr,
    arguments: &[FunctionArg],
    outer: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut super::LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    skip_eager: bool,
) {
    if !skip_eager {
        visit_eager_functions(expr, &mut |eager| {
            inspect(eager, outer, targets, seen, out, false);
        });
    }
    let (evaluated, _) = split_coalesce_arguments(arguments);
    for argument in evaluated.iter().filter_map(function_arg_expr) {
        inspect(argument, outer, targets, seen, out, true);
    }
}

fn merge_history(seen: &mut super::LookupHistory, branch: super::LookupHistory) {
    for (key, site) in branch {
        seen.entry(key).or_insert(site);
    }
}
