use super::super::super::{Evaluator, File};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn stateful_callback_revisits_observe_changed_lexical_arguments() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-revisit/src/query.cjs",
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
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let start = source.find("database.query(saved)").unwrap() as u32;
    let events = evaluator
        .events
        .get(&(path.clone(), start))
        .expect("saved query occurrence");
    assert!(
        events.len() >= 2,
        "a changed captured slot permits a bounded second invocation"
    );
    assert!(events.iter().any(|(_, value)| matches!(value, super::super::super::Value::Prefix(text, true, None) if text == "SELECT 1") || matches!(value, super::super::super::Value::Possible(values) if values.iter().any(|value| matches!(value, super::super::super::Value::Prefix(text, true, None) if text == "SELECT 1")))), "second invocation reads the prior unannotated slot");
    // Dynamic deletion changes semantic state even when stored slot values do not.
    let super::super::super::Value::Function(function, _, captured) =
        evaluator.scopes[root]["reader"].clone()
    else {
        panic!("saved reader callback");
    };
    let super::super::super::Value::Arguments(id) = evaluator.scopes[captured]["arguments"] else {
        panic!("saved lexical arguments identity");
    };
    let before = evaluator.callback_snapshot(captured, &function);
    evaluator.deleted_argument_slots.insert((id, None));
    assert!(
        before != evaluator.callback_snapshot(captured, &function),
        "mask-only changes are observed without scanning unrelated frames"
    );
}
