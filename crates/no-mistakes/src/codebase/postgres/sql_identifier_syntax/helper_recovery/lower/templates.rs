use super::{Expr, Scope};
use crate::codebase::postgres::{
    embedded::walk::resolve::functions::provenance::RecoveredSql, EmbeddedSqlSourcePosition,
};
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::TemplateLiteral;
use oxc_span::GetSpan;

pub(super) fn lower(scope: &mut Scope<'_>, template: &TemplateLiteral<'_>, binds: bool) -> Expr {
    let mut pieces = Vec::new();
    for quasi in &template.quasis {
        let Some(_) = &quasi.value.cooked else {
            return Expr::Opaque;
        };
        let cooked = crate::ast::template_quasi_text(quasi);
        if !super::super::super::literal_projection::literal_matches(
            quasi.value.raw.as_str(),
            &cooked,
        ) {
            return Expr::Opaque;
        }
        let line = byte_offset_to_line(scope.ctx.source, quasi.span.start as usize);
        let mut positions = vec![EmbeddedSqlSourcePosition {
            sql_line: 1,
            sql_column: 1,
            source_line: line,
            source_path: None,
        }];
        let Some(raw) = scope
            .ctx
            .source
            .get(quasi.span.start as usize..quasi.span.end as usize)
        else {
            return Expr::Opaque;
        };
        let mut origins = Vec::new();
        let mut source_index = 0;
        for byte in cooked.bytes() {
            if raw.as_bytes().get(source_index..source_index + 2) == Some(b"\r\n") && byte == b'\n'
            {
                source_index += 1;
            }
            origins.push(quasi.span.start + source_index as u32);
            source_index += 1;
            if byte == b'\n' {
                let next = positions.len() as u32 + 1;
                positions.push(EmbeddedSqlSourcePosition {
                    sql_line: next,
                    sql_column: 1,
                    source_line: line + next - 1,
                    source_path: None,
                });
            }
        }
        pieces.push(RecoveredSql {
            sql: cooked.to_string(),
            line,
            positions,
            origins,
        });
    }
    let holes = template
        .expressions
        .iter()
        .map(|hole| (scope.expression(hole), scope.mark(hole.span())))
        .collect();
    if binds {
        Expr::Sql(pieces, holes, true)
    } else {
        Expr::Template(pieces, holes)
    }
}
