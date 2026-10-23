use super::dynamic_values::{self, DynamicIdentifierValues};
use super::scoped_defaults::ScopedStaticIdentifierDefault;
use super::types::{AppSelectorValue, TemplatePattern};
use crate::playwright::ast;
use oxc_span::GetSpan;
use std::path::Path;

mod identifier;

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
        oxc_ast::ast::JSXExpression::Identifier(identifier) => identifier::resolve_identifier(
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
            identifier::recovered_leaves(
                source,
                expression.span(),
                dynamic_values::collect_string_leaves(&cond.consequent)
                    .into_iter()
                    .chain(dynamic_values::collect_string_leaves(&cond.alternate)),
            )
        }
        oxc_ast::ast::JSXExpression::LogicalExpression(logical) if file_context.recover_dynamic => {
            identifier::recovered_leaves(
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
