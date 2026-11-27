//! `stubGlobal("process")` replaces the process object this pass records writes on.
use oxc_ast::ast::*;

pub(super) fn stubs_process(call: &CallExpression<'_>) -> bool {
    callee_is_stub_global(&call.callee) && !other_global(call.arguments.first())
}

/// A string literal other than `"process"` names a different global.
fn other_global(argument: Option<&Argument<'_>>) -> bool {
    let Some(expression) = argument
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        return false;
    };
    match expression {
        Expression::StringLiteral(text) => crate::ast::js_literal(text) != "process",
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .is_some_and(|text| matches!(text.as_str(), Some(value) if value != "process")),
        _ => false,
    }
}

fn callee_is_stub_global(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => id.name == "stubGlobal",
        Expression::StaticMemberExpression(member) => member.property.name == "stubGlobal",
        Expression::ComputedMemberExpression(member) => string_is_stub_global(&member.expression),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => member.property.name == "stubGlobal",
            ChainElement::ComputedMemberExpression(member) => {
                string_is_stub_global(&member.expression)
            }
            ChainElement::TSNonNullExpression(inner) => callee_is_stub_global(&inner.expression),
            ChainElement::CallExpression(_) | ChainElement::PrivateFieldExpression(_) => false,
        },
        _ => false,
    }
}

fn string_is_stub_global(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::StringLiteral(text) if crate::ast::js_literal(text) == "stubGlobal"
    )
}
