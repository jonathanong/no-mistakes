use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::ScopeSnapshot;
use crate::codebase::postgres::idents::{
    ident_key, object_name_identity, relation_suffix_name, unwrap_expr,
};
use sqlparser::ast::{
    Expr, GroupByExpr, LimitClause, ObjectNamePart, OrderByKind, Query, Select, SetExpr, Spanned,
    TableFactor, Visit, Visitor,
};
use std::ops::ControlFlow;

pub(super) struct SubqueryLookup {
    pub(super) key: LookupKey,
    pub(super) key_display: String,
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) conditional: bool,
}

const AGGREGATE_FUNCTIONS: &[&str] = &[
    "count",
    "sum",
    "avg",
    "min",
    "max",
    "array_agg",
    "json_agg",
    "jsonb_agg",
    "json_object_agg",
    "jsonb_object_agg",
    "string_agg",
    "bool_and",
    "bool_or",
    "every",
    "bit_and",
    "bit_or",
    "bit_xor",
    "xmlagg",
    "percentile_cont",
    "percentile_disc",
    "mode",
    "stddev",
    "stddev_pop",
    "stddev_samp",
    "variance",
    "var_pop",
    "var_samp",
    "corr",
    "covar_pop",
    "covar_samp",
    "range_agg",
    "range_intersect_agg",
    "any_value",
];

pub(super) fn collect_expr_lookups(
    expr: &Expr,
    outer: &ScopeSnapshot,
    conditional: bool,
    out: &mut Vec<SubqueryLookup>,
) {
    match unwrap_expr(expr) {
        Expr::Subquery(query) => {
            if let Some(lookup) = extract_scalar_subquery(query, outer, conditional) {
                out.push(lookup);
            }
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(operand) = operand {
                collect_expr_lookups(operand, outer, conditional, out);
            }
            for (index, branch) in conditions.iter().enumerate() {
                let cond_flag = conditional || index > 0;
                collect_expr_lookups(&branch.condition, outer, cond_flag, out);
                collect_expr_lookups(&branch.result, outer, true, out);
            }
            if let Some(else_expr) = else_result {
                collect_expr_lookups(else_expr, outer, true, out);
            }
        }
        other => {
            crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
                collect_expr_lookups(child, outer, conditional, out);
            });
        }
    }
}

fn extract_scalar_subquery(
    query: &Query,
    outer: &ScopeSnapshot,
    conditional: bool,
) -> Option<SubqueryLookup> {
    if query.with.is_some() || has_offset(query.limit_clause.as_ref()) {
        return None;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    if select.distinct.is_some()
        || select.having.is_some()
        || !empty_group_by(&select.group_by)
        || select.from.len() != 1
        || !select.from[0].joins.is_empty()
        || projection_has_aggregate(select)
    {
        return None;
    }
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = &select.from[0].relation
    else {
        return None;
    };
    let selection = select.selection.as_ref()?;
    let relation = object_name_identity(name);
    let self_qualifier = match alias {
        Some(alias) => ident_key(&alias.name),
        None => relation_suffix_name(&relation),
    };
    let order_by = match query.order_by.as_ref().map(|order| &order.kind) {
        Some(OrderByKind::Expressions(exprs)) => exprs.as_slice(),
        Some(OrderByKind::All(_)) => return None,
        None => &[],
    };
    let (key, key_display) =
        extract_lookup_key(&relation, &self_qualifier, true, selection, outer, order_by)?;
    let start = select.select_token.0.span.start;
    let fallback = query.span().start;
    let at = if start.line > 0 { start } else { fallback };
    Some(SubqueryLookup {
        key,
        key_display,
        line: at.line.max(1) as usize,
        column: at.column.max(1) as usize,
        conditional,
    })
}

fn has_offset(limit: Option<&LimitClause>) -> bool {
    matches!(
        limit,
        Some(
            LimitClause::LimitOffset {
                offset: Some(_),
                ..
            } | LimitClause::OffsetCommaLimit { .. }
        )
    )
}

fn empty_group_by(group_by: &GroupByExpr) -> bool {
    matches!(
        group_by,
        GroupByExpr::Expressions(exprs, modifiers) if exprs.is_empty() && modifiers.is_empty()
    )
}

fn projection_has_aggregate(select: &Select) -> bool {
    struct AggregateFinder(bool);
    impl Visitor for AggregateFinder {
        type Break = ();
        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::Function(function) = unwrap_expr(expr) {
                let is_agg = function.over.is_some()
                    || function.filter.is_some()
                    || !function.within_group.is_empty()
                    || matches!(
                        function.name.0.last(),
                        Some(ObjectNamePart::Identifier(ident))
                            if is_aggregate_name(&ident.value)
                    );
                if is_agg {
                    self.0 = true;
                    return ControlFlow::Break(());
                }
            }
            ControlFlow::Continue(())
        }
    }
    let mut finder = AggregateFinder(false);
    let _ = select.projection.visit(&mut finder);
    finder.0
}

fn is_aggregate_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    AGGREGATE_FUNCTIONS.contains(&lower.as_str()) || lower.starts_with("regr_")
}
