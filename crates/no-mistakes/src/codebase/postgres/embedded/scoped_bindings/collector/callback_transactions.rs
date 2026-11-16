use oxc_ast::ast::{BindingPattern, CallExpression, Expression, FormalParameters};
use oxc_span::{GetSpan, Span};
use std::collections::{HashMap, HashSet};

pub(super) fn record(
    call: &CallExpression<'_>,
    factories: &HashSet<String>,
    shadows: &HashMap<String, Vec<Span>>,
    transactions: &mut HashMap<String, Vec<Span>>,
) {
    let Expression::Identifier(factory) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return;
    };
    if !factories.contains(factory.name.as_str())
        || shadows.get(factory.name.as_str()).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.start <= call.span.start && call.span.start < span.end)
        })
    {
        return;
    }
    let Some(callback) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
    else {
        return;
    };
    let params = match crate::codebase::ts_source::unwrap_ts_wrappers(callback) {
        Expression::ArrowFunctionExpression(callback) => &callback.params,
        Expression::FunctionExpression(callback) => &callback.params,
        _ => return,
    };
    let Some(param) = params.items.first() else {
        return;
    };
    let BindingPattern::BindingIdentifier(name) = &param.pattern else {
        return;
    };
    transactions
        .entry(name.name.to_string())
        .or_default()
        .push(Span::new(name.span.start, callback.span().end));
}

pub(super) fn record_parameter_shadows(
    params: &FormalParameters<'_>,
    scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    for param in &params.items {
        if let BindingPattern::BindingIdentifier(name) = &param.pattern {
            if factories.contains(name.name.as_str()) {
                shadows
                    .entry(name.name.to_string())
                    .or_default()
                    .push(Span::new(name.span.start, scope.end));
            }
        }
    }
}

pub(super) fn record_shadow(name: &str, span: Span, shadows: &mut HashMap<String, Vec<Span>>) {
    shadows.entry(name.to_string()).or_default().push(span);
}

pub(super) fn record_local_shadows(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    function_scope: Option<Span>,
    block_scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(name) = &declarator.id else {
            continue;
        };
        if !factories.contains(name.name.as_str()) {
            continue;
        }
        let scope = if declaration.kind == oxc_ast::ast::VariableDeclarationKind::Var {
            function_scope.unwrap_or(block_scope)
        } else {
            block_scope
        };
        record_shadow(
            name.name.as_str(),
            Span::new(scope.start.min(name.span.start), scope.end),
            shadows,
        );
    }
}
