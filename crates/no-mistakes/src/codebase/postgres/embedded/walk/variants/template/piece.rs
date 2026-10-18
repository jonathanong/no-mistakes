use super::{Recovered, ScopeVisitor, ValueKind};
use crate::codebase::postgres::embedded::{source_positions, tags};
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::TemplateElement;

/// The static text of one template quasi as a SQL fragment with exact origins.
pub(super) fn quasi(visitor: &ScopeVisitor<'_>, quasi: &TemplateElement<'_>) -> Recovered {
    let line = byte_offset_to_line(visitor.source, quasi.span.start as usize);
    let raw = &visitor.source[quasi.span.start as usize..quasi.span.end as usize];
    let decoded = tags::quasi_text(quasi, false);
    Recovered {
        value: ValueKind::Sql,
        sql: decoded.into(),
        origins: source_positions::origins::piece(
            raw,
            decoded,
            quasi.span.start,
            quasi.value.cooked.is_none(),
        ),
        line,
        positions: source_positions::piece(raw, decoded, line, quasi.value.cooked.is_none()),
        fragment: true,
        choices: Vec::new(),
        append_sites: Vec::new(),
        enumerated: false,
    }
}
