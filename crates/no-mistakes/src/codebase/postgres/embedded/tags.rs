use super::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn interpolating_untrusted_tag(expr: &Expression<'_>) -> bool {
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(expr) else {
        return false;
    };
    if tagged.quasi.expressions.is_empty() {
        return false;
    }
    !is_sql_tag(&tagged.tag)
}

pub(super) fn kind_for_const(sql: String, is_const: bool) -> (Option<String>, EmbeddedSqlKind) {
    if is_const {
        (Some(sql), EmbeddedSqlKind::ImmutableLocal)
    } else {
        (Some(sql), EmbeddedSqlKind::Dynamic)
    }
}

fn is_sql_tag(tag: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(tag),
        Expression::Identifier(ident) if ident.name.eq_ignore_ascii_case("sql")
    )
}
