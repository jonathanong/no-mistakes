use super::super::{Evaluator, File};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn sequential_alternatives_discard_noncallback_frames() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/frame-count.mts",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    assert_eq!(root, 0);
    assert!(
        evaluator.next_builder >= 64,
        "both helper arms must execute"
    );
    assert_eq!(evaluator.scopes.len(), 1, "only the module frame survives");
}

#[test]
fn sloppy_named_arguments_function_uses_implicit_invocation_object() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/self-arguments.cjs",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    assert!(matches!(
        evaluator.scopes[root].get("result"),
        Some(super::super::Value::Prefix(text, true, None))
            if text == "/* implicit arguments wins */ SELECT 1"
    ));
}

#[test]
fn sequential_alternatives_discard_unreachable_deleted_slots() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/deleted-state.cjs",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    assert_eq!(root, 0);
    assert!(
        evaluator.next_builder >= 64,
        "both helper arms must execute"
    );
    assert_eq!(evaluator.scopes.len(), 1, "only the module frame survives");
    assert!(evaluator.deleted_argument_slots.is_empty());
    assert!(evaluator.mapped_arguments.is_empty());
}

#[test]
fn retained_mapping_preserves_deleted_slots_after_alias_rebinding() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/disconnected-metadata.cjs",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    assert!(matches!(
        evaluator.scopes[root].get("result"),
        Some(super::super::Value::Prefix(text, true, None))
            if text == "/* disconnected parameter */ SELECT 1"
    ));
}

#[test]
fn callback_frame_compaction_remaps_retained_parameter_metadata() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/mapped-callback-frame.cjs",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let Some(super::super::Value::Aggregate(values)) = evaluator.scopes[root].get("callback")
    else {
        panic!("expected alternative callback values");
    };
    // Conditional facts include the test value before the alternative result.
    let super::super::Value::Aggregate(arms) = &values[1] else {
        panic!("expected alternative arm values");
    };
    let super::super::Value::Function(_, _, captured) = &arms[0] else {
        panic!("expected retained callback");
    };
    assert_eq!(*captured, 1, "disposable helper frame is removed");
    assert_eq!(evaluator.scopes.len(), 2);
    assert!(evaluator.mapped_arguments.contains_key(captured));
}

#[test]
fn sequential_alternatives_discard_unreachable_builder_taint() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback/src/invalidated-state.mts",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    assert_eq!(root, 0);
    assert!(
        evaluator.next_builder >= 32,
        "all disposable builders must be evaluated"
    );
    assert_eq!(evaluator.scopes.len(), 1, "only the module frame survives");
    assert!(evaluator.invalidated_builders.is_empty());
    assert!(evaluator.deleted_argument_slots.is_empty());
    assert!(evaluator.mapped_arguments.is_empty());
}

#[test]
fn nested_scalar_joins_discard_sql_proof_but_keep_callback_references() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-live-binding/src/nested-scalars.cjs",
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
        definite_deleted_argument_slots: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    for (name, references) in [("scalar", false), ("callback", true)] {
        let original = vec![FxHashMap::from_iter([(
            "value".into(),
            evaluator.scopes[root][name].clone(),
        )])];
        let mut joined = original.clone();
        let current = vec![FxHashMap::from_iter([(
            "value".into(),
            super::super::Value::Unknown,
        )])];
        super::bindings::join(&mut joined, &current, &original);
        if references {
            assert!(matches!(
                joined[0]["value"],
                super::super::Value::Aggregate(_)
            ));
        } else {
            assert!(matches!(joined[0]["value"], super::super::Value::Unknown));
        }
    }
}

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
        definite_deleted_argument_slots: Default::default(),
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
        &Default::default(),
        &Default::default(),
        &mut changed,
        &Default::default(),
    );
    assert!(changed.contains(id));
    // Apply the detected alias mutation to the saved actual frame, retaining
    // unrelated prefixes and function bindings in the same scope.
    let mut scopes = vec![evaluator.scopes[root].clone()];
    super::values::apply_taint(&mut scopes, &changed);
    assert!(matches!(scopes[0]["annotated"], Value::Unknown));
    assert!(matches!(scopes[0]["bare"], Value::Prefix(_, _, Some(_))));
}
