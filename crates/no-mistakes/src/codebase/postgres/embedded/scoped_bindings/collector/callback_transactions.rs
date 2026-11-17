use oxc_ast::ast::{BindingPattern, CallExpression, Expression, FormalParameters};
use oxc_span::{GetSpan, Span};
use std::collections::{HashMap, HashSet};

pub(super) struct Candidate {
    factory: String,
    call_start: u32,
    name: String,
    binding_start: u32,
    scope_end: u32,
}

pub(super) fn candidate(
    call: &CallExpression<'_>,
    factories: &HashSet<String>,
) -> Option<Candidate> {
    let Expression::Identifier(factory) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if !factories.contains(factory.name.as_str()) {
        return None;
    }
    let callback = call.arguments.first()?.as_expression()?;
    let params = match crate::codebase::ts_source::unwrap_ts_wrappers(callback) {
        Expression::ArrowFunctionExpression(callback) => &callback.params,
        Expression::FunctionExpression(callback) => &callback.params,
        _ => return None,
    };
    let param = params.items.first()?;
    let BindingPattern::BindingIdentifier(name) = &param.pattern else {
        return None;
    };
    Some(Candidate {
        factory: factory.name.to_string(),
        call_start: call.span.start,
        name: name.name.to_string(),
        binding_start: name.span.start,
        scope_end: callback.span().end,
    })
}

pub(super) fn finalize(
    candidates: Vec<Candidate>,
    factories: &HashSet<String>,
    shadows: &HashMap<String, Vec<Span>>,
    transactions: &mut HashMap<String, Vec<Span>>,
) {
    for candidate in candidates {
        if !factories.contains(&candidate.factory)
            || shadows.get(&candidate.factory).is_some_and(|spans| {
                spans.iter().any(|span| {
                    span.start <= candidate.call_start && candidate.call_start < span.end
                })
            })
        {
            continue;
        }
        transactions
            .entry(candidate.name)
            .or_default()
            .push(Span::new(candidate.binding_start, candidate.scope_end));
    }
}

pub(super) fn record_parameter_shadows(
    params: &FormalParameters<'_>,
    scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    for param in &params.items {
        let mut names = HashSet::new();
        bound_names(&param.pattern, &mut names);
        for name in names {
            if factories.contains(&name) {
                record_shadow(&name, scope, shadows);
            }
        }
    }
    if let Some(rest) = &params.rest {
        let mut names = HashSet::new();
        bound_names(&rest.rest.argument, &mut names);
        for name in names {
            if factories.contains(&name) {
                record_shadow(&name, scope, shadows);
            }
        }
    }
}

pub(super) fn record_shadow(name: &str, span: Span, shadows: &mut HashMap<String, Vec<Span>>) {
    shadows.entry(name.to_string()).or_default().push(span);
}

pub(super) fn record_function_shadow(
    function: &oxc_ast::ast::Function<'_>,
    outer_scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    let Some(name) = &function.id else { return };
    if !factories.contains(name.name.as_str()) {
        return;
    }
    let scope = match function.r#type {
        oxc_ast::ast::FunctionType::FunctionDeclaration
        | oxc_ast::ast::FunctionType::TSDeclareFunction => outer_scope,
        oxc_ast::ast::FunctionType::FunctionExpression
        | oxc_ast::ast::FunctionType::TSEmptyBodyFunctionExpression => function.span,
    };
    record_shadow(name.name.as_str(), scope, shadows);
}

pub(super) fn record_pattern_shadows(
    pattern: &BindingPattern<'_>,
    scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    let mut names = HashSet::new();
    bound_names(pattern, &mut names);
    for name in names {
        if factories.contains(&name) {
            record_shadow(&name, Span::new(pattern.span().start, scope.end), shadows);
        }
    }
}

pub(super) fn record_local_shadows(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    function_scope: Option<Span>,
    block_scope: Span,
    factories: &HashSet<String>,
    shadows: &mut HashMap<String, Vec<Span>>,
) {
    for declarator in &declaration.declarations {
        let mut names = HashSet::new();
        bound_names(&declarator.id, &mut names);
        let scope = if declaration.kind == oxc_ast::ast::VariableDeclarationKind::Var {
            function_scope.unwrap_or(block_scope)
        } else {
            block_scope
        };
        for name in names {
            if factories.contains(&name) {
                record_shadow(
                    &name,
                    Span::new(scope.start.min(declarator.id.span().start), scope.end),
                    shadows,
                );
            }
        }
    }
}

fn bound_names(pattern: &BindingPattern<'_>, names: &mut HashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.to_string());
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                bound_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                bound_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                bound_names(element, names);
            }
            if let Some(rest) = &array.rest {
                bound_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => bound_names(&assignment.left, names),
    }
}
