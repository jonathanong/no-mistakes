use super::super::super::{Evaluator, File};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn opaque_callback_siblings_each_execute_the_reader_against_their_own_state() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-alternatives/src/query.mts",
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
            executors: embedded.call_starts.iter().copied().collect(),
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
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    // A discarded earlier invocation forces module-root compaction to change
    // live environment IDs; seen keys must not preserve that discarded frame.
    let discarded = evaluator.environment(Default::default());
    let root = evaluator.module_environment(&path);
    let start = source.find("write(arguments[0])").unwrap() as u32;
    let events = evaluator
        .events
        .get(&(path.clone(), start))
        .expect("saved executor occurrence");
    assert_eq!(
        events.len(),
        2,
        "the first sibling invocation must not suppress the second"
    );
    assert!(evaluator.active_callback_functions.is_none());
    let super::super::super::Value::Function(function, callback_path, captured) =
        evaluator.scopes[root]["installer"].clone()
    else {
        panic!("saved installer function");
    };
    let key = (
        callback_path.clone(),
        function.start,
        captured,
        function.params.clone(),
    );
    let discarded_key = (
        callback_path,
        function.start,
        discarded,
        function.params.clone(),
    );
    evaluator.active_callback_functions =
        Some(crate::fx::FxHashSet::from_iter([key, discarded_key]));
    evaluator.active_callback_executions = evaluator.active_callback_functions.clone();
    evaluator.prune_snapshot_state();
    assert_eq!(
        evaluator.active_callback_executions,
        evaluator.active_callback_functions
    );
    let seen = evaluator.active_callback_functions.as_ref().unwrap();
    assert_eq!(seen.len(), 1, "seen keys never keep discarded frames alive");
    let live = seen.iter().next().unwrap();
    assert!(
        live.2 < captured,
        "captured environment key follows the module compaction mapping"
    );
    let module = evaluator.modules[&path];
    assert!(
        matches!(&evaluator.scopes[module]["installer"], super::super::super::Value::Function(_,_,env) if *env == live.2)
    );
}
