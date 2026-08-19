use oxc_ast::ast::Expression;

fn hook_callee_name(expr: &oxc_ast::ast::CallExpression<'_>) -> Option<String> {
    match &expr.callee {
        Expression::Identifier(id) => Some(id.name.as_ref().to_string()),
        Expression::StaticMemberExpression(m) => {
            if matches!(&m.object, Expression::Identifier(id) if id.name == "React") {
                Some(m.property.name.as_ref().to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn call_sets_state(expr: &oxc_ast::ast::CallExpression<'_>) -> bool {
    hook_callee_name(expr).is_some_and(|name| {
        matches!(
            name.as_str(),
            "useState" | "useReducer" | "useOptimistic" | "useSyncExternalStore"
        )
    })
}

pub(crate) fn member_is_this_state(expr: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    matches!(&expr.object, Expression::ThisExpression(_))
        && matches!(expr.property.name.as_ref(), "state" | "setState")
}

#[cfg(test)]
mod tests;
