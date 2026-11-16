//! A value-position string stays a bind inside a raw `sql` tag.
//! Identifier positions and SQL fragments still splice.

use super::super::super::super::super::ScopeVisitor;
use super::super::super::super::Recovered;
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::placeholders;
use crate::codebase::postgres::embedded::source_positions;
use crate::codebase::postgres::embedded::tags;
use crate::codebase::postgres::embedded::walk::resolve::compose;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

pub(super) fn bind(
    enabled: bool,
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    versions: &[Recovered],
) -> Option<Vec<Recovered>> {
    if !enabled || !versions.iter().all(|version| sentinel::value_position(&version.sql)) {
        return None;
    }
    let expr = unwrap_ts_wrappers(expr);
    if fragment(visitor, expr) || !text_value(visitor, expr) {
        return None;
    }
    let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
    let mut value = Recovered::empty(line);
    value.sql = placeholders::internal_placeholder(1);
    value.origins = vec![expr.span().start; value.sql.len()];
    value.positions = source_positions::for_expression(
        expr,
        visitor.source,
        expr.span().start as usize,
        line,
    );
    (value.origins.len() == value.sql.len()).then_some(vec![value])
}

fn fragment(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> bool {
    tags::fragments::is_sql_fragment(
        expr,
        &mut |name| compose::tag_shadowed(name, visitor),
        visitor.functions.imported_sql_tags(),
    )
}

fn text_value(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::Identifier(ident) => visitor
            .literal_objects
            .is_text_const(&visitor.scopes, ident.name.as_str()),
        Expression::StaticMemberExpression(_) => true,
        _ => false,
    }
}
