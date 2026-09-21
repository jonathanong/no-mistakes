use super::CanonicalOrderKey;

/// PostgreSQL `USING` operators leave sort direction unspecified.
pub(crate) fn order_by_ascending(options: &sqlparser::ast::OrderByOptions) -> Option<bool> {
    match options.sort.as_ref() {
        Some(sqlparser::ast::OrderBySort::Asc) => Some(true),
        Some(sqlparser::ast::OrderBySort::Desc) => Some(false),
        Some(sqlparser::ast::OrderBySort::Using(_)) | None => None,
    }
}

/// Canonical `ORDER BY` keys. `USING` operators make the whole list unanalyzable.
pub(crate) fn canonical_order_keys(
    order: &sqlparser::ast::OrderBy,
) -> Option<Vec<CanonicalOrderKey>> {
    let sqlparser::ast::OrderByKind::Expressions(expressions) = &order.kind else {
        return None;
    };
    if expressions.iter().any(|expression| {
        matches!(
            expression.options.sort,
            Some(sqlparser::ast::OrderBySort::Using(_))
        )
    }) {
        return None;
    }
    Some(
        expressions
            .iter()
            .map(|expression| {
                let ascending = order_by_ascending(&expression.options).unwrap_or(true);
                CanonicalOrderKey {
                    expression: expression.expr.to_string(),
                    ascending,
                    nulls_first: expression.options.nulls_first.unwrap_or(!ascending),
                }
            })
            .collect(),
    )
}
