//! Executor calls shaped ``read(`... ${b.text}`, b.values)``.

use super::super::super::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant};
use super::super::variants::ValueKind;
use super::super::ScopeVisitor;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::CallExpression;

/// The call recovered from the template and the builder's final state, or
/// `None` so the caller falls back to the ordinary (fail-closed) path.
pub(super) fn executor_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: &str,
) -> Option<EmbeddedSqlCall> {
    let versions = visitor.recover_builder_text(call)?;
    if versions.is_empty() || versions.iter().any(|value| value.value != ValueKind::Sql) {
        return None;
    }
    let line = byte_offset_to_line(visitor.source, call.span.start as usize);
    let base = EmbeddedSqlCall {
        line,
        callee: callee.to_string(),
        kind: EmbeddedSqlKind::Dynamic,
        ..EmbeddedSqlCall::default()
    };
    if let [single] = versions.as_slice() {
        if single.choices.is_empty() {
            let declaration_line = Some(single.line);
            let variant = single.clone().publish();
            return Some(EmbeddedSqlCall {
                sql_text: Some(variant.sql_text),
                kind: EmbeddedSqlKind::Composed,
                declaration_line,
                sql_source_positions: variant.sql_source_positions,
                recovered_placeholder_positions: variant.recovered_placeholder_positions,
                ..base
            });
        }
    }
    let variants = versions.into_iter().map(|version| version.publish()).fold(
        Vec::new(),
        |mut unique, variant| {
            EmbeddedSqlVariant::push_unique(&mut unique, variant);
            unique
        },
    );
    Some(EmbeddedSqlCall { variants, ..base })
}
