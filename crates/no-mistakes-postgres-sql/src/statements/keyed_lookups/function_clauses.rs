use super::select_exprs::target_identity;
use crate::fx::FxHashSet;
use crate::idents::{function_arg_expr, visit_shallow_exprs};
use sqlparser::ast::{Expr, FunctionArgumentClause, FunctionArguments};

pub(super) fn visit_argument_clauses(arguments: &FunctionArguments, visit: &mut impl FnMut(&Expr)) {
    let FunctionArguments::List(list) = arguments else {
        return;
    };
    for clause in &list.clauses {
        if let FunctionArgumentClause::OrderBy(order_by) = clause {
            let mut targets: FxHashSet<_> = list
                .args
                .iter()
                .filter_map(function_arg_expr)
                .map(target_identity)
                .collect();
            for item in order_by {
                if targets.insert(target_identity(&item.expr)) {
                    visit(&item.expr);
                }
            }
        } else {
            visit_shallow_exprs(clause, visit);
        }
    }
}
