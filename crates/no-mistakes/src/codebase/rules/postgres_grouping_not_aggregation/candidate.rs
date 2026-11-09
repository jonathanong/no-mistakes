use super::{expressions::*, scope::CatalogScope, walk};
use sqlparser::ast::*;
pub(super) fn matches(select: &Select, catalog: &CatalogScope<'_>) -> bool {
    let keys = groups(select);
    if keys.is_empty() {
        return false;
    }
    let Some(projected) = projection(select) else {
        return false;
    };
    if projected
        .iter()
        .any(|expr| !keys.iter().any(|key| bare(key) == bare(expr)))
    {
        return false;
    }
    let Some(Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    }) = select.having.as_ref().map(bare)
    else {
        return false;
    };
    let count_threshold = |count: &Expr, threshold: &Expr| {
        function(count, "count").and_then(args).is_some_and(|args| matches!(args.args.as_slice(),
            [FunctionArg::Unnamed(FunctionArgExpr::Wildcard)]) || args.duplicate_treatment == Some(DuplicateTreatment::Distinct)
            && matches!(args.args.as_slice(), [FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))] if column(expr).is_some() && !catalog.parameter(expr)))
            && (catalog.pinned(threshold) || function(threshold, "cardinality").is_some())
    };
    let count = if count_threshold(left, right) {
        left
    } else if count_threshold(right, left) {
        right
    } else {
        return false;
    };
    let counted = function(count, "count").and_then(argument).and_then(column);
    if counted.as_ref().is_some_and(|(_, name)| {
        keys.iter()
            .any(|key| column(key).is_some_and(|(_, key)| key == *name))
    }) {
        return false;
    }
    let [from] = select.from.as_slice() else {
        return false;
    };
    if !from.joins.is_empty() {
        return false;
    }
    match &from.relation {
        TableFactor::Derived { subquery, alias, .. } if alias.as_ref().is_none_or(|alias| alias.columns.is_empty()) => union_pool(&subquery.body, catalog),
        factor => walk::relation(factor).is_some_and(|relation| catalog.relation(&relation.name).is_some()
            && conjuncts(select.selection.as_ref()).iter().any(|predicate| matches!(predicate,
                Expr::AnyOp { left, compare_op: BinaryOperator::Eq, right, .. } if column(left).is_some_and(|(_, name)| counted.as_ref().is_none_or(|(_, counted)| *counted == name)) && !catalog.parameter(left) && array_parameter(right, catalog)))),
    }
}
fn union_pool(body: &SetExpr, catalog: &CatalogScope<'_>) -> bool {
    match body {
        SetExpr::SetOperation {
            op: SetOperator::Union,
            left,
            right,
            ..
        } => arm(left, catalog) && arm(right, catalog),
        SetExpr::Query(query) if query.limit_clause.is_none() && query.fetch.is_none() => {
            union_pool(&query.body, catalog)
        }
        _ => false,
    }
}
fn arm(body: &SetExpr, catalog: &CatalogScope<'_>) -> bool {
    match body {
        SetExpr::Select(select) => walk::table_columns(select).is_some_and(|relations| {
            relations
                .iter()
                .all(|relation| catalog.relation(&relation.name).is_some())
        }),
        other => union_pool(other, catalog),
    }
}
fn array_parameter(expr: &Expr, catalog: &CatalogScope<'_>) -> bool {
    if catalog.parameter(expr) {
        return true;
    }
    match bare(expr) {
        Expr::Value(value) => matches!(value.value, Value::Placeholder(_)),
        Expr::Cast { expr, .. } => array_parameter(expr, catalog),
        _ => false,
    }
}
