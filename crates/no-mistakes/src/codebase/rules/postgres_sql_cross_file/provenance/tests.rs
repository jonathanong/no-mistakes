use super::*;
use crate::codebase::dependencies::extract::ValueExpression;
use std::sync::Arc;

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: None,
        offset: 0,
        context: None,
    }
}

fn member(inner: ValueExpression, key: &str) -> ValueExpression {
    ValueExpression::Member(Arc::new(inner), key.into())
}

#[test]
fn process_env_requires_the_process_env_member_chain() {
    assert!(is_process_env(&member(
        member(named("process"), "env"),
        "SCHEMA"
    )));
    assert!(!is_process_env(&member(
        member(named("process"), "argv"),
        "0"
    )));
    assert!(!is_process_env(&member(
        member(named("config"), "env"),
        "SCHEMA"
    )));
}

#[test]
fn process_specifiers_include_node_and_bare_process() {
    assert!(is_process_specifier("node:process"));
    assert!(is_process_specifier("process"));
    assert!(!is_process_specifier("./process"));
}
