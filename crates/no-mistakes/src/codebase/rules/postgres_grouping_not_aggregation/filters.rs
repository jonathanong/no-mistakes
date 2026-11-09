use super::{
    analyze::{finding, Finding},
    expressions::*,
    walk,
};
use sqlparser::ast::*;
use std::collections::BTreeSet;
use std::ops::ControlFlow;
mod consumers;
mod fanout;
use fanout::fanout;
mod join_keys;
pub(super) use consumers::cte_consumers;
use join_keys::join_covers;
pub(super) fn select_filters(
    select: &Select,
    catalog: &super::scope::CatalogScope<'_>,
    order_owners: &BTreeSet<String>,
    out: &mut Vec<Finding>,
) {
    for from in &select.from {
        for join in &from.joins {
            if walk::on(join).is_none() {
                continue;
            }
            let TableFactor::Derived {
                subquery,
                alias: Some(alias),
                lateral: false,
                ..
            } = &join.relation
            else {
                continue;
            };
            let name = ident(&alias.name);
            let Some(inner) = super::expressions::select(subquery) else {
                continue;
            };
            let Some(names) = output_names(inner) else {
                continue;
            };
            let used = |expr: &Expr| uses_output(expr, &name, &names);
            if order_owners.contains(&name)
                || names
                    .iter()
                    .any(|col| order_owners.contains(&format!(":{col}")))
                || !join_covers(join, subquery, &name)
            {
                continue;
            }
            if from
                .joins
                .iter()
                .any(|other| !std::ptr::eq(other, join) && walk::on(other).is_some_and(used))
            {
                continue;
            }
            if inner.distinct != Some(Distinct::Distinct)
                || subquery.limit_clause.is_some()
                || subquery.fetch.is_some()
                || !alias.columns.is_empty()
                || groups(inner).len() > 0
            {
                continue;
            }
            let Some(projected) = projection(select) else {
                continue;
            };
            if projected.iter().any(|expr| used(expr))
                || groups(select).iter().any(|expr| used(expr))
                || select.having.as_ref().is_some_and(|expr| used(expr))
                || select.selection.as_ref().is_some_and(|expr| used(expr))
            {
                continue;
            }
            if walk::table_columns(inner).is_some_and(|relations| {
                relations
                    .iter()
                    .all(|r| catalog.relation(&r.name).is_some())
            }) {
                out.push(finding(inner, "distinct-join-filter", "joined DISTINCT rows are used only to test membership; replace the filter join with EXISTS"));
            }
        }
    }
    for expr in [&select.selection, &select.having].into_iter().flatten() {
        membership(expr, &mut |query| {
            if let Some(inner) = super::expressions::select(query) {
                if inner.distinct == Some(Distinct::Distinct)
                    && query.limit_clause.is_none()
                    && query.fetch.is_none()
                    && super::evidence::catalogued(inner, catalog)
                {
                    out.push(finding(inner, "distinct-join-filter", "duplicates do not affect membership or existence; remove DISTINCT from the membership subquery"));
                }
            }
        });
    }
    if fanout(select, catalog, order_owners) {
        out.push(finding(select, "distinct-join-filter", "COUNT(DISTINCT) masks filter-side join fan-out; replace those joins with EXISTS and remove DISTINCT from COUNT(expression)"));
    }
}
pub(super) fn membership(expr: &Expr, emit: &mut impl FnMut(&Query)) {
    let _: ControlFlow<()> = visit_expressions(expr, |expr| {
        match bare(expr) {
            Expr::InSubquery { subquery, .. } | Expr::Exists { subquery, .. } => emit(subquery),
            Expr::AnyOp { right, .. } | Expr::AllOp { right, .. } => match bare(right) {
                Expr::Subquery(query) => emit(query),
                Expr::Function(function)
                    if function.name.to_string().eq_ignore_ascii_case("array") =>
                {
                    if let FunctionArguments::Subquery(query) = &function.args {
                        emit(query);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        ControlFlow::Continue(())
    });
}
