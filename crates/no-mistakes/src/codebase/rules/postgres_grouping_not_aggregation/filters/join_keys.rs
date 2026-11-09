use super::*;
pub(super) fn join_covers(select: &Select, on: &Expr, alias: &str) -> bool {
    let predicates = conjuncts(Some(on));
    select.projection.iter().all(|item| {
        let name = match item {
            SelectItem::UnnamedExpr(expr) => column(expr).map(|(_, name)| name),
            SelectItem::ExprWithAlias { alias, .. } => Some(ident(alias)),
            _ => None,
        };
        let Some(name) = name else { return false };
        predicates.iter().any(|predicate| match predicate {
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Eq,
                right,
            } => [(left, right), (right, left)].iter().any(|(key, value)| {
                column(key) == Some((Some(alias.to_string()), name.clone()))
                    && !uses(value, alias)
                    && (pinned_value(value)
                        || column(value).is_some_and(|(owner, _)| owner.is_some()))
            }),
            _ => false,
        })
    })
}
