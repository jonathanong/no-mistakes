use super::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

/// `is_shadowed` reports whether a name is currently bound to something
/// other than the global trusted SQL tag (e.g. a same-file helper's own
/// parameter named `sql`) — matching the tag purely by spelling, as
/// [`is_sql_tag`] alone does, would trust a caller-controlled tag function
/// passed in under that name.
pub(super) fn interpolating_untrusted_tag(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
) -> bool {
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(expr) else {
        return false;
    };
    if tagged.quasi.expressions.is_empty() {
        return false;
    }
    !is_sql_tag(&tagged.tag, is_shadowed)
}

pub(super) fn kind_for_const(sql: String, is_const: bool) -> (Option<String>, EmbeddedSqlKind) {
    if is_const {
        (Some(sql), EmbeddedSqlKind::ImmutableLocal)
    } else {
        (Some(sql), EmbeddedSqlKind::Dynamic)
    }
}

fn is_sql_tag(tag: &Expression<'_>, is_shadowed: &mut impl FnMut(&str) -> bool) -> bool {
    match unwrap_ts_wrappers(tag) {
        Expression::Identifier(ident) if ident.name.eq_ignore_ascii_case("sql") => {
            !is_shadowed(ident.name.as_str())
        }
        _ => false,
    }
}
