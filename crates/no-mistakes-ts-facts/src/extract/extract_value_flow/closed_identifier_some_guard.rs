use oxc_ast::ast::{BindingPattern, CallExpression, Expression};
use oxc_syntax::operator::BinaryOperator;

pub(in super::super) fn some_subject(call: &CallExpression<'_>) -> Option<(String, u32)> {
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return None;
    };
    if member.optional || member.property.name != "some" {
        return None;
    }
    let Expression::ArrowFunctionExpression(arrow) = call.arguments[0].as_expression()? else {
        return None;
    };
    if arrow.r#async || arrow.params.items.len() != 1 || arrow.params.rest.is_some() {
        return None;
    }
    let parameter = &arrow.params.items[0];
    let BindingPattern::BindingIdentifier(item) = &parameter.pattern else {
        return None;
    };
    if parameter.initializer.is_some() || parameter.optional || !parameter.decorators.is_empty() {
        return None;
    }
    let Expression::BinaryExpression(equality) = arrow.body.as_expression()?.get_inner_expression()
    else {
        return None;
    };
    if equality.operator != BinaryOperator::StrictEquality {
        return None;
    }
    for (element, candidate) in [
        (&equality.left, &equality.right),
        (&equality.right, &equality.left),
    ] {
        let Expression::Identifier(element) = element.get_inner_expression() else {
            continue;
        };
        let Expression::Identifier(candidate) = candidate.get_inner_expression() else {
            continue;
        };
        if element.name == item.name && candidate.name != item.name {
            return Some((candidate.name.to_string(), candidate.span.start));
        }
    }
    None
}
