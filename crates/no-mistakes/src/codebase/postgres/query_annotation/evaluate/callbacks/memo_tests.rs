use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn shared_callback_read_profiles_construct_one_projection_per_iteration() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-profile/shared.cjs",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: embedded.call_spans.iter().copied().collect(),
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
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let table = evaluator.scopes[root]["table"].clone();
    evaluator.active_callback_functions = Some(Default::default());
    evaluator.active_callback_executions = Some(Default::default());
    let mut state = super::CallbackState::default();
    evaluator.callback_values(&[table], 8, &mut state);
    assert_eq!(
        state.functions.len(),
        32,
        "every saved distinct callback participates"
    );
    assert_eq!(
        state.memo.constructions, 1,
        "shared read-only state is projected once initially"
    );
    let mut memo = super::memo::Memo::default();
    for (depth, value, before) in state.functions.values() {
        let Value::Function(function, path, captured) = value else {
            panic!("saved callback");
        };
        let current =
            evaluator.memoized_callback_snapshot(path, *captured, function, *depth, &mut memo);
        assert!(
            current == *before,
            "pure callbacks leave observed state unchanged"
        );
    }
    assert_eq!(
        memo.constructions, 1,
        "comparison shares the same callback table projection"
    );
    assert!(state.memo.constructions + memo.constructions <= 2);
}
