use super::super::{Evaluator, File};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn precise_records_with_reference_bearing_arms_create_no_prefix_observer_work() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/observer-batching/precise-root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@data-stores/psql", &[]);
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let Some(query_annotation::Expr::Sequence(parts)) = facts.globals.get("joined") else {
        panic!("saved conditional evaluates its condition before the arms");
    };
    assert!(matches!(parts.last(),
        Some(query_annotation::Expr::Alternatives(arms)) if arms.len() == 2));
    // Branch-only input keeps references live after evaluation. Opaque executor
    // calls in the public report fixture conservatively invalidate them later.
    facts.precise_records = true;
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        parallel_arms: None,
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
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
    let observer = InvocationObserver::new(true);
    let root = with_observer(Some(observer.clone()), || {
        evaluator.module_environment(&path)
    });
    assert!(evaluator.scopes[root].contains_reference());
    assert!(evaluator.scopes[root].contains_key("joined"));
    assert!(evaluator.scopes[root]["joined"].contains_reference());
    let work = observer.snapshot().work;
    assert!(!work.contains_key("postgres.annotation.prefix_compare_arm"));
    assert!(!work.contains_key("postgres.annotation.prefix_observer_batches"));
    assert!(!work.contains_key("postgres.annotation.prefix_walks"));
}
