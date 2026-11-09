use super::{
    analyze::{finding, Finding},
    expressions::*,
    walk,
};
use sqlparser::ast::*;
mod counts;
pub(super) fn select_tests(
    select: &Select,
    catalog: &super::scope::CatalogScope<'_>,
    out: &mut Vec<Finding>,
) {
    counts::tests(select, catalog, out);
    if let Some(message) = grouped_threshold(select, catalog) {
        out.push(finding(select, "aggregate-existence-test", message));
    }
}
fn grouped_threshold(
    select: &Select,
    catalog: &super::scope::CatalogScope<'_>,
) -> Option<&'static str> {
    let Some(Expr::BinaryOp {
        left,
        op: BinaryOperator::Or,
        right,
    }) = select.having.as_ref().map(bare)
    else {
        return None;
    };
    let check = |absence: &Expr, comparison: &Expr| -> Option<&'static str> {
        let Expr::Exists {
            subquery,
            negated: true,
        } = bare(absence)
        else {
            return None;
        };
        let Expr::BinaryOp {
            left: maximum,
            op: BinaryOperator::Lt,
            right: threshold,
        } = bare(comparison)
        else {
            return None;
        };
        let column_expr = argument(function(maximum, "max")?)?;
        let (owner, max_column) = column(column_expr)?;
        let [from] = select.from.as_slice() else {
            return None;
        };
        if !from.joins.is_empty() || select.selection.is_some() || !catalog.pinned(threshold) {
            return None;
        }
        let relation = walk::relation(&from.relation)?;
        if owner
            .as_deref()
            .is_some_and(|owner| owner != relation.alias)
        {
            return None;
        }
        if catalog.relation(&relation.name).is_none() || groups(select).is_empty() {
            return None;
        }
        if projection(select)?
            .iter()
            .any(|expr| !groups(select).iter().any(|key| bare(key) == bare(expr)))
        {
            return None;
        }
        if subquery.with.is_some() {
            return None;
        }
        let inner = super::expressions::select(subquery)?;
        if projection(inner)?
            .iter()
            .any(|expr| column(expr).is_none() && !catalog.pinned(expr))
        {
            return None;
        }
        if !groups(inner).is_empty()
            || inner.having.is_some()
            || subquery.limit_clause.is_some()
            || subquery.fetch.is_some()
        {
            return None;
        }
        let [inner_from] = inner.from.as_slice() else {
            return None;
        };
        let inner_relation = walk::relation(&inner_from.relation)?;
        if inner_relation.name != relation.name || !inner_from.joins.is_empty() {
            return None;
        }
        let predicates = conjuncts(inner.selection.as_ref());
        for key in groups(select) {
            let (owner, name) = column(key)?;
            if owner
                .as_deref()
                .is_some_and(|owner| owner != relation.alias)
            {
                return None;
            }
            let same = |left: &Expr, right: &Expr| {
                column(left) == Some((Some(inner_relation.alias.clone()), name.clone()))
                    && column(right) == Some((Some(relation.alias.clone()), name.clone()))
            };
            if !predicates.iter().any(|predicate| matches!(predicate, Expr::BinaryOp {left, op: BinaryOperator::Eq, right} if same(left, right) || same(right,left))) { return None }
        }
        let threshold_predicate = predicates.iter().any(|predicate| matches!(predicate,
            Expr::BinaryOp {left, op: BinaryOperator::GtEq, right} if column(left) == Some((Some(inner_relation.alias.clone()), max_column.clone())) && bare(right) == bare(threshold) && !catalog.parameter(right) && !catalog.parameter(threshold)));
        if threshold_predicate || catalog.column_is_not_null(&relation.name, &max_column) {
            Some(if threshold_predicate {
                "MAX repeats the anti-existence threshold; retain the correlated NOT EXISTS and remove the MAX disjunct"
            } else {
                "MAX is a boolean absence test; preserve the original NOT EXISTS disjunct and replace MAX < threshold with threshold IS NOT NULL AND NOT EXISTS for rows at or above the threshold"
            })
        } else {
            None
        }
    };
    check(left, right).or_else(|| check(right, left))
}
