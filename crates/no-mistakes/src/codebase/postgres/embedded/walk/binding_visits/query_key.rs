use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

/// A computed write is harmless only when its key is a known other property.
pub(super) fn may_write_query(key: &Expression<'_>) -> bool {
    !matches!(
        unwrap_ts_wrappers(key),
        Expression::StringLiteral(literal) if literal.value != "query"
    )
}
