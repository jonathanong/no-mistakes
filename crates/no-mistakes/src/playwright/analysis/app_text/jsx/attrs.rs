use oxc_span::GetSpan;

pub(crate) fn attr_exists_at_runtime(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
) -> bool {
    find_attr(opening, name).is_some_and(|attribute| match attribute.value.as_ref() {
        None => true,
        Some(oxc_ast::ast::JSXAttributeValue::StringLiteral(_)) => true,
        Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) => {
            jsx_expression_attribute_present(&container.expression)
        }
        _ => false,
    })
}

pub(crate) fn aria_bool_attr(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
) -> Option<bool> {
    find_attr(opening, name).and_then(|attribute| match attribute.value.as_ref()? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => match literal.value.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => match &container
            .expression
        {
            oxc_ast::ast::JSXExpression::StringLiteral(literal) => match literal.value.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            expression => bool_expr(expression),
        },
        _ => None,
    })
}

pub(crate) fn bool_attr(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> Option<bool> {
    find_attr(opening, name).and_then(|attribute| match attribute.value.as_ref() {
        None => Some(true),
        Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) => {
            bool_expr(&container.expression)
        }
        _ => Some(true),
    })
}

pub(crate) fn numeric_attr(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
    source: &str,
) -> Option<u32> {
    find_attr(opening, name).and_then(|attribute| numeric_attr_value(attribute, source))
}

fn find_attr<'a>(
    opening: &'a oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
) -> Option<&'a oxc_ast::ast::JSXAttribute<'a>> {
    opening
        .attributes
        .iter()
        .find_map(|item| {
            let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
                return None;
            };
            (super::jsx_attribute_name(&attribute.name) == Some(name)).then_some(attribute)
        })
        .map(|attribute| &**attribute)
}

fn numeric_attr_value(attribute: &oxc_ast::ast::JSXAttribute<'_>, source: &str) -> Option<u32> {
    match attribute.value.as_ref()? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => {
            literal.value.parse::<u32>().ok()
        }
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            let span = container.expression.span();
            source
                .get(span.start as usize..span.end as usize)
                .and_then(|value| value.parse::<u32>().ok())
        }
        _ => None,
    }
}

fn bool_expr(expression: &oxc_ast::ast::JSXExpression<'_>) -> Option<bool> {
    match expression {
        oxc_ast::ast::JSXExpression::BooleanLiteral(literal) => Some(literal.value),
        oxc_ast::ast::JSXExpression::NullLiteral(_) => Some(false),
        oxc_ast::ast::JSXExpression::NumericLiteral(literal) => Some(literal.value != 0.0),
        oxc_ast::ast::JSXExpression::StringLiteral(literal) => Some(!literal.value.is_empty()),
        oxc_ast::ast::JSXExpression::TemplateLiteral(template)
            if template.expressions.is_empty() =>
        {
            template
                .quasis
                .first()
                .map(|quasi| !quasi.value.raw.is_empty())
        }
        oxc_ast::ast::JSXExpression::Identifier(identifier)
            if identifier.name.as_str() == "undefined" =>
        {
            Some(false)
        }
        _ => None,
    }
}

fn jsx_expression_attribute_present(expression: &oxc_ast::ast::JSXExpression<'_>) -> bool {
    match expression {
        oxc_ast::ast::JSXExpression::NullLiteral(_) => false,
        oxc_ast::ast::JSXExpression::Identifier(identifier) => {
            identifier.name.as_str() != "undefined"
        }
        _ => true,
    }
}
