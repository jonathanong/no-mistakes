use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn evaluated(name: &str) -> (FxHashMap<String, Value>, usize, usize) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-mapped-context/{name}",
    ));
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
    (
        evaluator.scopes[root].clone(),
        evaluator.argument_objects.len(),
        evaluator.argument_objects.values().map(Vec::len).sum(),
    )
}

fn outputs(name: &str) -> FxHashMap<String, Value> {
    evaluated(name).0
}

#[test]
fn sloppy_rebinding_updates_the_mapped_slot() {
    assert!(matches!(
        outputs("rebinding.cjs").get("result"),
        Some(Value::Prefix(text, true, None)) if text == "SELECT 1"
    ));
}

#[test]
fn strict_rebinding_and_arrow_shadowing_preserve_original_arguments() {
    for (name, expected) in [
        ("strict-rebinding.cjs", "/* initial annotation */ SELECT 1"),
        ("arrow-shadow.cjs", "/* lexical arguments */ SELECT 1"),
    ] {
        assert!(
            matches!(outputs(name).get("result"), Some(Value::Prefix(text, true, None)) if text == expected)
        );
    }
}

#[test]
fn only_the_last_duplicate_parameter_maps() {
    let values = outputs("duplicate-rebinding.cjs");
    assert!(
        matches!(values.get("original"), Some(Value::Prefix(text, true, None)) if text == "/* retained first */ SELECT 1")
    );
    assert!(
        matches!(values.get("result"), Some(Value::Prefix(text, true, None)) if text == "replacement")
    );
}

#[test]
fn known_slot_write_updates_parameter_and_deletion_disconnects_it() {
    let values = outputs("reverse-rebinding.cjs");
    assert!(
        matches!(values.get("result"), Some(Value::Prefix(text, true, None)) if text == "SELECT 1")
    );
    assert!(matches!(
        values.get("inheritedResult"),
        Some(Value::Unknown)
    ));
    assert!(
        matches!(values.get("preserved"), Some(Value::Prefix(text, true, None)) if text == "/* disconnected parameter */ SELECT 1")
    );
}

#[test]
fn absent_last_parameter_slot_has_no_mapping() {
    let values = outputs("reverse-rebinding.cjs");
    for (name, expected) in [
        ("absentResult", "/* locally assigned */ SELECT 1"),
        ("duplicateAbsentResult", "/* duplicate local */ SELECT 1"),
    ] {
        assert!(
            matches!(values.get(name), Some(Value::Prefix(text, true, None)) if text == expected)
        );
    }
}

#[test]
fn container_aliases_and_local_shadows_keep_mapping_boundaries() {
    let values = outputs("aliases.cjs");
    assert!(
        matches!(values.get("result"), Some(Value::Prefix(text, true, None)) if text == "SELECT 1")
    );
    assert!(matches!(values.get("unknownResult"), Some(Value::Unknown)));
    assert!(matches!(values.get("deletedResult"), Some(Value::Unknown)));
    assert!(matches!(values.get("reservedResult"), Some(Value::Unknown)));
    assert!(
        matches!(values.get("shadowResult"), Some(Value::Prefix(text, true, None)) if text == "/* preserved */ SELECT 1")
    );
}

#[test]
fn mapped_rebinding_preserves_callback_capture_effects() {
    assert!(matches!(
        outputs("callback-rebinding.cjs").get("result"),
        Some(Value::Unknown)
    ));
}

#[test]
fn repeated_argument_aliases_are_identity_only() {
    let (values, objects, slots) = evaluated("cyclic-aliases.cjs");
    assert!(matches!(values.get("result"), Some(Value::Arguments(_))));
    assert_eq!((objects, slots), (1, 2));
}

#[test]
fn escaped_arguments_keep_possible_rebound_callbacks() {
    assert!(matches!(
        outputs("escaped-callback.cjs").get("result"),
        Some(Value::Unknown)
    ));
}

#[test]
fn deletion_cannot_restore_parameters_after_opaque_argument_escape() {
    let values = outputs("escaped-then-deleted.cjs");
    for name in [
        "result",
        "capturedResult",
        "callbackResult",
        "duplicateFirstResult",
        "duplicateLastResult",
    ] {
        assert!(matches!(values.get(name), Some(Value::Unknown)), "{name}");
    }
    assert!(
        matches!(values.get("absentResult"), Some(Value::Prefix(text, true, None)) if text == "/* unmapped local */ SELECT 1")
    );
}
