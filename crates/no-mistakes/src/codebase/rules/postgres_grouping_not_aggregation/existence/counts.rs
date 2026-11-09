use super::*;
use std::ops::ControlFlow;
pub(super) fn tests(
    select: &Select,
    catalog: &super::super::scope::CatalogScope<'_>,
    out: &mut Vec<Finding>,
) {
    let projected = projection(select).unwrap_or_default();
    let expressions = projected
        .iter()
        .copied()
        .chain(select.selection.iter())
        .chain(select.having.iter());
    for expression in expressions {
        let _: ControlFlow<()> = visit_expressions(expression, |expr| {
            let Expr::BinaryOp { left, op, right } = bare(expr) else {
                return ControlFlow::Continue(());
            };
            for (count, value, reversed) in [(left, right, false), (right, left, true)] {
                if !zero_test(value, op, reversed) {
                    continue;
                }
                match bare(count) {
                    Expr::Subquery(query)
                        if query.limit_clause.is_none() && query.fetch.is_none() =>
                    {
                        let Some(inner) = super::super::expressions::select(query) else {
                            continue;
                        };
                        let Some(projected) = projection(inner) else {
                            continue;
                        };
                        if matches!(projected.as_slice(), [expr] if counts_rows(expr, inner, catalog))
                            && eligible(inner, catalog)
                        {
                            out.push(report(inner));
                        }
                    }
                    expr if counts_rows(expr, select, catalog)
                        && eligible(select, catalog)
                        && projection(select).is_some_and(|exprs| {
                            exprs.iter().all(|expr| bare(expr) == bare(expression))
                        }) =>
                    {
                        out.push(report(select));
                    }
                    _ => {}
                }
            }
            ControlFlow::Continue(())
        });
    }
}
fn report(select: &Select) -> Finding {
    finding(
        select,
        "aggregate-existence-test",
        "COUNT is used only as a boolean existence test; use EXISTS or NOT EXISTS with the same filters",
    )
}
fn eligible(select: &Select, catalog: &super::super::scope::CatalogScope<'_>) -> bool {
    groups(select).is_empty()
        && select.having.is_none()
        && walk::table_columns(select).is_some_and(|relations| {
            relations
                .iter()
                .all(|relation| catalog.relation(&relation.name).is_some())
        })
}
fn counts_rows(
    expr: &Expr,
    select: &Select,
    catalog: &super::super::scope::CatalogScope<'_>,
) -> bool {
    let Some(function) = function(expr, "count") else {
        return false;
    };
    let Some(args) = args(function) else {
        return false;
    };
    match args.args.as_slice() {
        [FunctionArg::Unnamed(FunctionArgExpr::Wildcard)] => args.duplicate_treatment.is_none(),
        [FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))] => {
            if matches!(bare(expr), Expr::Value(value) if matches!(value.value, Value::Number(..) | Value::SingleQuotedString(_) | Value::Boolean(_)))
            {
                return true;
            }
            if catalog.parameter(expr) {
                return false;
            }
            let Some((owner, name)) = column(expr) else {
                return false;
            };
            let Some(relations) = walk::table_columns(select) else {
                return false;
            };
            let owners = relations
                .iter()
                .filter(|relation| {
                    owner.as_ref().is_some_and(|owner| owner == &relation.alias)
                        || owner.is_none()
                            && catalog.relation(&relation.name).is_some_and(|table| {
                                table.columns.iter().any(|col| col.name == name)
                            })
                })
                .collect::<Vec<_>>();
            let [relation] = owners.as_slice() else {
                return false;
            };
            catalog.column_is_not_null(&relation.name, &name)
                || conjuncts(select.selection.as_ref()).iter().any(|predicate| matches!(predicate, Expr::IsNotNull(value) if column(value) == column(expr)))
        }
        _ => false,
    }
}
fn zero_test(value: &Expr, op: &BinaryOperator, reversed: bool) -> bool {
    matches!(bare(value), Expr::Value(value) if matches!(&value.value, Value::Number(n, _) if n == "0"))
        && (matches!(op, BinaryOperator::Eq | BinaryOperator::NotEq)
            || !reversed && matches!(op, BinaryOperator::Gt | BinaryOperator::LtEq)
            || reversed && matches!(op, BinaryOperator::Lt | BinaryOperator::GtEq))
}
