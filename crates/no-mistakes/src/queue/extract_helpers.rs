use crate::ast;
use oxc_ast::ast::{Argument, BindingPattern, Expression, ModuleExportName, TemplateLiteral};
use oxc_span::GetSpan;
use std::collections::HashMap;

/// The processor parameter the queue checks treat as the active job.
#[derive(Debug, Default, Clone)]
pub(super) struct JobFrame {
    /// Function identity for conservative imported-module dispatch summaries.
    pub scope: u32,
    pub object: Option<String>,
    pub name_idents: Vec<String>,
    pub id_idents: Vec<String>,
}

pub(super) fn binding_name(pattern: &BindingPattern<'_>) -> Option<String> {
    if let BindingPattern::BindingIdentifier(id) = pattern {
        return Some(id.name.as_str().to_string());
    }
    None
}

#[rustfmt::skip]
pub(super) fn export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(id) => id.name.as_str().to_string(), ModuleExportName::IdentifierReference(id) => id.name.as_str().to_string(),
        ModuleExportName::StringLiteral(value) => value.value.as_str().to_string(),
    }
}

pub(super) fn is_queue_package(source: &str) -> bool {
    matches!(source, "bullmq" | "glide-mq")
}

pub(super) fn static_string(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::StringLiteral(value) => Some(value.value.as_str().to_string()),
        Expression::TemplateLiteral(template) => no_substitution_template(template),
        _ => None,
    }
}

pub(super) fn literal_expr(
    expr: &Expression<'_>,
    consts: &HashMap<String, String>,
) -> Option<String> {
    if let Some(value) = static_string(expr) {
        return Some(value);
    }
    match expr {
        Expression::Identifier(id) => consts.get(id.name.as_str()).cloned(),
        _ => None,
    }
}

pub(super) fn literal_or_import(
    expr: &Expression<'_>,
    consts: &HashMap<String, String>,
    imports: &HashMap<String, (String, String)>,
) -> (Option<String>, Option<String>) {
    if let Some(value) = literal_expr(expr, consts) {
        return (Some(value), None);
    }
    let Expression::Identifier(id) = expr else {
        return (None, None);
    };
    if imports.contains_key(id.name.as_str()) {
        (None, Some(id.name.to_string()))
    } else {
        (None, None)
    }
}

fn no_substitution_template(template: &TemplateLiteral<'_>) -> Option<String> {
    if !template.expressions.is_empty() {
        return None;
    }
    let mut text = String::new();
    for quasi in &template.quasis {
        text.push_str(quasi.value.cooked.as_deref()?);
    }
    Some(text)
}

pub(super) fn processor_specifier(
    args: &[Argument<'_>],
    source: &str,
    namespace_imports: &HashMap<String, String>,
) -> Option<String> {
    let source = args
        .iter()
        .filter_map(Argument::as_expression)
        .find_map(|expr| {
            let text = ast::span_text(source, expr.span());
            namespace_imports
                .keys()
                .find_map(|name| text.contains(name).then_some(name.clone()))
        });
    source.and_then(|name| namespace_imports.get(&name).cloned())
}
