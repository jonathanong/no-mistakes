use super::super::super::{first_call_argument, EmbeddedSqlCall, EmbeddedSqlKind};
use super::{classify_init, BindingState, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
mod variants;

fn legacy_executor_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
) -> EmbeddedSqlCall {
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, call.span.start as usize);
    let Some(argument) = first_call_argument(call) else {
        return EmbeddedSqlCall {
            variants: Vec::new(),
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
            let mut binding = visitor.lookup(ident.name.as_str());
            let (sql_text, recovered_placeholder_positions) = binding
                .as_ref()
                .and_then(|binding| binding.sql.clone())
                .map(super::super::super::placeholders::publish_placeholders_with_positions)
                .map_or_else(
                    || (None, Vec::new()),
                    |(sql, positions)| (Some(sql), positions),
                );
            let variants = binding
                .as_mut()
                .filter(|binding| {
                    binding.kind == EmbeddedSqlKind::Composed
                        && !visitor.append_crosses_function(ident.name.as_str())
                })
                .map(|binding| variants::publish(binding.variants.take(), binding.kind))
                .unwrap_or_default();
            EmbeddedSqlCall {
                variants,
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
                variants: Vec::new(),
                line,
                callee,
                sql_text,
                kind: if kind == EmbeddedSqlKind::ImmutableLocal {
                    EmbeddedSqlKind::Inline
                } else {
                    kind
                },
                declaration_line: None,
                sql_source_positions: super::compose::helper_provenance(argument, visitor)
                    .map(|provenance| provenance.positions)
                    .or_else(|| super::compose::parameter_source_positions(argument, visitor))
                    .unwrap_or_else(|| {
                        super::super::super::source_positions::for_expression(
                            argument,
                            visitor.source,
                            call.span.start as usize,
                            line,
                        )
                    }),
                recovered_placeholder_positions,
            }
        }
    }
}

pub(crate) fn executor_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
) -> EmbeddedSqlCall {
    if visitor.head_calls_only {
        return EmbeddedSqlCall {
            line: crate::codebase::ts_source::byte_offset_to_line(
                visitor.source,
                call.span.start as usize,
            ),
            callee,
            sql_text: first_call_argument(call)
                .and_then(|argument| super::super::head_calls::text(argument, visitor)),
            kind: EmbeddedSqlKind::Dynamic,
            ..Default::default()
        };
    }
    if let Some(recovered) = (!visitor.head_calls_only)
        .then(|| super::builder_text::executor_call(visitor, call, &callee))
        .flatten()
    {
        return recovered;
    }
    let mut recovered = legacy_executor_call(visitor, call, callee);
    if recovered.variants.is_empty() {
        recovered.variants = first_call_argument(call)
            .map(|argument| variants::argument(visitor, argument, recovered.kind))
            .unwrap_or_default();
    }
    recovered
}

pub(in crate::codebase::postgres::embedded::walk) fn executor_call_from_binding(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
    binding: &BindingState,
) -> EmbeddedSqlCall {
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, call.span.start as usize);
    let (sql_text, recovered_placeholder_positions) = binding
        .sql
        .clone()
        .map(super::super::super::placeholders::publish_placeholders_with_positions)
        .map_or_else(
            || (None, Vec::new()),
            |(sql, positions)| (Some(sql), positions),
        );
    let variants = if matches!(
        binding.kind,
        EmbeddedSqlKind::Dynamic | EmbeddedSqlKind::Composed
    ) {
        variants::publish_for_argument(
            visitor,
            first_call_argument(call),
            binding.variants.clone(),
            binding.kind,
        )
    } else {
        Vec::new()
    };
    EmbeddedSqlCall {
        variants,
        line,
        callee,
        sql_text,
        kind: binding.kind,
        declaration_line: Some(binding.line),
        sql_source_positions: binding.sql_source_positions.clone(),
        recovered_placeholder_positions,
    }
}
