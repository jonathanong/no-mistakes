use oxc_span::GetSpan;

pub(crate) fn attr_exists_at_runtime(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
) -> bool {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if super::jsx_attribute_name(&attribute.name) != Some(name) {
            continue;
        }
        return match attribute.value.as_ref() {
            None => true,
            Some(oxc_ast::ast::JSXAttributeValue::StringLiteral(literal)) => {
                !literal.value.is_empty()
            }
            Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) => {
                jsx_expression_truthy(&container.expression)
            }
            _ => false,
        };
    }
    false
}

pub(crate) fn bool_attr(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> Option<bool> {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if super::jsx_attribute_name(&attribute.name) != Some(name) {
            continue;
        }
        return Some(match attribute.value.as_ref() {
            None => true,
            Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) => {
                matches!(
                    &container.expression,
                    oxc_ast::ast::JSXExpression::BooleanLiteral(literal) if literal.value
                )
            }
            _ => true,
        });
    }
    None
}

pub(crate) fn numeric_attr(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
    source: &str,
) -> Option<u32> {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if super::jsx_attribute_name(&attribute.name) != Some(name) {
            continue;
        }
        return match attribute.value.as_ref()? {
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
        };
    }
    None
}

fn jsx_expression_truthy(expression: &oxc_ast::ast::JSXExpression<'_>) -> bool {
    match expression {
        oxc_ast::ast::JSXExpression::NullLiteral(_) => false,
        oxc_ast::ast::JSXExpression::BooleanLiteral(literal) => literal.value,
        oxc_ast::ast::JSXExpression::Identifier(identifier) => {
            identifier.name.as_str() != "undefined"
        }
        _ => true,
    }
}
