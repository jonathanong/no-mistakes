use oxc_ast::ast::JSXElementName;

pub(crate) fn jsx_is_context_provider(elem: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
    match &elem.name {
        JSXElementName::MemberExpression(m) if m.property.name == "Provider" => true,
        JSXElementName::IdentifierReference(id) if id.name == "Provider" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
