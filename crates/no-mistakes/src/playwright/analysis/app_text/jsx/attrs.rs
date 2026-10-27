use crate::ast::js_literal;
use oxc_ast::ast::{
    Expression, JSXAttribute, JSXAttributeItem, JSXAttributeValue, JSXExpression, JSXOpeningElement,
};
use oxc_span::GetSpan;

#[cfg(test)]
mod tests;

pub(crate) fn attr_exists_at_runtime(opening: &JSXOpeningElement<'_>, name: &str) -> bool {
    find_attr(opening, name).is_some_and(|attribute| match attribute.value.as_ref() {
        None => true,
        Some(JSXAttributeValue::StringLiteral(_)) => true,
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            jsx_expression_attribute_present(&container.expression)
        }
        _ => false,
    })
}

pub(crate) fn aria_bool_attr(opening: &JSXOpeningElement<'_>, name: &str) -> Option<bool> {
    find_attr(opening, name).and_then(|attribute| match attribute.value.as_ref()? {
        JSXAttributeValue::StringLiteral(literal) => match js_literal(literal).as_ref() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            JSXExpression::StringLiteral(literal) => match js_literal(literal).as_ref() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            expression => bool_expr(expression),
        },
        _ => None,
    })
}

pub(crate) fn bool_attr(opening: &JSXOpeningElement<'_>, name: &str) -> Option<bool> {
    find_attr(opening, name).and_then(|attribute| match attribute.value.as_ref() {
        None => Some(true),
        Some(JSXAttributeValue::ExpressionContainer(container)) => bool_expr(&container.expression),
        _ => Some(true),
    })
}

pub(crate) fn numeric_attr(
    opening: &JSXOpeningElement<'_>,
    name: &str,
    source: &str,
) -> Option<u32> {
    find_attr(opening, name).and_then(|attribute| numeric_attr_value(attribute, source))
}

fn find_attr<'a>(opening: &'a JSXOpeningElement<'_>, name: &str) -> Option<&'a JSXAttribute<'a>> {
    opening
        .attributes
        .iter()
        .find_map(|item| {
            let JSXAttributeItem::Attribute(attribute) = item else {
                return None;
            };
            (super::jsx_attribute_name(&attribute.name) == Some(name)).then_some(attribute)
        })
        .map(|attribute| &**attribute)
}

fn numeric_attr_value(attribute: &JSXAttribute<'_>, source: &str) -> Option<u32> {
    match attribute.value.as_ref()? {
        JSXAttributeValue::StringLiteral(literal) => js_literal(literal).parse::<u32>().ok(),
        JSXAttributeValue::ExpressionContainer(container) => numeric_expr(&container.expression)
            .or_else(|| {
                let span = container.expression.span();
                source
                    .get(span.start as usize..span.end as usize)
                    .and_then(|value| value.parse::<u32>().ok())
            }),
        _ => None,
    }
}

fn bool_expr(expression: &JSXExpression<'_>) -> Option<bool> {
    match expression {
        JSXExpression::BooleanLiteral(literal) => Some(literal.value),
        JSXExpression::NullLiteral(_) => Some(false),
        JSXExpression::NumericLiteral(literal) => Some(literal.value != 0.0),
        JSXExpression::StringLiteral(literal) => Some(!literal.value.is_empty()),
        JSXExpression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .map(|quasi| !quasi.value.raw.is_empty()),
        JSXExpression::Identifier(identifier) if identifier.name.as_str() == "undefined" => {
            Some(false)
        }
        JSXExpression::TSAsExpression(expression) => bool_expression(&expression.expression),
        JSXExpression::TSSatisfiesExpression(expression) => bool_expression(&expression.expression),
        JSXExpression::TSNonNullExpression(expression) => bool_expression(&expression.expression),
        // TSX cannot parse `<T>expr` type assertions; those are `as`/`satisfies`.
        _ => None,
    }
}

fn jsx_expression_attribute_present(expression: &JSXExpression<'_>) -> bool {
    match expression {
        JSXExpression::NullLiteral(_) => false,
        JSXExpression::Identifier(identifier) => identifier.name.as_str() != "undefined",
        JSXExpression::TSAsExpression(expression) => {
            expression_attribute_present(&expression.expression)
        }
        JSXExpression::TSSatisfiesExpression(expression) => {
            expression_attribute_present(&expression.expression)
        }
        JSXExpression::TSNonNullExpression(expression) => {
            expression_attribute_present(&expression.expression)
        }
        // TSX cannot parse `<T>expr` type assertions; those are `as`/`satisfies`.
        _ => true,
    }
}

fn numeric_expr(expression: &JSXExpression<'_>) -> Option<u32> {
    match expression {
        JSXExpression::NumericLiteral(literal) if literal.value >= 0.0 => {
            Some(literal.value as u32)
        }
        JSXExpression::TSAsExpression(expression) => numeric_expression(&expression.expression),
        JSXExpression::TSSatisfiesExpression(expression) => {
            numeric_expression(&expression.expression)
        }
        JSXExpression::TSNonNullExpression(expression) => {
            numeric_expression(&expression.expression)
        }
        // TSX cannot parse `<T>expr` type assertions; those are `as`/`satisfies`.
        _ => None,
    }
}

fn bool_expression(expression: &Expression<'_>) -> Option<bool> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        Expression::NullLiteral(_) => Some(false),
        Expression::NumericLiteral(literal) => Some(literal.value != 0.0),
        Expression::StringLiteral(literal) => Some(!literal.value.is_empty()),
        Expression::Identifier(identifier) if identifier.name.as_str() == "undefined" => {
            Some(false)
        }
        _ => None,
    }
}

fn expression_attribute_present(expression: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::NullLiteral(_) => false,
        Expression::Identifier(identifier) => identifier.name.as_str() != "undefined",
        _ => true,
    }
}

fn numeric_expression(expression: &Expression<'_>) -> Option<u32> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::NumericLiteral(literal) if literal.value >= 0.0 => Some(literal.value as u32),
        _ => None,
    }
}
