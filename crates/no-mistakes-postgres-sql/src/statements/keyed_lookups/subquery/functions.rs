use super::super::expr_norm::ExprNormalizer;
use super::super::function_clauses::{
    coalesce_arguments, filter_skips_inputs, row_window_value, split_coalesce_arguments,
    visit_eager_functions, visit_function_inputs, visit_ordered_set_arguments,
};
use super::super::scope::ScopeSnapshot;
use super::super::select_exprs::{target_identity, SelectTargets};
use super::super::windows::window_expressions;
use super::super::LookupHistory;
use super::inspect;
use crate::idents::function_arg_expr;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, Function, FunctionArg, WindowType};

pub(super) fn inspect_function(
    expr: &Expr,
    function: &Function,
    outer: &ScopeSnapshot,
    targets: &SelectTargets<'_>,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    skip_eager: bool,
) {
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
        let window_filter = function.over.is_some();
        if window_filter {
            visit_function_inputs(function, &mut |arg| {
                visit_eager_functions(arg, &mut |eager| {
                    inspect(eager, input_scope, targets, seen, out, false);
                });
            });
        }
        let mut filtered = seen.clone();
        filtered.mark_filtered(key);
        filtered.restrict(filter, input_scope, true);
        inspect_function_inputs(
            function,
            input_scope,
            targets,
            &mut filtered,
            out,
            skip_eager || window_filter,
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
            if !skip_eager {
                visit_eager_functions(expr, &mut |eager| {
                    inspect(eager, scope, targets, seen, out, false);
                });
            }
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
    let mut reachable = seen.clone();
    for argument in evaluated.iter().filter_map(function_arg_expr) {
        inspect(argument, outer, targets, &mut reachable, out, true);
        reachable.restrict_to_null(argument, outer);
        if !reachable.is_reachable() {
            break;
        }
    }
    seen.merge(reachable);
}
