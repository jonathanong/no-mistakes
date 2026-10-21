use crate::codebase::dependencies::extract::ValueExpression;

pub(crate) fn walk_expression(expr: &ValueExpression, visit: &mut impl FnMut(&ValueExpression)) {
    visit(expr);
    match expr {
        ValueExpression::Awaited(inner) => walk_expression(inner, visit),
        ValueExpression::Member(inner, _) => walk_expression(inner, visit),
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            walk_expression(left, visit);
            walk_expression(right, visit);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            for value in fields.values() {
                walk_expression(value, visit);
            }
        }
        ValueExpression::Named { .. }
        | ValueExpression::Call(_, _)
        | ValueExpression::Callable(_)
        | ValueExpression::Super(_)
        | ValueExpression::Undefined
        | ValueExpression::Null
        | ValueExpression::Unknown => {}
    }
}

#[cfg(test)]
mod tests;
