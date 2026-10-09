use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn outputs() -> FxHashMap<String, Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-named-delete/src/query.mts",
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
    evaluator.scopes[root].clone()
}

fn is_prefix(values: &FxHashMap<String, Value>, name: &str, expected: &str) -> bool {
    matches!(values.get(name), Some(Value::Prefix(text, true, _)) if text == expected)
}

#[test]
fn named_and_statically_known_non_index_deletes_preserve_argument_slots() {
    let values = outputs();
    for (name, expected) in [
        ("namedResult", "/* named property */ SELECT 1"),
        (
            "computedNamedResult",
            "/* computed named property */ SELECT 1",
        ),
        ("nonIndexResult", "/* known non-index key */ SELECT 1"),
        ("nonCanonicalResult", "/* non-canonical index */ SELECT 1"),
        ("fractionalResult", "/* fractional property */ SELECT 1"),
        ("preservedIndexResult", "/* retained slot */ SELECT 1"),
        (
            "conditionalDeleteResult",
            "/* conditional slot deletion */ SELECT 1",
        ),
    ] {
        assert!(is_prefix(&values, name, expected), "{name}");
    }
}

#[test]
fn numeric_indices_and_dynamic_keys_keep_their_invalidation_behavior() {
    let values = outputs();
    let Some(Value::Aggregate(dynamic)) = values.get("dynamicResult") else {
        panic!("dynamic deletion preserves a possible callback slot");
    };
    assert!(matches!(dynamic.first(), Some(Value::Unknown)));
    assert!(matches!(
        dynamic.get(1),
        Some(Value::Prefix(text, true, _)) if text == "/* dynamic key */ SELECT 1"
    ));
    for name in [
        "deletedIndexResult",
        "deletedNumericIndexResult",
        "dynamicEffectResult",
        "conditionalOpaqueArmResult",
        "conditionalOpaqueTestResult",
    ] {
        assert!(matches!(values.get(name), Some(Value::Unknown)), "{name}");
    }
}

#[test]
fn handled_effect_checks_fail_closed_on_mismatched_evaluated_shapes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-named-delete/src/query.mts",
    );
    let facts = query_annotation::QueryAnnotationFileFacts::default();
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
    let evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
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
    let children =
        query_annotation::Expr::Children(vec![query_annotation::Expr::Text("SELECT 1".into())]);
    let slot_write = query_annotation::Expr::SlotWrite {
        receiver: Box::new(query_annotation::Expr::Name("arguments".into())),
        index: 0,
        value: Box::new(query_annotation::Expr::Text("replacement".into())),
    };
    let delete = query_annotation::Expr::Delete(
        vec![query_annotation::Expr::Name("arguments".into())],
        query_annotation::DeleteKey::Index(0),
    );

    assert!(!evaluator.handled_deletion_effect(&children, &Value::Unknown, &path, 0));
    assert!(!evaluator.handled_deletion_effect(&slot_write, &Value::Unknown, &path, 0));
    assert!(evaluator.handled_deletion_effect(
        &slot_write,
        &Value::Evaluated(Box::new(Value::Unknown), true),
        &path,
        0,
    ));
    assert!(!evaluator.handled_deletion_effect(
        &slot_write,
        &Value::Evaluated(Box::new(Value::Unknown), false),
        &path,
        0,
    ));
    assert!(!evaluator.handled_deletion_effect(&delete, &Value::Unknown, &path, 0));
}
