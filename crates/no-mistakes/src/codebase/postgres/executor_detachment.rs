use oxc_ast::ast::*;

pub(crate) fn executor_receiver<'e, 'a>(
    expression: &'e Expression<'a>,
    names: &[String],
) -> Option<&'e Expression<'a>> {
    let (receiver, name) = static_member(expression)?;
    names
        .iter()
        .any(|selected| selected == &name)
        .then_some(receiver)
}

pub(crate) fn bind_method<'e, 'a>(call: &'e CallExpression<'a>) -> Option<&'e Expression<'a>> {
    bind_callee(&call.callee)
}

/// The expression a call invokes through `.call`, `.apply`, `.bind` or `Reflect.apply`.
pub(crate) fn invoked_reference<'e, 'a>(
    call: &'e CallExpression<'a>,
) -> Option<&'e Expression<'a>> {
    if let Expression::SequenceExpression(sequence) = call.callee.get_inner_expression() {
        return sequence.expressions.last();
    }
    match static_member(&call.callee)? {
        (Expression::Identifier(reflect), name) if reflect.name == "Reflect" && name == "apply" => {
            call.arguments.first()?.as_expression()
        }
        (object, name) if matches!(name.as_str(), "call" | "apply" | "bind") => Some(object),
        _ => None,
    }
}

pub(crate) fn bind_callee<'e, 'a>(callee: &'e Expression<'a>) -> Option<&'e Expression<'a>> {
    let (method, name) = static_member(callee)?;
    (name == "bind").then_some(method)
}

pub(crate) fn executor_bindings<'e, 'a>(
    pattern: &'e BindingPattern<'a>,
    names: &[String],
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
            if key
                .as_deref()
                .is_some_and(|key| !names.iter().any(|name| name == key))
            {
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
            Some((local.as_ref(), key.is_some() && plain))
        })
        .collect()
}

pub(crate) fn static_member<'e, 'a>(
    expression: &'e Expression<'a>,
) -> Option<(&'e Expression<'a>, String)> {
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
