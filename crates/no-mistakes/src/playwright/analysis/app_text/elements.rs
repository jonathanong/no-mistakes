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
