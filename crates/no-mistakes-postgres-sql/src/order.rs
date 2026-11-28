#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalOrderKey {
    pub expression: String,
    pub ascending: bool,
    pub nulls_first: bool,
}

/// PostgreSQL `USING` operators leave sort direction unspecified.
pub fn order_by_ascending(options: &sqlparser::ast::OrderByOptions) -> Option<bool> {
    match options.sort.as_ref() {
        Some(sqlparser::ast::OrderBySort::Asc) => Some(true),
        Some(sqlparser::ast::OrderBySort::Desc) => Some(false),
        Some(sqlparser::ast::OrderBySort::Using(_)) | None => None,
    }
}

pub(crate) fn canonical_order_options(
    options: &sqlparser::ast::OrderByOptions,
) -> Option<(bool, bool)> {
    if matches!(options.sort, Some(sqlparser::ast::OrderBySort::Using(_))) {
        return None;
    }
    let ascending = order_by_ascending(options).unwrap_or(true);
    Some((ascending, options.nulls_first.unwrap_or(!ascending)))
}

/// Canonical `ORDER BY` keys. `USING` operators make the whole list unanalyzable.
pub fn canonical_order_keys(order: &sqlparser::ast::OrderBy) -> Option<Vec<CanonicalOrderKey>> {
    let sqlparser::ast::OrderByKind::Expressions(expressions) = &order.kind else {
        return None;
    };
    expressions
        .iter()
        .map(|expression| {
            let (ascending, nulls_first) = canonical_order_options(&expression.options)?;
            Some(CanonicalOrderKey {
                expression: expression.expr.to_string(),
                ascending,
                nulls_first,
            })
        })
        .collect()
}
