use super::conditions::{boolean_result_is_fixed, case_condition};
use super::function_clauses::{is_eager, visit_eager_functions};
use super::history::record_lookup;
use super::scope::ScopeSnapshot;
use super::select_exprs::SelectTargets;
use super::LookupHistory;
use crate::idents::{unwrap_expr, visit_child_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::Expr;
mod functions;
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
        Expr::BinaryOp { left, op, right } if boolean_result_is_fixed(left, op, right) => {}
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
                if condition.is_none() {
                    arm_seen.restrict_case(operand.as_deref(), &branch.condition, outer, true);
                    cond_seen.restrict_case(operand.as_deref(), &branch.condition, outer, false);
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
            functions::inspect_function(expr, function, outer, targets, seen, out, skip_eager);
        }
        other => visit_child_exprs(other, &mut |expr| {
            inspect(expr, outer, targets, seen, out, skip_eager);
        }),
    }
}
