use super::super::dynamic_values::{self, DynamicIdentifierValues};
use super::super::scoped_defaults::{
    scoped_static_default_for_identifier, ScopedStaticIdentifierDefault,
};
use super::super::types::AppSelectorValue;
use super::{literal, recovered, ResolvedAppValue, SelectorFileContext};
use crate::playwright::ast;

pub(super) fn recovered_leaves(
    source: &str,
    span: oxc_span::Span,
    leaves: impl IntoIterator<Item = String>,
) -> Vec<ResolvedAppValue> {
    let leaves: Vec<_> = leaves.into_iter().collect();
    if leaves.is_empty() {
        return vec![literal(AppSelectorValue::Unsupported(
            ast::span_text(source, span).trim().to_string(),
        ))];
    }
    leaves
        .into_iter()
        .map(|value| recovered(AppSelectorValue::Exact(value)))
        .collect()
}

pub(super) fn resolve_identifier(
    name: &str,
    span: oxc_span::Span,
    source: &str,
    file_context: SelectorFileContext<'_>,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
) -> Vec<ResolvedAppValue> {
    // Prop defaults remain accepted at low: they match playwright-literals.
    if let Some(value) =
        scoped_static_default_for_identifier(name, span, scoped_static_identifier_defaults, source)
    {
        return vec![literal(AppSelectorValue::Exact(value))];
    }
    if !file_context.recover_dynamic {
        return vec![literal(AppSelectorValue::Unsupported(name.to_string()))];
    }
    let mut values =
        dynamic_values::resolve_dynamic_identifier(name, span, dynamic_identifier_values);
    if values.is_empty() {
        values = match file_context.visible_files {
            Some(visible) => dynamic_values::cross_file::resolve_imported_values_from_visible(
                name,
                program,
                file_context.file,
                visible,
                file_context.sources,
            ),
            None => dynamic_values::cross_file::resolve_imported_values(
                name,
                program,
                file_context.file,
            ),
        };
    }
    if !values.is_empty() {
        return values
            .into_iter()
            .map(|value| recovered(AppSelectorValue::Exact(value)))
            .collect();
    }
    vec![literal(AppSelectorValue::Unsupported(name.to_string()))]
}
