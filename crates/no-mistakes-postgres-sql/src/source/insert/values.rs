//! VALUES facts expose rows only; omitted query modifiers make them incomplete.
use sqlparser::ast::Query;

pub(super) fn unmodified(query: &Query) -> bool {
    [
        query.with.is_some(),
        query.order_by.is_some(),
        query.limit_clause.is_some(),
        query.fetch.is_some(),
        !query.locks.is_empty(),
        query.for_clause.is_some(),
        query.settings.is_some(),
        query.format_clause.is_some(),
        !query.pipe_operators.is_empty(),
    ]
    .into_iter()
    .all(|unsupported| !unsupported)
}
