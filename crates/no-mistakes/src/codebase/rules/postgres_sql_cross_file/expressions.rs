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
mod tests {
    use super::*;

    #[test]
    fn walks_member_and_object_trees() {
        let expr = ValueExpression::Member(
            std::sync::Arc::new(ValueExpression::Named {
                name: "config".into(),
                scope: None,
                offset: 4,
                context: None,
            }),
            "schema".into(),
        );
        let mut names = Vec::new();
        walk_expression(&expr, &mut |value| {
            if let ValueExpression::Named { name, .. } = value {
                names.push(name.clone());
            }
        });
        assert_eq!(names, ["config"]);
    }
}
