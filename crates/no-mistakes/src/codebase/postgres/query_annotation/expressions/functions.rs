use super::super::{Expr, Function, Step};
use oxc_ast::ast::{ArrowFunctionBody, ArrowFunctionExpression, BindingPattern, FormalParameters};

pub(super) fn arrow(value: &ArrowFunctionExpression<'_>, source: &str) -> Expr {
    let body = match &value.body {
        ArrowFunctionBody::FunctionBody(body) => {
            super::super::statements::steps(&body.statements, source)
        }
        other => vec![Step::Return(
            other
                .as_expression()
                .map_or(Expr::Unknown, |expr| super::expression(expr, source)),
        )],
    };
    Expr::Function(function(
        &value.params,
        body,
        true,
        value.r#async,
        true,
        value.span.start,
    ))
}

pub(super) fn named_function_expression(value: &oxc_ast::ast::Function<'_>, source: &str) -> Expr {
    let mut function = function_expression(value, source);
    if let (Some(id), Expr::Function(summary)) = (&value.id, &mut function) {
        std::sync::Arc::make_mut(summary).self_name = Some(id.name.to_string());
    }
    function
}

pub(in crate::codebase::postgres::query_annotation) fn function_expression(
    value: &oxc_ast::ast::Function<'_>,
    source: &str,
) -> Expr {
    let body = value
        .body
        .as_ref()
        .map(|body| super::super::statements::steps(&body.statements, source))
        .unwrap_or_default();
    Expr::Function(function(
        &value.params,
        body,
        !value.generator,
        value.r#async,
        false,
        value.span.start,
    ))
}

pub(super) fn function(
    params: &FormalParameters<'_>,
    body: Vec<Step>,
    supported: bool,
    asynchronous: bool,
    arrow: bool,
    start: u32,
) -> std::sync::Arc<Function> {
    let names = params
        .items
        .iter()
        .map(|param| match &param.pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    let supported = supported
        && names.is_some()
        && params.rest.is_none()
        && !body.iter().any(|step| matches!(step, Step::Unsupported));
    let names = names.unwrap_or_else(|| {
        params
            .items
            .iter()
            .flat_map(|param| super::super::trust::bound_names(&param.pattern))
            .collect()
    });
    let mut names = names;
    if let Some(rest) = &params.rest {
        names.extend(super::super::trust::bound_names(&rest.rest.argument));
    }
    std::sync::Arc::new(Function {
        start,
        params: names,
        body,
        supported,
        asynchronous,
        arrow,
        self_name: None,
    })
}
