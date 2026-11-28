use super::super::expr_norm::function::pure_name;
use super::super::select_item_expr;
use crate::idents::unwrap_expr;
use sqlparser::ast::{
    visit_expressions, Distinct, Expr, GroupByExpr, Query, Select, Visit, Visitor,
};
use std::ops::ControlFlow;

pub(super) fn select_has_plain_outputs(select: &Select) -> bool {
    select.having.is_none()
        && matches!(
            select.distinct,
            None | Some(Distinct::All | Distinct::Distinct)
        )
        && matches!(&select.group_by, GroupByExpr::Expressions(exprs, modifiers)
            if exprs.is_empty() && modifiers.is_empty())
        && select
            .projection
            .iter()
            .filter_map(select_item_expr)
            .all(plain_output)
}

pub(in crate::statements::keyed_lookups) fn plain_output(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) => true,
        Expr::Subquery(query) => visit_expressions(query, possible_outer_aggregate).is_continue(),
        _ => false,
    }
}

fn possible_outer_aggregate(expr: &Expr) -> ControlFlow<()> {
    let Expr::Function(function) = expr else {
        return ControlFlow::Continue(());
    };
    if function.over.is_some() || pure_name(function).is_some() {
        return ControlFlow::Continue(());
    }
    visit_expressions(function, |input| match input {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => ControlFlow::Break(()),
        _ => ControlFlow::Continue(()),
    })
}

pub(in crate::statements::keyed_lookups) fn per_row_output(expr: &Expr) -> bool {
    if matches!(
        unwrap_expr(expr),
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) | Expr::Subquery(_)
    ) {
        return plain_output(expr);
    }
    #[derive(Default)]
    struct AggregateOwners {
        depth: usize,
    }

    impl Visitor for AggregateOwners {
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
            if self.depth > 0 {
                return possible_outer_aggregate(expr);
            }
            if matches!(expr, Expr::Function(function)
                if function.over.is_none() && pure_name(function).is_none())
            {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }

    expr.visit(&mut AggregateOwners::default()).is_continue()
}

pub(super) fn grouping_needs_rows(group_by: &GroupByExpr) -> bool {
    matches!(group_by, GroupByExpr::Expressions(exprs, _) if exprs.iter().any(nonempty_group_key))
}

fn nonempty_group_key(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Rollup(_) | Expr::Cube(_) => false,
        Expr::GroupingSets(options) => options.iter().all(|set| set.iter().any(nonempty_group_key)),
        Expr::Tuple(keys) => keys.iter().any(nonempty_group_key),
        _ => true,
    }
}
