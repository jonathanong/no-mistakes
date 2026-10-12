use super::super::{Environment, Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn references(value: &Value, callbacks: &mut Vec<Environment>, arguments: &mut Vec<u64>) {
    match value {
        Value::Function(_, _, env) => callbacks.push(*env),
        Value::Arguments(id) => arguments.push(*id),
        Value::Promise(value) | Value::Evaluated(value, _) => {
            references(value, callbacks, arguments)
        }
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Joined(values)
        | Value::Possible(values) => {
            for value in values {
                references(value, callbacks, arguments);
            }
        }
        _ => {}
    }
}

#[test]
fn saved_dense_helper_retains_callback_and_cyclic_argument_state() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-branch-copies/src/dense.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
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
        active_module_initials: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let retained = evaluator.scopes[root].get("retained").expect("saved state");
    let mut callbacks = Vec::new();
    let mut arguments = Vec::new();
    references(retained, &mut callbacks, &mut arguments);
    assert!(
        !callbacks.is_empty(),
        "the selected callback remains reachable"
    );
    assert!(callbacks.iter().all(|env| *env < evaluator.scopes.len()));
    assert!(
        !arguments.is_empty(),
        "the arguments cycle remains reachable"
    );
    assert!(arguments.iter().any(|id| {
        evaluator
            .argument_objects
            .get(id)
            .is_some_and(|slots| slots.len() == 3)
            && evaluator
                .argument_extra_slots
                .get(id)
                .is_some_and(|extras| extras.get(&1000) == Some(&Value::Arguments(*id)))
    }));
}
