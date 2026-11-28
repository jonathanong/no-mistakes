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

pub(super) fn is_eager(function: &sqlparser::ast::Function) -> bool {
    use crate::idents::ident_key;
    if function.filter.is_some() || function.over.is_some() || !function.within_group.is_empty() {
        return true;
    }
    if let FunctionArguments::List(list) = &function.args {
        if list.duplicate_treatment.is_some()
            || list
                .clauses
                .iter()
                .any(|clause| matches!(clause, FunctionArgumentClause::OrderBy(_)))
        {
            return true;
        }
    }
    let [schema, name] = function.name.0.as_slice() else {
        return false;
    };
    schema
        .as_ident()
        .is_some_and(|schema| ident_key(schema) == "pg_catalog")
        && name.as_ident().is_some_and(|name| {
            matches!(
                ident_key(name).as_str(),
                "min"
                    | "max"
                    | "sum"
                    | "avg"
                    | "count"
                    | "array_agg"
                    | "string_agg"
                    | "bool_and"
                    | "bool_or"
                    | "json_agg"
                    | "jsonb_agg"
            )
        })
}

pub(super) fn visit_eager_functions(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    use sqlparser::ast::{Query, Visit, Visitor};
    use std::ops::ControlFlow;

    struct EagerFunctions<'a, F> {
        visit: &'a mut F,
        depth: usize,
    }

    impl<F: FnMut(&Expr)> Visitor for EagerFunctions<'_, F> {
        type Break = ();

        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth += 1;
            ControlFlow::Continue(())
        }

        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }

        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if matches!(expr, Expr::Function(function) if is_eager(function)) {
                if self.depth == 0 {
                    (self.visit)(expr);
                }
                self.depth += 1;
            }
            ControlFlow::Continue(())
        }

        fn post_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if matches!(expr, Expr::Function(function) if is_eager(function)) {
                self.depth -= 1;
            }
            ControlFlow::Continue(())
        }
    }

    let _ = expr.visit(&mut EagerFunctions { visit, depth: 0 });
}
