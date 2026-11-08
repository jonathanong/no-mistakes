use sqlparser::ast::{visit_expressions, Expr};
use std::ops::ControlFlow;

pub(super) fn configured_function(expr: &Expr, functions: &[String]) -> bool {
    let Expr::Function(function) = crate::codebase::postgres::idents::unwrap_expr(expr) else {
        return false;
    };
    let name = crate::codebase::postgres::idents::object_name_identity(&function.name);
    functions.contains(&name) && scalar(expr, functions)
}

pub(super) fn scalar(expr: &Expr, functions: &[String]) -> bool {
    visit_expressions(expr, |child| {
        let rejected = match child {
            Expr::Identifier(_)
            | Expr::CompoundIdentifier(_)
            | Expr::Subquery(_)
            | Expr::Exists { .. }
            | Expr::InSubquery { .. } => true,
            Expr::Function(function) => {
                let name = crate::codebase::postgres::idents::object_name_identity(&function.name);
                function.over.is_some()
                    || function.filter.is_some()
                    || !(functions.contains(&name) || temporal(&name))
            }
            _ => false,
        };
        if rejected {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_continue()
}

pub(super) fn uses_configured_function(expr: &Expr, functions: &[String]) -> bool {
    visit_expressions(expr, |child| {
        if let Expr::Function(function) = child {
            let name = crate::codebase::postgres::idents::object_name_identity(&function.name);
            if functions.contains(&name) && !temporal(&name) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    })
    .is_break()
}

fn temporal(name: &str) -> bool {
    matches!(
        name.strip_prefix("pg_catalog.").unwrap_or(name),
        "now" | "current_timestamp" | "current_date" | "localtimestamp"
    )
}
