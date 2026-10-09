use super::super::{Expr, Function, Step};
use oxc_ast::ast::{BindingPattern, FormalParameters};

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
        value.span.start,
    ))
}

pub(super) fn function(
    params: &FormalParameters<'_>,
    body: Vec<Step>,
    supported: bool,
    asynchronous: bool,
    start: u32,
) -> Function {
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
    Function {
        start,
        params: names,
        body,
        supported,
        asynchronous,
        self_name: None,
    }
}
