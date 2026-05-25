pub(super) fn jsx_element_name<'a>(name: &'a oxc_ast::ast::JSXElementName<'a>) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::JSXElementName::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        oxc_ast::ast::JSXElementName::MemberExpression(expression) => {
            jsx_member_expression_root(expression)
        }
        _ => None,
    }
}

pub(super) fn is_component_jsx_element_name(name: &oxc_ast::ast::JSXElementName<'_>) -> bool {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(identifier) => identifier
            .name
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii_lowercase()),
        oxc_ast::ast::JSXElementName::IdentifierReference(identifier) => identifier
            .name
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii_lowercase()),
        oxc_ast::ast::JSXElementName::MemberExpression(_) => true,
        oxc_ast::ast::JSXElementName::NamespacedName(_)
        | oxc_ast::ast::JSXElementName::ThisExpression(_) => false,
    }
}

fn jsx_member_expression_root<'a>(
    expression: &'a oxc_ast::ast::JSXMemberExpression<'a>,
) -> Option<&'a str> {
    match &expression.object {
        oxc_ast::ast::JSXMemberExpressionObject::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        oxc_ast::ast::JSXMemberExpressionObject::MemberExpression(expression) => {
            jsx_member_expression_root(expression)
        }
        oxc_ast::ast::JSXMemberExpressionObject::ThisExpression(_) => None,
    }
}
