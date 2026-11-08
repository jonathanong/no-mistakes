use super::{RuleFinding, SqlBoundKind, RULE_ID};

pub(super) fn message(
    kind: SqlBoundKind,
    table: &str,
    object: &crate::codebase::postgres::CatalogObjectRef,
) -> String {
    if matches!(object, crate::codebase::postgres::CatalogObjectRef::View(_)) {
        return format!("{kind:?} can read unbounded rows from view {table}; add a row cap or configure viewBounds as bounded only when its result is finite");
    }
    match kind {
        SqlBoundKind::Select => format!(
            "SELECT can return every row of {table}; add a LIMIT, look rows up by a primary or unique key, or allow the table with a reason"
        ),
        SqlBoundKind::Update => format!(
            "UPDATE can change every row of {table}; pick the target rows in a CTE or subquery with a LIMIT, or match a primary or unique key"
        ),
        SqlBoundKind::Delete => format!(
            "DELETE can remove every row of {table}; pick the target rows in a CTE or subquery with a LIMIT, or match a primary or unique key"
        ),
    }
}

pub(super) fn finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
