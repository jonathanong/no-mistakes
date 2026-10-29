use super::*;

#[path = "mock_boundaries_bindings.rs"]
mod bindings;
#[path = "mock_boundaries_factory.rs"]
mod factory;

pub(super) fn collect(
    call: &CallExpression<'_>,
    node: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    if factory::actual_loader(call) {
        facts.module_mock_actual_spans.push(call.span.into());
    }
    let Some((method, optional)) = bindings::mock_method(call, semantic) else {
        return;
    };
    let specifier = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(|expression| match expression.get_inner_expression() {
            Expression::StringLiteral(literal) => {
                Some(crate::ast::js_literal(literal).into_owned())
            }
            _ => None,
        });
    let replacement = call
        .arguments
        .get(1)
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression);
    let factory_span = replacement.and_then(|value| match value {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            Some(value.span().into())
        }
        _ => None,
    });
    let hoisted = method == "mock";
    let full = hoisted
        && top_level(node, semantic)
        && !call.optional
        && !optional
        && specifier.is_some()
        && match call.arguments.len() {
            1 => true,
            2 => replacement.is_some_and(factory::isolated),
            _ => false,
        };
    facts.module_mock_calls.push(ModuleMockCall {
        automock: call.arguments.len() == 1,
        hoisted,
        specifier,
        factory_span,
        full,
    });
}

fn top_level(node: oxc_semantic::NodeId, semantic: &Semantic<'_>) -> bool {
    let mut ancestors = semantic.nodes().ancestor_kinds(node).filter(|kind| {
        !matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    });
    matches!(ancestors.next(), Some(AstKind::ExpressionStatement(_)))
        && matches!(ancestors.next(), Some(AstKind::Program(_)))
}
