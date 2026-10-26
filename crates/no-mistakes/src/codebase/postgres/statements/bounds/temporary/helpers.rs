use super::{identity, sql_name};
use sqlparser::ast::{ObjectName, SetExpr};

// INTO belongs to the first SELECT in a set operation, including parenthesized queries.
pub(super) fn first_select(expr: &SetExpr) -> Option<&sqlparser::ast::Select> {
    match expr {
        SetExpr::Select(select) => Some(select),
        SetExpr::SetOperation { left, .. } => first_select(left),
        SetExpr::Query(query) => first_select(&query.body),
        _ => None,
    }
}

pub(super) fn temporary_name(name: &ObjectName) -> bool {
    let parts = crate::codebase::postgres::decoded_parts(&sql_name(name));
    matches!(parts.as_slice(), [schema, _] if schema == "pg_temp")
        || identity::database(&sql_name(name)).is_some()
}
