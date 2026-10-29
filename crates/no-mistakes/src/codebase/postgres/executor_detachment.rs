use oxc_ast::ast::*;

pub(crate) fn query_receiver<'e, 'a>(expression: &'e Expression<'a>) -> Option<&'e Expression<'a>> {
    let (receiver, name) = static_member(expression)?;
    (name == "query").then_some(receiver)
}

pub(crate) fn bind_method<'e, 'a>(call: &'e CallExpression<'a>) -> Option<&'e Expression<'a>> {
    bind_callee(&call.callee)
}

pub(crate) fn bind_callee<'e, 'a>(callee: &'e Expression<'a>) -> Option<&'e Expression<'a>> {
    let (method, name) = static_member(callee)?;
    (name == "bind").then_some(method)
}

/// A false proof still retains a possible query when a default or dynamic key intervenes.
pub(crate) fn query_bindings<'e, 'a>(
    pattern: &'e BindingPattern<'a>,
) -> Vec<(&'e BindingIdentifier<'a>, bool)> {
    let BindingPattern::ObjectPattern(object) = pattern else {
        return Vec::new();
    };
    object
        .properties
        .iter()
        .filter_map(|property| {
            let key = if property.computed {
                property.key.as_expression().and_then(literal_name)
            } else {
                crate::ast::property_key_name(&property.key)
            };
            if key.as_deref().is_some_and(|key| key != "query") {
                return None;
            }
            let (local, plain) = match &property.value {
                BindingPattern::BindingIdentifier(local) => (local, true),
                BindingPattern::AssignmentPattern(default) => {
                    let BindingPattern::BindingIdentifier(local) = &default.left else {
                        return None;
                    };
                    (local, false)
                }
                _ => return None,
            };
            Some((local.as_ref(), key.as_deref() == Some("query") && plain))
        })
        .collect()
}

fn static_member<'e, 'a>(expression: &'e Expression<'a>) -> Option<(&'e Expression<'a>, String)> {
    match expression.get_inner_expression() {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.to_string()))
        }
        Expression::ComputedMemberExpression(member) => Some((
            &member.object,
            literal_name(&member.expression)?.into_owned(),
        )),
        _ => None,
    }
}

fn literal_name<'e>(expression: &'e Expression<'_>) -> Option<std::borrow::Cow<'e, str>> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value)),
        Expression::TemplateLiteral(value)
            if value.expressions.is_empty() && value.quasis.len() == 1 =>
        {
            Some(crate::ast::template_quasi_text(&value.quasis[0]))
        }
        _ => None,
    }
}
