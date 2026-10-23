use super::super::Expr;
use super::{expr, Scope};
use crate::codebase::postgres::embedded::helpers::split::with_origin_entry;
use crate::codebase::postgres::embedded::tags::{self, is_sql_tag};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::{source_positions, tags::quasi_text};
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{TaggedTemplateExpression, TemplateElement, TemplateLiteral};
use oxc_span::GetSpan;

/// The static text of one quasi with exact physical positions.
fn piece(scope: &Scope<'_, '_>, quasi: &TemplateElement<'_>, raw_mode: bool) -> RecoveredSql {
    let source = scope.ctx.source;
    let line = byte_offset_to_line(source, quasi.span.start as usize);
    let raw = &source[quasi.span.start as usize..quasi.span.end as usize];
    let decoded = quasi_text(quasi, raw_mode);
    let broken = quasi.value.cooked.is_none();
    with_origin_entry(RecoveredSql {
        sql: decoded.to_string(),
        line,
        positions: source_positions::piece(raw, decoded, line, broken || raw_mode),
        origins: source_positions::origins::piece(
            raw,
            decoded,
            quasi.span.start,
            broken || raw_mode,
        ),
    })
}

pub(super) fn untagged(scope: &mut Scope<'_, '_>, template: &TemplateLiteral<'_>) -> Expr {
    let pieces = template
        .quasis
        .iter()
        .map(|quasi| piece(scope, quasi, false))
        .collect();
    let holes = template
        .expressions
        .iter()
        .map(|hole| (expr::lower(scope, hole), scope.mark(hole.span())))
        .collect();
    Expr::Template(pieces, holes)
}

/// A template under the trusted SQL tag; any other tag stays opaque.
pub(super) fn tagged(scope: &mut Scope<'_, '_>, tagged: &TaggedTemplateExpression<'_>) -> Expr {
    let trusted = is_sql_tag(
        &tagged.tag,
        &mut |name| scope.is_rebound(name),
        scope.ctx.tags,
    );
    if !trusted || tags::is_string_raw_tag_spelling(&tagged.tag) {
        return Expr::Opaque;
    }
    let pieces = tagged
        .quasi
        .quasis
        .iter()
        .map(|quasi| piece(scope, quasi, false))
        .collect();
    let holes = tagged
        .quasi
        .expressions
        .iter()
        .map(|hole| (expr::lower(scope, hole), scope.mark(hole.span())))
        .collect();
    Expr::Sql(pieces, holes, scope.ctx.tags.parameterizes_all(&tagged.tag))
}
