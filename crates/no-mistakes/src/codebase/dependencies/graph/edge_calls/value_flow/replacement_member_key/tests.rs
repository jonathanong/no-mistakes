use super::*;
use std::sync::Arc;

fn named(name: &str, scope: Option<usize>, offset: u32) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope,
        offset,
        context: Some(CallableId(offset)),
    }
}
fn member(name: &str, scope: Option<usize>, property: &str) -> ValueExpression {
    ValueExpression::Member(Arc::new(named(name, scope, 10)), property.into())
}

#[test]
fn shallow_keys_keep_shape_scope_and_property_but_ignore_unused_source_coordinates() {
    assert!(same_member_root(
        &named("r", None, 10),
        &named("r", None, 20)
    ));
    assert!(!same_member_root(
        &named("r", None, 10),
        &named("r", Some(0), 10)
    ));
    assert!(!same_member_root(
        &named("r", None, 10),
        &named("s", None, 10)
    ));
    assert!(same_member_root(
        &member("r", Some(1), "get"),
        &member("r", Some(1), "get")
    ));
    assert!(!same_member_root(
        &member("r", Some(1), "get"),
        &member("r", Some(2), "get")
    ));
    assert!(!same_member_root(
        &member("r", Some(1), "get"),
        &member("r", Some(1), "set")
    ));
    // Dotted Named and Member roots enter the depth-limited matcher differently.
    assert!(!same_member_root(
        &named("r.get", None, 10),
        &member("r", None, "get")
    ));
    assert!(!same_member_root(
        &ValueExpression::Unknown,
        &ValueExpression::Unknown
    ));
    let nested = ValueExpression::Member(Arc::new(member("r", None, "get")), "call".into());
    assert!(!same_member_root(&nested, &nested));
}
