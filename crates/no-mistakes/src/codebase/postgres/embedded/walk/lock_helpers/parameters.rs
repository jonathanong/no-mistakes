use super::{summary::Parameter, *};
pub(super) fn parameter(expression: &Expression<'_>, params: &[&str]) -> Parameter {
    if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
        return params
            .iter()
            .position(|param| *param == id.name.as_str())
            .map(Parameter::Argument)
            .unwrap_or(Parameter::Unknown);
    }
    super::super::transaction::literal_identity(expression)
        .map(Parameter::Literal)
        .unwrap_or(Parameter::Unknown)
}
pub(super) fn compose(
    helper: &Helper,
    call: &CallExpression<'_>,
    params: &[&str],
) -> Option<Vec<summary::Query>> {
    for (index, argument) in call.arguments.iter().enumerate() {
        let expression = unwrap_ts_wrappers(argument.as_expression()?);
        if !matches!(parameter(expression, params), Parameter::Unknown) {
            continue;
        }
        let Expression::StaticMemberExpression(member) = expression else {
            return None;
        };
        if member.optional
            || member.property.name != "query"
            || !matches!(parameter(&member.object, params), Parameter::Argument(_))
            || !helper
                .calls
                .iter()
                .any(|query| query.executor == index && query.access == ExecutorAccess::Callable)
        {
            return None;
        }
    }
    helper
        .calls
        .iter()
        .map(|query| {
            let argument = unwrap_ts_wrappers(call.arguments.get(query.executor)?.as_expression()?);
            let (receiver, access) = match argument {
                Expression::Identifier(_) => (argument, query.access),
                Expression::StaticMemberExpression(member)
                    if !member.optional
                        && member.property.name == "query"
                        && query.access == ExecutorAccess::Callable =>
                {
                    (unwrap_ts_wrappers(&member.object), ExecutorAccess::Object)
                }
                _ => return None,
            };
            let Parameter::Argument(executor) = parameter(receiver, params) else {
                return None;
            };
            let mut projected = query.clone();
            projected.executor = executor;
            projected.access = access;
            projected.parameters = query
                .parameters
                .iter()
                .map(|value| match value {
                    Parameter::Argument(index) => call
                        .arguments
                        .get(*index)
                        .and_then(|argument| argument.as_expression())
                        .map(|expression| parameter(expression, params))
                        .unwrap_or(Parameter::Unknown),
                    value => value.clone(),
                })
                .collect();
            Some(projected)
        })
        .collect()
}
