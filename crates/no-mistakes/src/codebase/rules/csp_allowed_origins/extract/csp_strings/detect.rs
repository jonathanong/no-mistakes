use super::eval::unwrap_join_array_target;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, BinaryOperator, Expression, TemplateLiteral,
};

pub(super) fn array_contains_csp_fragment(array: &ArrayExpression<'_>) -> bool {
    array.elements.iter().any(|element| match element {
        ArrayExpressionElement::SpreadElement(spread) => {
            expr_contains_csp_fragment(&spread.argument)
        }
        other => other
            .as_expression()
            .is_some_and(expr_contains_csp_fragment),
    })
}

pub(super) fn template_contains_csp_fragment(template: &TemplateLiteral<'_>) -> bool {
    template.quasis.iter().any(|quasi| {
        quasi.value.cooked.is_some() && crate::ast::template_quasi_text(quasi).contains("-src")
    }) || template.expressions.iter().any(expr_contains_csp_fragment)
}

pub(super) fn expr_contains_csp_fragment(expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(lit) => crate::ast::js_literal(lit).contains("-src"),
        Expression::TemplateLiteral(tpl) => template_contains_csp_fragment(tpl),
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            expr_contains_csp_fragment(&bin.left) || expr_contains_csp_fragment(&bin.right)
        }
        Expression::ConditionalExpression(cond) => {
            expr_contains_csp_fragment(&cond.consequent)
                || expr_contains_csp_fragment(&cond.alternate)
        }
        Expression::LogicalExpression(logical) => {
            expr_contains_csp_fragment(&logical.left) || expr_contains_csp_fragment(&logical.right)
        }
        Expression::ArrayExpression(array) => array_contains_csp_fragment(array),
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee)
            else {
                return false;
            };
            !member.optional
                && member.property.name.as_str() == "join"
                && unwrap_join_array_target(&member.object)
                    .is_some_and(|(array, _)| array_contains_csp_fragment(array))
        }
        _ => false,
    }
}
