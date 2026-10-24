use super::super::{hooks, Value};
use super::legacy;
use crate::codebase::postgres::query_annotation::{
    self,
    evaluate::{alternatives::value_equality_test_support, Evaluator, File, Scope},
    modules::Module,
};
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::{path::PathBuf, sync::Arc};
fn scope_state(scopes: &[Scope]) -> Vec<std::collections::BTreeMap<String, String>> {
    scopes
        .iter()
        .map(|scope| {
            scope
                .iter()
                .map(|(name, value)| (name.clone(), legacy::describe(value)))
                .collect()
        })
        .collect()
}
type Resolve = fn(&str, &std::path::Path) -> Option<PathBuf>;
fn with_fixture<T>(
    name: &str,
    inspect: impl FnOnce(&mut Evaluator<'_, Resolve>, &std::path::Path, &str) -> T,
) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/performance/annotation-shared-record-dag/{name}/root.mts"
    ));
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
        resolve: (|_: &str, _: &std::path::Path| -> Option<PathBuf> { None }) as Resolve,
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
    inspect(&mut evaluator, &path, &source)
}
fn run() -> (Vec<String>, usize) {
    with_fixture("value-equality", |evaluator, path, source| {
        for (sql, annotation) in [("/* clean */ SELECT 1", false), ("SELECT 2", true)] {
            let call = format!("read(\"{sql}\")");
            let start = source.find(&call).unwrap() as u32;
            let events =
                &evaluator.events[&(path.to_path_buf(), (start, start + call.len() as u32))];
            assert!(!events.is_empty());
            assert!(events.iter().all(|(_, value)| matches!(value, Value::Prefix(text, _, _)
            if text == sql && crate::codebase::postgres::sql_requires_query_annotation(text) == annotation)));
        }
        let env = evaluator.module_environment(path);
        assert!(matches!(
            evaluator.scopes[env].get("shared0"),
            Some(Value::Record(_, _))
        ));
        let current = evaluator.scopes.clone();
        let mut initial = current.clone();
        assert!(initial[env].remove("shared0").is_some());
        let current_before = scope_state(&current);
        let initial_before = scope_state(&initial);
        let evaluator_before = scope_state(&evaluator.scopes);
        let mut joined = current.clone();
        assert!(joined[env].same_bindings(&current[env]));
        assert!(!joined[env].same_bindings(&initial[env]));
        // A binding absent initially must stay unproven even when both arms share
        // their entire fixture-derived scope. The three-scope guard cannot skip it.
        value_equality_test_support::join(&mut joined, &current, &initial);
        assert!(matches!(joined[env].get("shared0"), Some(Value::Unknown)));
        assert_eq!(scope_state(&current), current_before);
        assert_eq!(scope_state(&initial), initial_before);
        assert_eq!(scope_state(&evaluator.scopes), evaluator_before);
        assert!(current[env].same_bindings(&evaluator.scopes[env]));
        assert!(!joined[env].same_bindings(&current[env]));
        let mut expected_join = current_before;
        expected_join[env].insert("shared0".into(), legacy::describe(&Value::Unknown));
        assert_eq!(scope_state(&joined), expected_join);
        let mut output = vec![format!(
            "joined={:?};current={:?};initial={:?};evaluator={:?}",
            scope_state(&joined),
            scope_state(&current),
            scope_state(&initial),
            scope_state(&evaluator.scopes)
        )];
        output.extend(evaluator.events.iter().flat_map(|(key, events)| {
            events.iter().map(move |(generic, value)| {
                format!("{key:?}:{generic}:{}", legacy::describe(value))
            })
        }));
        (output, evaluator.scopes.len())
    })
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
                "real fixture-derived join must exercise shared Value comparisons"
            );
            eprintln!("value equality observed={observed}: fallback entries {old_work} -> {work}");
        });
    }
}

#[test]
fn saved_distinct_dag_fixture_preserves_events_and_reduces_real_comparisons() {
    let run = || {
        with_fixture("value-equality-distinct", |evaluator, _, _| {
            let mut events = evaluator
                .events
                .values()
                .flat_map(|events| events.iter().map(|(_, value)| legacy::describe(value)))
                .collect::<Vec<_>>();
            events.sort();
            assert_eq!(events, vec!["Prefix(\"/* clean */ SELECT 1\", true, None)"]);
            events
        })
    };
    let (expected, before) = hooks::measure(true, run);
    let (actual, after) = hooks::measure(false, run);
    assert_eq!(actual, expected);
    eprintln!("saved distinct DAG evaluator fallbacks: {before} -> {after}");
    assert!(
        before > after * 10,
        "real branch joins must reuse completed pairs"
    );
}
