use super::*;

#[test]
fn shared_containers_are_visited_once_per_reachability_collection() {
    let mut value = Value::Possible(
        vec![
            Value::Prefix("SELECT 1".into(), true, Some(7)),
            Value::Arguments(8),
        ]
        .into(),
    );
    // A shared DAG has exponentially many paths but only thirteen containers.
    for _ in 0..12 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    let mut reachable = Reachable::default();
    reachable.value(&value);
    reachable.value(&value);
    assert_eq!(reachable.visited_values.len(), 13);
    assert_eq!(reachable.identities, FxHashSet::from_iter([7]));
    assert_eq!(reachable.pending_args, vec![8]);
}

fn scalar_tree() -> Value {
    let mut value = Value::Possible(
        vec![
            Value::Prefix("SELECT 1".into(), true, None),
            Value::Primitive,
            Value::Unknown,
            Value::Unsupported,
            Value::SlotDeletion,
        ]
        .into(),
    );
    for _ in 0..12 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    value
}

fn saved_function() -> std::sync::Arc<crate::codebase::postgres::query_annotation::Function> {
    use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-profile/reads.mts",
    );
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
        panic!("saved function summary");
    };
    function
}

#[test]
fn scalar_containers_skip_reachability_walks_even_through_result_wrappers() {
    let scalar = scalar_tree();
    let mut reachable = Reachable::default();
    reachable.value(&scalar);
    assert_eq!(
        reachable.visited_values.len(),
        0,
        "no scalar containers are walked"
    );
    assert!(reachable.pending_env.is_empty());
    assert!(reachable.pending_args.is_empty());
    assert!(reachable.identities.is_empty());

    let wrapped = Value::Promise(Box::new(Value::Evaluated(Box::new(scalar), true)));
    let mut reachable = Reachable::default();
    reachable.value(&wrapped);
    assert_eq!(
        reachable.visited_values.len(),
        0,
        "wrappers do not walk scalar containers"
    );
}

#[test]
fn mixed_containers_keep_environment_argument_and_builder_roots() {
    let mut mixed = Value::Possible(
        vec![
            Value::Function(saved_function(), "reads.mts".into(), 42),
            Value::Arguments(8),
            Value::Prefix("SELECT 1".into(), true, Some(7)),
        ]
        .into(),
    );
    mixed = Value::Joined(vec![mixed].into());
    mixed = Value::Object(vec![mixed].into());
    mixed = Value::References(vec![mixed].into());
    mixed = Value::Aggregate(vec![mixed].into());
    mixed = Value::Promise(Box::new(Value::Evaluated(Box::new(mixed), true)));
    let value = Value::Aggregate(vec![scalar_tree(), mixed].into());
    let mut reachable = Reachable::default();
    reachable.value(&value);
    assert_eq!(reachable.pending_env, vec![42]);
    assert_eq!(reachable.pending_args, vec![8]);
    assert_eq!(reachable.identities, FxHashSet::from_iter([7]));
    assert_eq!(
        reachable.visited_values.len(),
        6,
        "the scalar sibling is skipped"
    );
}

#[test]
fn detached_function_only_container_retains_its_environment() {
    let mut values = super::super::super::value::Values::from(vec![scalar_tree()]);
    assert!(!values.contains_reference());
    assert!(!values.contains_environment());
    let scalar_snapshot = values.clone();
    values.push(Value::Promise(Box::new(Value::Evaluated(
        Box::new(Value::Function(saved_function(), "reads.mts".into(), 42)),
        true,
    ))));

    // Function captures require the environment summary even when the separate
    // builder/argument summary remains empty after a detached mutation.
    assert!(!values.contains_reference());
    let mut reachable = Reachable::default();
    reachable.value(&Value::Object(values));
    assert_eq!(reachable.pending_env, vec![42]);
    assert!(reachable.pending_args.is_empty());
    assert!(reachable.identities.is_empty());
    assert_eq!(reachable.visited_values.len(), 1);

    let mut previous = Reachable::default();
    previous.value(&Value::Object(scalar_snapshot));
    assert_eq!(previous.visited_values.len(), 0);
    assert!(previous.pending_env.is_empty());
}
