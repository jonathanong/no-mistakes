use super::*;

pub(super) fn expression(
    expression: &Expression<'_>,
    name: &str,
    source: Option<&str>,
    project: bool,
    calls: &mut Vec<u32>,
) -> bool {
    let expression = expression.get_inner_expression();
    let expression = match expression {
        Expression::AwaitExpression(value) => value.argument.get_inner_expression(),
        value => value,
    };
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    let Expression::Identifier(receiver) = &member.object else {
        return false;
    };
    if !project && matches!(member.property.name.as_str(), "exit" | "abort") {
        return false;
    }
    if Some(receiver.name.as_str()) == source {
        return false;
    }
    if receiver.name == name {
        if !safe_arguments(call, name, source) {
            return false;
        }
        if project {
            calls.push(call.span.start);
        }
        true
    } else {
        if project {
            return false;
        }
        safe_arguments(call, name, source)
    }
}

pub(super) fn safe_arguments(call: &CallExpression<'_>, name: &str, source: Option<&str>) -> bool {
    call.arguments.iter().all(|argument| {
        argument.as_expression().is_some_and(|value| {
            !matches!(
                value.get_inner_expression(),
                Expression::ObjectExpression(_)
            ) && super::guards::read_only(value, name, source)
        })
    })
}
