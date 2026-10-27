use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;

#[test]
fn weak_epochs_follow_shared_and_unique_scope_writes() {
    let mut scope = Scope::default();
    scope.insert("value".into(), Value::Number(1));
    let initial = scope.state();
    let shared = scope.clone();
    assert!(initial.matches(&scope));
    assert!(initial.matches(&shared));
    scope.insert("value".into(), Value::Number(2));
    assert!(!initial.matches(&scope));
    assert!(initial.matches(&shared));
    let unique = scope.state();
    assert_eq!(Arc::strong_count(&scope.0), 1);
    // Even an equal write must dissociate weak epochs before taking &mut.
    scope.insert("value".into(), Value::Number(2));
    assert!(!unique.matches(&scope));
    assert!(scope.state().matches(&scope));
}

#[test]
fn weak_epochs_do_not_keep_dead_bindings_or_function_definitions_alive() {
    use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
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
        unreachable!()
    };
    let function_weak = Arc::downgrade(&function);
    let mut scope = Scope::default();
    scope.insert("callback".into(), Value::Function(function, path.into(), 0));
    let state = scope.state();
    assert_eq!(state.0.strong_count(), 1);
    drop(scope);
    drop(facts);
    assert_eq!(state.0.strong_count(), 0);
    assert!(function_weak.upgrade().is_none());
    assert!(!state.matches(&Scope::default()));
}
