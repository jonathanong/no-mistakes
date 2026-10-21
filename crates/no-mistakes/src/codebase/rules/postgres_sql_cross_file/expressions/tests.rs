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
