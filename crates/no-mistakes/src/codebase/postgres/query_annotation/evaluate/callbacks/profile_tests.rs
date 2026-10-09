use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn captured_snapshot_tracks_only_live_semantic_state_and_bounded_revisits() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-profile/state.cjs",
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
    let Value::Function(function, _, captured) = evaluator.scopes[root]["observer"].clone() else {
        panic!("saved observer");
    };
    let Value::Arguments(id) = evaluator.scopes[captured]["arguments"] else {
        panic!("saved invocation identity");
    };
    for (name, mutating) in [
        ("mutationHoisted", false),
        ("mutationAppend", true),
        ("mutationTemplate", true),
        ("mutationSequence", true),
    ] {
        let Value::Function(summary, _, env) = &evaluator.scopes[root][name] else {
            panic!("saved mutation summary");
        };
        assert_eq!(
            evaluator.callback_may_mutate(summary, &path, *env),
            mutating,
            "{name}"
        );
    }
    let mut memo = super::memo::Memo::default();
    let memo_initial =
        evaluator.memoized_callback_snapshot(&path, captured, &function, 8, &mut memo);
    assert!(
        memo_initial
            == evaluator.memoized_callback_snapshot(&path, captured, &function, 8, &mut memo)
    );
    assert_eq!(
        memo.constructions, 1,
        "unchanged fresh bindings reuse their snapshot"
    );
    let initial = evaluator.callback_snapshot(captured, &function);
    assert!(evaluator.fresh_mapped_parameters[&captured].contains("parameter"));
    evaluator.invalidate_mapped_freshness(&crate::fx::FxHashSet::from_iter([id]));
    assert!(!evaluator.fresh_mapped_parameters.contains_key(&captured));
    let changed_proof =
        evaluator.memoized_callback_snapshot(&path, captured, &function, 8, &mut memo);
    assert!(changed_proof != memo_initial);
    assert_eq!(
        memo.constructions, 2,
        "proof revocation invalidates the same memo key"
    );
    let previous = evaluator.scopes[captured]
        .insert("parameter".into(), Value::Unknown)
        .unwrap();
    let changed_binding =
        evaluator.memoized_callback_snapshot(&path, captured, &function, 8, &mut memo);
    assert!(changed_binding != changed_proof);
    assert_eq!(
        memo.constructions, 3,
        "changed actual root binding invalidates cached snapshot"
    );
    evaluator.scopes[captured].insert("parameter".into(), previous);

    evaluator.mark_mapped_fresh(captured, "parameter");
    evaluator.invalidated_builders.insert(id);
    assert!(initial != evaluator.callback_snapshot(captured, &function));
    let escaped = evaluator.callback_snapshot(captured, &function);
    evaluator
        .fresh_mapped_parameters
        .get_mut(&captured)
        .unwrap()
        .remove("parameter");
    assert!(escaped != evaluator.callback_snapshot(captured, &function));
    let masks = evaluator.callback_snapshot(captured, &function);
    evaluator.deleted_argument_slots.insert((id, Some(0)));
    evaluator.definite_deleted_argument_slots.insert((id, 0));
    evaluator.disconnected_argument_slots.insert((id, 0));
    assert!(masks != evaluator.callback_snapshot(captured, &function));
    let Value::Prefix(_, _, Some(builder_id)) = evaluator.scopes[root]["builder"] else {
        panic!("saved builder");
    };
    assert!(evaluator.builder_updates.contains_key(&builder_id));
    let before = evaluator.callback_snapshot(captured, &function);
    evaluator.invalidated_builders.insert(builder_id);
    assert!(before != evaluator.callback_snapshot(captured, &function));
    // Preserve real reference identities while probing private wrapper traversal.
    let builder = evaluator.scopes[root]["builder"].clone();
    evaluator.scopes[captured].insert(
        "parameter".into(),
        Value::Possible(vec![Value::Evaluated(Box::new(builder), true)]),
    );
    let wrappers = evaluator.callback_snapshot(captured, &function);
    evaluator.builder_updates.remove(&builder_id);
    assert!(wrappers != evaluator.callback_snapshot(captured, &function));
    let first = evaluator.scopes[root]["first"].clone();
    let second = evaluator.scopes[root]["second"].clone();
    evaluator.opaque_callbacks(&[first, second], 8);
    let start = source.find("database.query(saved)").unwrap() as u32;
    let events = &evaluator.events[&(path, start)];
    assert_eq!(
        events.len(),
        4,
        "two independently changed callbacks each execute twice"
    );
}
