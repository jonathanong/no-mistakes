use super::super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn slot_owner_lookup_preserves_duplicates_arity_and_canonical_inherited_origins() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-argument-slot-write/src/owners.cjs",
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
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    };
    evaluator.module_environment(&path);
    let owner = |label: &str| {
        evaluator
            .mapped_argument_owners
            .iter()
            .find_map(|(id, frame)| {
                matches!(evaluator.scopes[*frame].get("caseName"), Some(Value::Prefix(text, _, _)) if text == label)
                    .then_some((*id, *frame))
            })
            .unwrap()
    };
    let (direct, direct_frame) = owner("direct");
    let (duplicate, duplicate_frame) = owner("duplicate");
    let (absent, _) = owner("absent");
    let (empty, _) = owner("empty");
    let (outer, outer_frame) = owner("outer");
    let (inner, inner_frame) = owner("inner");
    assert!(evaluator.mapped_arguments[&inner_frame]
        .iter()
        .any(|(id, names)| *id == outer && names == &[String::new()]));
    assert_eq!(evaluator.mapped_slot_target(u64::MAX, 0), None);
    assert_eq!(evaluator.mapped_slot_target(direct, 3), None);
    assert_eq!(evaluator.mapped_slot_target(duplicate, 0), None);
    assert_eq!(evaluator.mapped_slot_target(absent, 0), None);
    assert_eq!(evaluator.mapped_slot_target(empty, 0), None);
    evaluator.rebuild_mapped_argument_owners();
    for (id, frame, name) in [
        (direct, direct_frame, "directParam"),
        (duplicate, duplicate_frame, "repeatedParam"),
        (outer, outer_frame, "outerParam"),
        (inner, inner_frame, "innerParam"),
    ] {
        let index = usize::from(id == duplicate);
        assert_eq!(
            evaluator.mapped_slot_target(id, index),
            Some((frame, name.into()))
        );
    }
    assert_eq!(evaluator.mapped_slot_target(absent, 0), None);
    assert_eq!(evaluator.mapped_slot_target(empty, 0), None);
    evaluator.prune_snapshot_state();
    assert!(evaluator.mapped_argument_owners.is_empty());
}
