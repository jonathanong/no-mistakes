use super::Value;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::fx::FxHashMap;
use std::sync::Arc;

fn saved_function() -> Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-callback-profile/reads.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let query_annotation::Expr::Function(function) = facts.globals.remove("profile").unwrap()
    else {
        panic!("saved callback definition");
    };
    Value::Function(function, path.into(), 7)
}

#[test]
fn function_value_and_scope_clones_share_the_immutable_definition_path() {
    let value = saved_function();
    let Value::Function(function, path, env) = &value else {
        unreachable!()
    };
    let cloned = value.clone();
    let Value::Function(other_function, other_path, other_env) = &cloned else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(function, other_function));
    assert!(Arc::ptr_eq(path, other_path));
    assert_eq!(env, other_env);
    let scope = Scope::from(FxHashMap::from_iter([("callback".into(), value.clone())]));
    let mut snapshot = scope.clone();
    snapshot.insert("new binding".into(), Value::Unknown);
    let Value::Function(_, snapshot_path, _) = &snapshot["callback"] else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(path, snapshot_path));
    assert!(scope["callback"] == snapshot["callback"]);
}

#[test]
fn function_path_sharing_preserves_structural_equality_and_copy_on_write() {
    let value = saved_function();
    let Value::Function(function, path, env) = &value else {
        unreachable!()
    };
    let independent_path = Arc::new(path.as_ref().clone());
    assert!(!Arc::ptr_eq(path, &independent_path));
    assert!(value == Value::Function(function.clone(), independent_path, *env));
    assert!(value == Value::Function(Arc::new(function.as_ref().clone()), path.clone(), *env));
    assert!(value != Value::Function(function.clone(), path.clone(), env + 1));
    let mut changed_function = function.as_ref().clone();
    changed_function.supported = !changed_function.supported;
    assert!(value != Value::Function(Arc::new(changed_function), path.clone(), *env));
    let mut snapshot = value.clone();
    let Value::Function(_, snapshot_path, _) = &mut snapshot else {
        unreachable!()
    };
    // A detached synthetic edit must not change a captured snapshot's path.
    Arc::make_mut(snapshot_path).push("different.mts");
    assert!(!Arc::ptr_eq(path, snapshot_path));
    assert_ne!(path.as_path(), snapshot_path.as_path());
    assert!(value != snapshot);
    assert!(path.ends_with("helper-tracing-callback-profile/reads.mts"));
}
