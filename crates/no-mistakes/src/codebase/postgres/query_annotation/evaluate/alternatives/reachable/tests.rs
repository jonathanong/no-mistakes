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
    reachable.value(&value, &FxHashMap::default());
    reachable.value(&value, &FxHashMap::default());
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
    reachable.value(&scalar, &FxHashMap::default());
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
    reachable.value(&wrapped, &FxHashMap::default());
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
            Value::Function(saved_function(), "reads.mts".into(), 42usize.into()),
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
    reachable.value(&value, &FxHashMap::default());
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
        Box::new(Value::Function(
            saved_function(),
            "reads.mts".into(),
            42usize.into(),
        )),
        true,
    ))));

    // Function captures require the environment summary even when the separate
    // builder/argument summary remains empty after a detached mutation.
    assert!(!values.contains_reference());
    let mut reachable = Reachable::default();
    reachable.value(&Value::Object(values), &FxHashMap::default());
    assert_eq!(reachable.pending_env, vec![42]);
    assert!(reachable.pending_args.is_empty());
    assert!(reachable.identities.is_empty());
    assert_eq!(reachable.visited_values.len(), 1);

    let mut previous = Reachable::default();
    previous.value(&Value::Object(scalar_snapshot), &FxHashMap::default());
    assert_eq!(previous.visited_values.len(), 0);
    assert!(previous.pending_env.is_empty());
}

#[test]
fn transient_roots_keep_private_environments_sparse_slots_and_argument_cycles() {
    let scopes = vec![
        Scope::default(),
        FxHashMap::from_iter([("args".into(), Value::Arguments(8))]).into(),
    ];
    let objects = FxHashMap::from_iter([(8, vec![Value::Arguments(8)]), (99, vec![])]);
    let extras = FxHashMap::from_iter([(
        8,
        std::collections::BTreeMap::from_iter([(
            4,
            Value::Prefix("SELECT 1".into(), true, Some(7)),
        )]),
    )]);
    let transient = [
        TransientRoot::Environment(1),
        TransientRoot::Argument(8),
        TransientRoot::Builder(9),
    ];
    let found = collect_from_roots(
        &scopes,
        RootSeeds {
            transient: &transient,
            ..RootSeeds::new(&Default::default())
        },
        &[],
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(found.environments, FxHashSet::from_iter([1]));
    assert_eq!(found.arguments, FxHashSet::from_iter([8]));
    assert_eq!(found.identities, FxHashSet::from_iter([7, 8, 9]));
    // Once operand evaluation completes, those otherwise unreachable identities die.
    let released = collect_from_roots(
        &scopes,
        RootSeeds::new(&Default::default()),
        &[],
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &Default::default(),
        &Default::default(),
    );
    assert!(released.environments.is_empty());
    assert!(released.arguments.is_empty());
    assert!(released.identities.is_empty());
}

#[test]
fn module_capture_liveness_follows_only_its_actual_owner_through_nested_values() {
    use crate::codebase::postgres::query_annotation::evaluate::Capture;
    let owner = std::path::PathBuf::from("reads.mts");
    let unrelated = std::path::PathBuf::from("unused.mts");
    let modules = FxHashMap::from_iter([(owner.clone(), 1), (unrelated, 2)]);
    let scopes = vec![
        Scope::default(),
        Scope::from(FxHashMap::from_iter([("owned".into(), Value::Array(42))])),
        Scope::from(FxHashMap::from_iter([(
            "unrelated".into(),
            Value::Array(99),
        )])),
    ];
    let objects = FxHashMap::from_iter([(42, vec![Value::Unknown]), (99, vec![Value::Unknown])]);
    let returned = [Value::Object(
        vec![Value::Promise(Box::new(Value::Function(
            saved_function(),
            owner,
            Capture::Module,
        )))]
        .into(),
    )];
    let found = collect_from_roots(
        &scopes,
        RootSeeds {
            ..RootSeeds::new(&modules)
        },
        &returned,
        Arena {
            objects: &objects,
            extras: &Default::default(),
        },
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(found.environments, FxHashSet::from_iter([1]));
    assert_eq!(found.arguments, FxHashSet::from_iter([42]));
}
