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
    let mut summary = function(
        &value.params,
        body,
        !value.generator,
        value.r#async,
        false,
        value.span.start,
    );
    summary_proof(value, source, &mut summary);
    Expr::Function(summary)
}

// The proof may come from a lowering the general summary does not use, so a
// function the evaluator treats as unsupported elsewhere stays unsupported.
// A supported summary omits property defaults, computed keys, and
// non-identifier rest on a destructured call; those patterns get no proof.
// Re-lowering and the body census run only when a callback call still survives
// the cheap step checks. A missing call, or a read that already rejects the
// proof, does not walk the function.
fn summary_proof(
    value: &oxc_ast::ast::Function<'_>,
    source: &str,
    summary: &mut std::sync::Arc<Function>,
) {
    let branched = summary
        .body
        .iter()
        .any(|step| matches!(step, Step::Branch(..)));
    if !branched && !super::append_callback::destructured_append(summary) {
        return;
    }
    if !super::append_callback::needs_body_census(value, summary) {
        return;
    }
    if summary.supported && super::super::statements::destructured::hides_append_proof(value) {
        return;
    }
    let candidate = if summary.supported {
        summary.clone()
    } else if branched
        && summary
            .body
            .iter()
            .any(|step| matches!(step, Step::Unsupported))
    {
        let body = value
            .body
            .as_ref()
            .expect("functions with branch steps have a body");
        let alt = super::super::statements::append_steps(&body.statements, source);
        let alt = function_from(value, alt);
        if !alt.supported {
            return;
        }
        alt
    } else {
        return;
    };
    let proof = super::append_callback::collect(value, &candidate).map(std::sync::Arc::new);
    std::sync::Arc::make_mut(summary).append_callback = proof;
}

fn function_from(value: &oxc_ast::ast::Function<'_>, body: Vec<Step>) -> std::sync::Arc<Function> {
    function(
        &value.params,
        body,
        !value.generator,
        value.r#async,
        false,
        value.span.start,
    )
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
        append_callback: None,
    })
}

#[cfg(test)]
#[path = "append_proof_walk_tests.rs"]
mod tests;
