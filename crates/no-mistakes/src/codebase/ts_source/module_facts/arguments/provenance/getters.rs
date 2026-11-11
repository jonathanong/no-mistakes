use super::super::path_origin::env_name;
use super::super::{PathOrigin, PathProvenance, Projector};
use oxc_ast::ast::*;

pub(super) fn record<'a>(
    projector: &mut Projector<'a, '_>,
    function: &Function<'a>,
    out: &mut PathProvenance,
) {
    record_defaults(projector, function.span.start, &function.params, out);
    if function.generator
        || function.r#async
        || !function.params.items.is_empty()
        || function.params.rest.is_some()
    {
        return;
    }
    let Some(body) = &function.body else { return };
    let [Statement::ReturnStatement(returned)] = body.statements.as_slice() else {
        return;
    };
    let Some(expression) = &returned.argument else {
        return;
    };
    let root = match expression.get_inner_expression() {
        Expression::LogicalExpression(logical)
            if matches!(
                logical.operator,
                LogicalOperator::Coalesce | LogicalOperator::Or
            ) && matches!(
                logical.right.get_inner_expression(),
                Expression::StringLiteral(_)
            ) =>
        {
            &logical.left
        }
        other => other,
    };
    let Some(name) = env_name(projector.semantic, root) else {
        return;
    };
    out.env_getters.insert(
        function.span.start,
        PathOrigin {
            env: Some(name),
            ..PathOrigin::unknown()
        },
    );
}

pub(super) fn record_defaults<'a>(
    projector: &mut Projector<'a, '_>,
    owner: u32,
    params: &FormalParameters<'a>,
    out: &mut PathProvenance,
) {
    for (index, parameter) in params.items.iter().enumerate() {
        if let Some(initializer) = &parameter.initializer {
            if !matches!(
                initializer.get_inner_expression(),
                Expression::StringLiteral(_)
            ) {
                continue;
            }
            let origin = projector.path_of(initializer);
            out.parameter_defaults
                .entry(owner)
                .or_default()
                .insert(index, origin);
        }
    }
}
