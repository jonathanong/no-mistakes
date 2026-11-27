use super::expr_norm::types::projection_expr_is_unproven;
use super::expr_norm::ExprNormalizer;
use super::join::record_lookup;
use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::ScopeSnapshot;
use super::select_exprs::ProjectionTargets;
use super::table::lookup_table;
use super::windows::window_expressions;
use crate::idents::{
    ident_key, object_name_identity, relation_suffix_name, unwrap_expr, visit_child_exprs,
    visit_shallow_exprs,
};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Expr, FunctionArguments, GroupByExpr, LimitClause, OrderByKind, Query, Select, SetExpr,
    Spanned, Visit, Visitor, WindowType,
};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

pub(super) fn inspect_expr(
    expr: &Expr,
    outer: &ScopeSnapshot,
    targets: &ProjectionTargets<'_>,
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
                inspect_expr(operand, outer, targets, seen, out);
            }
            let mut cond_seen = seen.clone();
            for (index, branch) in conditions.iter().enumerate() {
                if index == 0 {
                    inspect_expr(&branch.condition, outer, targets, seen, out);
                    cond_seen = seen.clone();
                } else {
                    inspect_expr(&branch.condition, outer, targets, &mut cond_seen, out);
                }
                let mut arm_seen = cond_seen.clone();
                inspect_expr(&branch.result, outer, targets, &mut arm_seen, out);
            }
            if let Some(else_expr) = else_result {
                let mut else_seen = cond_seen;
                inspect_expr(else_expr, outer, targets, &mut else_seen, out);
            }
        }
        other => {
            let mut visit = |child: &Expr| inspect_expr(child, outer, targets, seen, out);
            visit_child_exprs(other, &mut visit);
            if let Expr::Function(function) = other {
                for arguments in [&function.parameters, &function.args] {
                    if let FunctionArguments::List(list) = arguments {
                        visit_shallow_exprs(&list.clauses, &mut visit);
                    }
                }
                if let Some(WindowType::WindowSpec(spec)) = &function.over {
                    for (expr, can_reuse) in window_expressions(spec) {
                        if !can_reuse || !targets.reuses(expr) {
                            visit(expr);
                        }
                    }
                }
                visit_shallow_exprs(&function.within_group, &mut visit);
            }
        }
    }
}

fn extract_scalar_subquery(
    query: &Query,
    outer: &ScopeSnapshot,
) -> Option<(LookupKey, String, usize, usize)> {
    if query.with.is_some() || query.fetch.is_some() || !query.locks.is_empty() {
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
        || projection_is_unproven(select)
    {
        return None;
    }
    let (name, alias) = lookup_table(&select.from[0].relation)?;
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
        }) => Some(Some(
            ExprNormalizer::new(None, false, outer).normalize(expr)?,
        )),
        _ => None,
    }
}

fn empty_group_by(group_by: &GroupByExpr) -> bool {
    matches!(
        group_by,
        GroupByExpr::Expressions(exprs, modifiers) if exprs.is_empty() && modifiers.is_empty()
    )
}

fn projection_is_unproven(select: &Select) -> bool {
    struct CheckProjection;

    impl Visitor for CheckProjection {
        type Break = ();

        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            ControlFlow::Break(())
        }

        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if projection_expr_is_unproven(expr) {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }

    select.projection.visit(&mut CheckProjection).is_break()
}
