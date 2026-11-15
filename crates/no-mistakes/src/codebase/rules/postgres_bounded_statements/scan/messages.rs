use super::RULE_ID;
use crate::codebase::postgres::SqlBoundKind;
use crate::codebase::rules::RuleFinding;

pub(super) fn message(
    kind: SqlBoundKind,
    table: &str,
    object: &crate::codebase::postgres::CatalogObjectRef,
    domain_over: Option<u64>,
    cap: Option<u64>,
) -> String {
    let mut text = if matches!(object, crate::codebase::postgres::CatalogObjectRef::View(_)) {
        format!("{kind:?} can read unbounded rows from view {table}; add a row cap or configure viewBounds as bounded only when its result is finite")
    } else {
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
    };
    if let (Some(product), Some(cap)) = (domain_over, cap) {
        text.push_str(&format!(
            "; unique-key domain product is {product}, above maxDomainRows {cap}"
        ));
    }
    text
}

pub(super) fn finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
