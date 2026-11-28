use super::select_exprs::target_identity;
use crate::fx::FxHashSet;
use crate::idents::{function_arg_expr, ident_key, unwrap_expr, visit_shallow_exprs};
use sqlparser::ast::{
    DuplicateTreatment, Expr, Function, FunctionArg, FunctionArgumentClause, FunctionArguments,
    Value,
};

pub(super) fn coalesce_arguments(function: &Function) -> Option<&[FunctionArg]> {
    let [name] = function.name.0.as_slice() else {
        return None;
    };
    let name = name.as_ident()?;
    if name.quote_style.is_some()
        || !name.value.eq_ignore_ascii_case("coalesce")
        || is_eager(function)
    {
        return None;
    }
    match &function.args {
        FunctionArguments::List(list) => Some(&list.args),
        _ => None,
    }
}

pub(super) fn visit_function_inputs(function: &Function, visit: &mut impl FnMut(&Expr)) {
    visit_arguments(&function.parameters, usize::MAX, visit);
    let argument_limit = if has_optional_window_default(function) {
        2
    } else {
        usize::MAX
    };
    visit_arguments(&function.args, argument_limit, visit);
    visit_shallow_exprs(&function.within_group, visit);
}

pub(super) fn split_coalesce_arguments(
    arguments: &[FunctionArg],
) -> (&[FunctionArg], &[FunctionArg]) {
    let evaluated = arguments.iter().position(|argument| {
        matches!(function_arg_expr(argument).map(unwrap_expr), Some(Expr::Value(value))
            if !matches!(value.value, Value::Null | Value::Placeholder(_)))
    });
    arguments.split_at(evaluated.map_or(arguments.len(), |index| index + 1))
}

fn has_optional_window_default(function: &Function) -> bool {
    function.over.is_some()
        && match function.name.0.as_slice() {
            [name] => name.as_ident(),
            [schema, name]
                if schema
                    .as_ident()
                    .is_some_and(|s| ident_key(s) == "pg_catalog") =>
            {
                name.as_ident()
            }
            _ => None,
        }
        .is_some_and(|name| matches!(ident_key(name).as_str(), "lag" | "lead"))
}

fn visit_arguments(
    arguments: &FunctionArguments,
    argument_limit: usize,
    visit: &mut impl FnMut(&Expr),
) {
    let FunctionArguments::List(list) = arguments else {
        return;
    };
    for arg in list
        .args
        .iter()
        .take(argument_limit)
        .filter_map(function_arg_expr)
    {
        visit(arg);
    }
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
    if function.filter.is_some() || function.over.is_some() || !function.within_group.is_empty() {
        return true;
    }
    if let FunctionArguments::List(list) = &function.args {
        if list.duplicate_treatment == Some(DuplicateTreatment::Distinct)
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
