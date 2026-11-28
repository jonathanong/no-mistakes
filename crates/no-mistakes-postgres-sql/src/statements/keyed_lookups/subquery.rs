use super::conditions::{boolean_result_is_fixed, case_condition};
use super::expr_norm::ExprNormalizer;
use super::function_clauses::{
    coalesce_arguments, filter_skips_inputs, is_eager, row_window_value, split_coalesce_arguments,
    visit_eager_functions, visit_function_inputs, visit_ordered_set_arguments,
};
use super::join::record_lookup;
use super::scope::ScopeSnapshot;
use super::select_exprs::{target_identity, SelectTargets};
use super::windows::window_expressions;
use super::LookupHistory;
use crate::idents::{function_arg_expr, unwrap_expr, visit_child_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, FunctionArg, WindowType};
mod lookup;

use lookup::extract_scalar_subquery;

pub(super) fn inspect_expr(
    expr: &Expr,
    outer: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut LookupHistory,
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
        Expr::BinaryOp { left, op, right } if boolean_result_is_fixed(left, op, right) => {
            if !skip_eager {
                visit_eager_functions(expr, &mut |eager| {
                    inspect(eager, outer, targets, seen, out, false);
                });
            }
        }
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
                let condition = case_condition(operand.as_deref(), &branch.condition);
                if condition == Some(false) {
                    continue;
                }
                inspect(&branch.condition, outer, targets, &mut cond_seen, out, true);
                let mut arm_seen = cond_seen.clone();
                if operand.is_none() && condition.is_none() {
                    arm_seen.restrict(&branch.condition, outer, true);
                    cond_seen.restrict(&branch.condition, outer, false);
                }
                inspect(&branch.result, outer, targets, &mut arm_seen, out, true);
                seen.merge(arm_seen);
                if condition == Some(true) {
                    return;
                }
            }
            if let Some(else_expr) = else_result {
                inspect(else_expr, outer, targets, &mut cond_seen, out, true);
            }
            seen.merge(cond_seen);
        }
        Expr::Function(function) if skip_eager && is_eager(function) => {}
        Expr::Function(function) => {
            if function.over.is_some() && targets.skips_window_call(expr) {
                return;
            }
            if let Some(arguments) = coalesce_arguments(function) {
                inspect_coalesce(expr, arguments, outer, targets, seen, out, skip_eager);
                return;
            }
            visit_ordered_set_arguments(function, &mut |expr| {
                inspect(expr, outer, targets, seen, out, skip_eager);
            });
            let input_scope = targets.function_input_scope(function, outer);
            if let Some(filter) = &function.filter {
                inspect(filter, input_scope, targets, seen, out, skip_eager);
            }
            if let Some(filter) = function
                .filter
                .as_ref()
                .filter(|_| !filter_skips_inputs(function))
            {
                let key = ExprNormalizer::new(None, true, input_scope)
                    .normalize(filter)
                    .map(|_| target_identity(filter));
                let mut filtered = seen.clone();
                filtered.mark_filtered(key);
                filtered.restrict(filter, input_scope, true);
                inspect_function_inputs(
                    function,
                    input_scope,
                    targets,
                    &mut filtered,
                    out,
                    skip_eager,
                );
                seen.merge(filtered);
            } else {
                inspect_function_inputs(function, input_scope, targets, seen, out, skip_eager);
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

fn inspect_function_inputs(
    function: &sqlparser::ast::Function,
    scope: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    skip_eager: bool,
) {
    let selected = row_window_value(function);
    visit_function_inputs(function, &mut |expr| {
        if selected.is_some_and(|value| std::ptr::eq(expr, value)) {
            visit_eager_functions(expr, &mut |eager| {
                inspect(eager, scope, targets, seen, out, false);
            });
            inspect(expr, scope, targets, &mut LookupHistory::new(), out, true);
        } else {
            inspect(expr, scope, targets, seen, out, skip_eager);
        }
    });
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
