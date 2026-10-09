use super::super::{Evaluator, File};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn evaluated_prefix_comparison_rejects_a_lost_annotation_on_the_same_alias() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-live-binding/src/prefix-transitions.cjs",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: Default::default(),
            imports: Default::default(),
            exports: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        deleted_argument_slots: Default::default(),
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    use super::super::Value;
    let before = evaluator.scopes[root]["annotated"].clone();
    let Value::Prefix(_, _, Some(id)) = &before else {
        panic!("saved annotated builder");
    };
    let Value::Prefix(text, complete, _) = &evaluator.scopes[root]["bare"] else {
        panic!("saved bare prefix");
    };
    // Compare the saved states under one alias identity. Losing an annotation
    // must invalidate that identity even behind an evaluated-effect wrapper.
    let after = Value::Prefix(text.clone(), *complete, Some(*id));
    let mut changed = crate::fx::FxHashSet::default();
    super::values::changes(
        &Value::Evaluated(Box::new(before.clone()), true),
        &Value::Evaluated(Box::new(after), true),
        super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        &mut changed,
        &Default::default(),
    );
    assert!(changed.contains(id));
    let mut rebound = crate::fx::FxHashSet::default();
    let replacement = Value::Prefix("replacement".into(), true, None);
    super::values::changes(
        &before,
        &replacement,
        super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        &mut rebound,
        &Default::default(),
    );
    assert!(
        rebound.is_empty(),
        "replacing a binding does not mutate its previous builder"
    );

    // Apply the detected alias mutation to the saved actual frame, retaining
    // unrelated prefixes and function bindings in the same scope.
    let mut scopes = vec![evaluator.scopes[root].clone()];
    super::values::apply_taint(&mut scopes, &changed);
    assert!(matches!(scopes[0]["annotated"], Value::Unknown));
    assert!(matches!(scopes[0]["bare"], Value::Prefix(_, _, Some(_))));
    let callback = evaluator.scopes[root]["callback"].clone();
    let mut dense_joined = FxHashMap::default();
    super::merge::objects(
        &mut dense_joined,
        &FxHashMap::from_iter([(1000, vec![callback.clone()])]),
    );
    super::merge::objects(
        &mut dense_joined,
        &FxHashMap::from_iter([(1000, vec![Value::Unknown])]),
    );
    assert!(
        matches!(&dense_joined[&1000][0], Value::Aggregate(values) if values.contains(&callback) && values.contains(&Value::Unknown))
    );
    let mut sparse_joined = None;
    let first = FxHashMap::from_iter([(
        1000,
        std::collections::BTreeMap::from([(2000, callback.clone())]),
    )]);
    super::extras::join(&mut sparse_joined, &first, &Default::default());
    let second = FxHashMap::from_iter([(
        1000,
        std::collections::BTreeMap::from([(2000, Value::Unknown)]),
    )]);
    super::extras::join(&mut sparse_joined, &second, &Default::default());
    // A genuinely distinct arm-created container retains its own callback,
    // rather than joining an absent slot from a different runtime identity.
    let later = FxHashMap::from_iter([(
        1001,
        std::collections::BTreeMap::from([(2000, callback.clone())]),
    )]);
    super::extras::join(&mut sparse_joined, &later, &Default::default());
    let joined = sparse_joined.unwrap();
    assert!(matches!(&joined[&1001][&2000], Value::Function(..)));
    assert!(
        matches!(&joined[&1000][&2000], Value::Aggregate(values) if values.contains(&callback) && values.contains(&Value::Unknown))
    );
}
