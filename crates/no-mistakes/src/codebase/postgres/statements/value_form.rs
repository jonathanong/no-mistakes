use sqlparser::ast::{Expr, InTarget};

pub(super) fn is_list_or_subquery(target: &InTarget) -> bool {
    matches!(target, InTarget::List(_) | InTarget::Subquery(_))
}

pub(super) fn is_subquery_value(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Subquery(_)
            | Expr::Exists { .. }
            | Expr::In {
                target: InTarget::Subquery(_),
                ..
            }
    )
}
