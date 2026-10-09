use super::super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn earlier_arguments_follow_later_builder_mutations_through_result_wrappers() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-call-argument-order/src/query.mts",
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
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        deleted_argument_slots: Default::default(),
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        argument_objects: Default::default(),
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let values = &evaluator.scopes[root];
    for name in ["directResult", "wrappedResult"] {
        assert!(
            matches!(values.get(name), Some(Value::Prefix(text, true, _)) if text == "SELECT 1"),
            "{name}"
        );
    }
    assert!(
        matches!(values.get("promiseResult"), Some(Value::Promise(inner)) if matches!(inner.as_ref(), Value::Prefix(text, true, _) if text == "SELECT 1"))
    );
    assert!(
        matches!(values.get("nestedResult"), Some(Value::Aggregate(parts)) if matches!(parts.first(), Some(Value::Prefix(text, true, _)) if text == "SELECT 1"))
    );
    assert!(
        matches!(values.get("annotatedResult"), Some(Value::Prefix(text, true, _)) if text == "/* later annotation */ SELECT 1")
    );
    assert!(
        matches!(values.get("immutableResult"), Some(Value::Prefix(text, true, None)) if text == "/* literal copy */ SELECT 1")
    );
    assert!(matches!(values.get("escapedResult"), Some(Value::Unknown)));
    assert!(
        matches!(values.get("nestedEscapedResult"), Some(Value::Aggregate(parts)) if matches!(parts.first(), Some(Value::Unknown)))
    );
}
