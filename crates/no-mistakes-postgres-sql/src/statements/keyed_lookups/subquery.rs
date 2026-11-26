use super::builtins::AGGREGATE_FUNCTIONS;
use super::expr_norm::normalize_expr;
use super::join::record_lookup;
use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::ScopeSnapshot;
use crate::codebase::postgres::idents::{
    ident_key, object_name_identity, relation_suffix_name, unwrap_expr,
};
use crate::codebase::postgres::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Expr, GroupByExpr, LimitClause, ObjectNamePart, OrderByKind, Query, Select, SetExpr, Spanned,
    TableFactor, Visit, Visitor,
};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

pub(super) fn inspect_expr(
    expr: &Expr,
    outer: &ScopeSnapshot,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    match unwrap_expr(expr) {
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
            if let Some(operand) = operand {
                inspect_expr(operand, outer, seen, out);
            }
            let mut cond_seen = seen.clone();
            for (index, branch) in conditions.iter().enumerate() {
                if index == 0 {
                    inspect_expr(&branch.condition, outer, seen, out);
                    cond_seen = seen.clone();
                } else {
                    inspect_expr(&branch.condition, outer, &mut cond_seen, out);
                }
                let mut arm_seen = cond_seen.clone();
                inspect_expr(&branch.result, outer, &mut arm_seen, out);
            }
            if let Some(else_expr) = else_result {
                let mut else_seen = cond_seen;
                inspect_expr(else_expr, outer, &mut else_seen, out);
            }
        }
        other => {
            crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
                inspect_expr(child, outer, seen, out);
            });
        }
    }
}

fn extract_scalar_subquery(
    query: &Query,
    outer: &ScopeSnapshot,
) -> Option<(LookupKey, String, usize, usize)> {
    if query.with.is_some() || query.fetch.is_some() {
        return None;
    }
    let limit = normalize_limit(query.limit_clause.as_ref(), outer)?;
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
    if alias.as_ref().is_some_and(|a| !a.columns.is_empty()) {
        return None;
    }
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
    let (key, key_display) = extract_lookup_key(
        &relation,
        &self_qualifier,
        true,
        selection,
        outer,
        order_by,
        limit,
    )?;
    let start = select.select_token.0.span.start;
    let fallback = query.span().start;
    let at = if start.line > 0 { start } else { fallback };
    Some((
        key,
        key_display,
        at.line.max(1) as usize,
        at.column.max(1) as usize,
    ))
}

fn normalize_limit(limit: Option<&LimitClause>, outer: &ScopeSnapshot) -> Option<Option<String>> {
    match limit {
        None => Some(None),
        Some(LimitClause::LimitOffset {
            limit: None,
            offset: None,
            ..
        }) => Some(None),
        Some(LimitClause::LimitOffset {
            limit: Some(expr),
            offset: None,
            ..
        }) => Some(Some(normalize_expr(expr, None, false, outer)?)),
        _ => None,
    }
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
