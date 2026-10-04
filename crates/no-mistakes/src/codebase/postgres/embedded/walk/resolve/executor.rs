use super::super::super::{first_call_argument, EmbeddedSqlCall, EmbeddedSqlKind};
use super::{classify_init, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(crate) fn executor_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
) -> EmbeddedSqlCall {
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, call.span.start as usize);
    let Some(argument) = first_call_argument(call) else {
        return EmbeddedSqlCall {
            line,
            callee,
            sql_text: None,
            kind: EmbeddedSqlKind::Dynamic,
            declaration_line: None,
            sql_source_positions: Vec::new(),
            recovered_placeholder_positions: Vec::new(),
        };
    };
    match unwrap_ts_wrappers(argument) {
        Expression::Identifier(ident) => {
            let binding = visitor.lookup(ident.name.as_str());
            let (sql_text, recovered_placeholder_positions) = binding
                .as_ref()
                .and_then(|binding| binding.sql.clone())
                .map(super::super::super::placeholders::publish_placeholders_with_positions)
                .map_or_else(
                    || (None, Vec::new()),
                    |(sql, positions)| (Some(sql), positions),
                );
            EmbeddedSqlCall {
                line,
                callee,
                sql_text,
                kind: binding
                    .as_ref()
                    .map(|binding| binding.kind)
                    .unwrap_or(EmbeddedSqlKind::Dynamic),
                sql_source_positions: binding
                    .as_ref()
                    .map(|binding| binding.sql_source_positions.clone())
                    .unwrap_or_default(),
                recovered_placeholder_positions,
                declaration_line: binding.map(|binding| binding.line),
            }
        }
        _ => {
            let (sql, kind) = classify_init(argument, true, visitor);
            let (sql_text, recovered_placeholder_positions) = sql
                .map(super::super::super::placeholders::publish_placeholders_with_positions)
                .map_or_else(
                    || (None, Vec::new()),
                    |(sql, positions)| (Some(sql), positions),
                );
            EmbeddedSqlCall {
                line,
                callee,
                sql_text,
                kind: if kind == EmbeddedSqlKind::ImmutableLocal {
                    EmbeddedSqlKind::Inline
                } else {
                    kind
                },
                declaration_line: None,
                sql_source_positions: super::super::super::source_positions::for_expression(
                    argument,
                    visitor.source,
                    call.span.start as usize,
                    line,
                ),
                recovered_placeholder_positions,
            }
        }
    }
}
