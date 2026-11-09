use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn fill(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    sql: Option<&str>,
    offsets: &mut Vec<u32>,
) {
    if !visitor.exact_sql_origins || !offsets.is_empty() {
        return;
    }
    let Some(sql) = sql else { return };
    if !matches!(
        unwrap_ts_wrappers(argument),
        Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
    ) {
        return;
    }
    let origins =
        super::super::super::source_positions::origins::expression(argument, visitor.source, sql);
    *offsets = super::rewrite_origins(sql, &origins, 0, true);
}
