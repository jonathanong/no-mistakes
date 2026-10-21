use super::super::{hooks, Value};
use super::legacy;
use crate::codebase::postgres::query_annotation::{
    self,
    evaluate::{Evaluator, File},
    modules::Module,
};
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::{path::PathBuf, sync::Arc};
fn run() -> (Vec<String>, usize) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/value-equality/root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
    let (facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(TsFileFacts::default().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let mut evaluator = Evaluator {
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
    evaluator.run(&path);
    for (sql, annotation) in [("/* clean */ SELECT 1", false), ("SELECT 2", true)] {
        let call = format!("read(\"{sql}\")");
        let start = source.find(&call).unwrap() as u32;
        let events = &evaluator.events[&(path.clone(), (start, start + call.len() as u32))];
        assert!(!events.is_empty());
        assert!(events.iter().all(|(_, value)| matches!(value, Value::Prefix(text, _, _)
            if text == sql && crate::codebase::postgres::sql_requires_query_annotation(text) == annotation)));
    }
    (
        evaluator
            .events
            .iter()
            .flat_map(|(key, events)| {
                events.iter().map(move |(generic, value)| {
                    format!("{key:?}:{generic}:{}", legacy::describe(value))
                })
            })
            .collect(),
        evaluator.scopes.len(),
    )
}
#[test]
fn saved_evaluator_fixture_matches_legacy_with_real_identity_work() {
    let mut findings = None;
    for observed in [false, true] {
        let observer = observed.then(|| crate::diagnostics::InvocationObserver::new(true));
        crate::diagnostics::with_observer(observer, || {
            let (optimized, work) = hooks::measure(false, run);
            let (legacy, old_work) = hooks::measure(true, run);
            assert_eq!(optimized, legacy);
            if let Some(previous) = &findings {
                assert_eq!(&optimized, previous);
            }
            findings = Some(optimized);
            assert!(
                old_work > work,
                "saved branches must exercise shared Value comparisons"
            );
            eprintln!("value equality observed={observed}: fallback entries {old_work} -> {work}");
        });
    }
}
