use super::dynamic_values::{self, DynamicIdentifierValues};
use super::scoped_defaults::{scoped_static_default_for_identifier, ScopedStaticIdentifierDefault};
use super::types::{AppSelectorValue, TemplatePattern};
use crate::playwright::ast;
use oxc_span::GetSpan;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn app_selector_values(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    file: &Path,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
) -> Vec<AppSelectorValue> {
    app_selector_values_inner(
        value,
        source,
        file,
        scoped_static_identifier_defaults,
        dynamic_identifier_values,
        program,
        None,
    )
}

pub(super) fn app_selector_values_from_visible(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    file: &Path,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    visible_files: &HashSet<PathBuf>,
) -> Vec<AppSelectorValue> {
    app_selector_values_inner(
        value,
        source,
        file,
        scoped_static_identifier_defaults,
        dynamic_identifier_values,
        program,
        Some(visible_files),
    )
}

fn app_selector_values_inner(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    file: &Path,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<AppSelectorValue> {
    let Some(value) = value else {
        return vec![];
    };
    match value {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => {
            vec![AppSelectorValue::Exact(literal.value.to_string())]
        }
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            jsx_expression_values_inner(
                &container.expression,
                source,
                file,
                scoped_static_identifier_defaults,
                dynamic_identifier_values,
                program,
                visible_files,
            )
        }
        _ => vec![],
    }
}

struct SelectorFileContext<'a> {
    file: &'a Path,
    visible_files: Option<&'a HashSet<PathBuf>>,
}

fn jsx_expression_values_inner(
    expression: &oxc_ast::ast::JSXExpression<'_>,
    source: &str,
    file: &Path,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<AppSelectorValue> {
    match expression {
        oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
            vec![AppSelectorValue::Exact(literal.value.to_string())]
        }
        oxc_ast::ast::JSXExpression::TemplateLiteral(template) => {
            let raw = ast::template_literal_text(template, source);
            vec![TemplatePattern::new(&raw)
                .map(AppSelectorValue::Template)
                .unwrap_or_else(|| AppSelectorValue::Unsupported(raw))]
        }
        oxc_ast::ast::JSXExpression::Identifier(identifier) => resolve_identifier(
            identifier.name.as_str(),
            identifier.span(),
            source,
            SelectorFileContext {
                file,
                visible_files,
            },
            scoped_static_identifier_defaults,
            dynamic_identifier_values,
            program,
        ),
        oxc_ast::ast::JSXExpression::ConditionalExpression(cond) => {
            let mut leaves = dynamic_values::collect_string_leaves(&cond.consequent);
            leaves.extend(dynamic_values::collect_string_leaves(&cond.alternate));
            if !leaves.is_empty() {
                return leaves.into_iter().map(AppSelectorValue::Exact).collect();
            }
            vec![AppSelectorValue::Unsupported(
                ast::span_text(source, expression.span()).trim().to_string(),
            )]
        }
        oxc_ast::ast::JSXExpression::LogicalExpression(logical) => {
            let mut leaves = dynamic_values::collect_string_leaves(&logical.left);
            leaves.extend(dynamic_values::collect_string_leaves(&logical.right));
            if !leaves.is_empty() {
                return leaves.into_iter().map(AppSelectorValue::Exact).collect();
            }
            vec![AppSelectorValue::Unsupported(
                ast::span_text(source, expression.span()).trim().to_string(),
            )]
        }
        _ => vec![AppSelectorValue::Unsupported(
            ast::span_text(source, expression.span()).trim().to_string(),
        )],
    }
}

fn resolve_identifier(
    name: &str,
    span: oxc_span::Span,
    source: &str,
    file_context: SelectorFileContext<'_>,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &[DynamicIdentifierValues],
    program: &oxc_ast::ast::Program<'_>,
) -> Vec<AppSelectorValue> {
    // 1. Try scoped static defaults (existing behavior)
    if let Some(value) =
        scoped_static_default_for_identifier(name, span, scoped_static_identifier_defaults, source)
    {
        return vec![AppSelectorValue::Exact(value)];
    }
    // 2. Try dynamic resolution (ternary/if-else/object-map/fn-call)
    let mut values =
        dynamic_values::resolve_dynamic_identifier(name, span, dynamic_identifier_values);
    // 3. Try cross-file imports
    if values.is_empty() {
        values = match file_context.visible_files {
            Some(visible) => dynamic_values::cross_file::resolve_imported_values_from_visible(
                name,
                program,
                file_context.file,
                visible,
            ),
            None => dynamic_values::cross_file::resolve_imported_values(
                name,
                program,
                file_context.file,
            ),
        };
    }
    if !values.is_empty() {
        return values.into_iter().map(AppSelectorValue::Exact).collect();
    }
    // 4. Fallback to Unsupported
    vec![AppSelectorValue::Unsupported(name.to_string())]
}
