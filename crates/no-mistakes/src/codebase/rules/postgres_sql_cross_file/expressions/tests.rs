use super::*;
use std::sync::Arc;

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: None,
        offset: 4,
        context: None,
    }
}

fn names(expr: &ValueExpression) -> Vec<String> {
    let mut found = Vec::new();
    walk_expression(expr, &mut |value| {
        if let ValueExpression::Named { name, .. } = value {
            found.push(name.clone());
        }
    });
    found
}

#[test]
fn walks_member_awaited_alternatives_and_containers() {
    let member = ValueExpression::Member(Arc::new(named("config")), "schema".into());
    assert_eq!(names(&member), ["config"]);
    assert_eq!(
        names(&ValueExpression::Awaited(Arc::new(named("inner")))),
        ["inner"]
    );
    let left = Arc::new(named("left"));
    let right = Arc::new(named("right"));
    assert_eq!(
        names(&ValueExpression::Nullish(left.clone(), right.clone())),
        ["left", "right"]
    );
    assert_eq!(
        names(&ValueExpression::Default(left.clone(), right.clone())),
        ["left", "right"]
    );
    assert_eq!(
        names(&ValueExpression::Either(left, right)),
        ["left", "right"]
    );
    let mut object_fields = crate::fx::fx_map();
    object_fields.insert("schema".into(), named("obj"));
    assert_eq!(
        names(&ValueExpression::Object(Arc::new(object_fields), false)),
        ["obj"]
    );
    let mut array_fields = crate::fx::fx_map();
    array_fields.insert("0".into(), named("el"));
    assert_eq!(
        names(&ValueExpression::Array(Arc::new(array_fields))),
        ["el"]
    );
}

#[test]
fn walks_opaque_derived_inputs_without_losing_nested_provenance() {
    let expression = ValueExpression::Derived(
        vec![
            named("schema"),
            ValueExpression::Derived(vec![named("table")].into()),
            ValueExpression::Unknown,
        ]
        .into(),
    );
    assert_eq!(names(&expression), ["schema", "table"]);
}
