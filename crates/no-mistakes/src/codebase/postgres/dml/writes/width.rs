use sqlparser::ast::{Query, SelectItem, SetExpr};

pub(super) fn query_value_width(query: &Query) -> Option<usize> {
    set_expr_width(&query.body)
}

fn set_expr_width(expr: &SetExpr) -> Option<usize> {
    match expr {
        SetExpr::Values(values) => values.rows.iter().map(|row| row.len()).max(),
        SetExpr::Select(select) => select_width(&select.projection),
        SetExpr::Query(query) => query_value_width(query),
        SetExpr::SetOperation { left, right, .. } => {
            match (set_expr_width(left), set_expr_width(right)) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (left, right) => left.or(right),
            }
        }
        _ => None,
    }
}

fn select_width(projection: &[SelectItem]) -> Option<usize> {
    if projection.iter().any(|item| {
        matches!(
            item,
            SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _)
        )
    }) {
        return Some(usize::MAX);
    }
    Some(projection.len())
}

#[cfg(test)]
mod tests;
