use super::dynamic_values::{self, DynamicIdentifierValues};
use super::scoped_defaults::{scoped_static_default_for_identifier, ScopedStaticIdentifierDefault};
use super::types::{AppSelectorValue, TemplatePattern};
use crate::playwright::ast;
use oxc_span::GetSpan;
use std::path::Path;

pub(super) struct SelectorFileContext<'a> {
    pub(super) file: &'a Path,
    pub(super) visible_files: Option<&'a crate::fx::PathSet>,
    pub(super) sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    pub(super) recover_dynamic: bool,
}

pub(super) struct ResolvedAppValue {
    pub(super) value: AppSelectorValue,
    pub(super) recovered: bool,
}

fn literal(value: AppSelectorValue) -> ResolvedAppValue {
    ResolvedAppValue {
        value,
        recovered: false,
    }
}

fn recovered(value: AppSelectorValue) -> ResolvedAppValue {
    ResolvedAppValue {
        value,
        recovered: true,
    }
}

pub(super) fn app_selector_values(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    file: &Path,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
) -> Vec<ResolvedAppValue> {
    app_selector_values_inner(
        value,
        source,
        scoped_static_identifier_defaults,
        dynamic_identifier_values,
        program,
        SelectorFileContext {
            file,
            visible_files: None,
            sources: None,
            recover_dynamic: true,
        },
    )
}

pub(super) fn app_selector_values_from_visible(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    file_context: SelectorFileContext<'_>,
) -> Vec<ResolvedAppValue> {
    app_selector_values_inner(
        value,
        source,
        scoped_static_identifier_defaults,
        dynamic_identifier_values,
        program,
        file_context,
    )
}

fn app_selector_values_inner(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    file_context: SelectorFileContext<'_>,
) -> Vec<ResolvedAppValue> {
    let Some(value) = value else {
        return vec![];
    };
    match value {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(lit) => {
            vec![literal(AppSelectorValue::Exact(lit.value.to_string()))]
        }
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            jsx_expression_values_inner(
                &container.expression,
                source,
                scoped_static_identifier_defaults,
                dynamic_identifier_values,
                program,
                file_context,
            )
        }
        _ => vec![],
    }
}

fn jsx_expression_values_inner(
    expression: &oxc_ast::ast::JSXExpression<'_>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    file_context: SelectorFileContext<'_>,
) -> Vec<ResolvedAppValue> {
    match expression {
        oxc_ast::ast::JSXExpression::StringLiteral(lit) => {
            vec![literal(AppSelectorValue::Exact(lit.value.to_string()))]
        }
        oxc_ast::ast::JSXExpression::TemplateLiteral(template) => {
            let raw = ast::template_literal_text(template, source);
            vec![literal(
                TemplatePattern::new(&raw)
                    .map(AppSelectorValue::Template)
                    .unwrap_or_else(|| AppSelectorValue::Unsupported(raw)),
            )]
        }
        oxc_ast::ast::JSXExpression::Identifier(identifier) => resolve_identifier(
            identifier.name.as_str(),
            identifier.span(),
            source,
            file_context,
            scoped_static_identifier_defaults,
            dynamic_identifier_values,
            program,
        ),
        oxc_ast::ast::JSXExpression::ConditionalExpression(cond)
            if file_context.recover_dynamic =>
        {
            recovered_leaves(
                source,
                expression.span(),
                dynamic_values::collect_string_leaves(&cond.consequent)
                    .into_iter()
                    .chain(dynamic_values::collect_string_leaves(&cond.alternate)),
            )
        }
        oxc_ast::ast::JSXExpression::LogicalExpression(logical) if file_context.recover_dynamic => {
            recovered_leaves(
                source,
                expression.span(),
                dynamic_values::collect_string_leaves(&logical.left)
                    .into_iter()
                    .chain(dynamic_values::collect_string_leaves(&logical.right)),
            )
        }
        _ => vec![literal(AppSelectorValue::Unsupported(
            ast::span_text(source, expression.span()).trim().to_string(),
        ))],
    }
}

fn recovered_leaves(
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

fn resolve_identifier(
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
