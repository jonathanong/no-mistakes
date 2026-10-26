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

fn run() -> (Vec<String>, Vec<Value>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/performance/annotation-shared-record-dag/value-equality-discriminants/root.mts");
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
    let env = evaluator.module_environment(&path);
    let pair = ["left", "right"].map(|name| evaluator.scopes[env][name].clone());
    let [Value::Prefix(a, true, Some(id)), Value::Prefix(b, true, Some(other))] = &pair else {
        panic!("fixture must retain two complete concrete builders");
    };
    assert_eq!(a, b);
    assert_ne!(id, other);
    let joined = super::super::super::super::concat::joined(pair.to_vec());
    assert!(matches!(&joined, Value::Joined(values) if values.len() == 2));
    let mut state = evaluator
        .scopes
        .iter()
        .enumerate()
        .flat_map(|(env, scope)| {
            scope
                .iter()
                .map(move |(name, value)| format!("{env}:{name}:{}", legacy::describe(value)))
        })
        .collect::<Vec<_>>();
    fn sql_events(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Prefix(text, true, _) => out.push(text.clone()),
            Value::Aggregate(values) | Value::Joined(values) => {
                for value in values {
                    sql_events(value, out);
                }
            }
            _ => panic!("query event must contain complete concrete SQL"),
        }
    }
    let mut queries = Vec::new();
    for (_, value) in evaluator.events.values().flatten() {
        sql_events(value, &mut queries);
    }
    for (sql, annotated) in [(a.as_str(), false), ("SELECT 2", true)] {
        assert!(
            queries.iter().any(|query| query == sql),
            "missing query: {sql}"
        );
        assert_eq!(
            crate::codebase::postgres::sql_requires_query_annotation(sql),
            annotated
        );
    }
    state.extend(evaluator.events.iter().flat_map(|(key, values)| {
        values
            .iter()
            .map(move |(generic, value)| format!("{key:?}:{generic}:{}", legacy::describe(value)))
    }));
    state.push(legacy::describe(&joined));
    state.sort();
    (state, pair.to_vec())
}

#[test]
fn real_builder_candidates_reject_distinct_ids_before_sql_comparison() {
    for observed in [false, true] {
        let observer = observed.then(|| crate::diagnostics::InvocationObserver::new(true));
        crate::diagnostics::with_observer(observer, || {
            let ((state, pair), work) = hooks::measure_sql(false, run);
            let ((old_state, _), old_work) = hooks::measure_sql(true, run);
            assert_eq!(state, old_state);
            assert!(old_work > work);
            let candidates = || super::super::super::super::concat::joined(pair.clone());
            let (new, new_work) = hooks::measure_sql(false, candidates);
            let (old, old_work) = hooks::measure_sql(true, candidates);
            assert_eq!(legacy::describe(&new), legacy::describe(&old));
            assert_eq!(new_work, 0);
            assert!(old_work > 0);
        });
    }
}

#[test]
fn equal_ids_sql_and_completeness_remain_semantic() {
    let prefix = |text: &str, complete, id| Value::Prefix(text.into(), complete, id);
    let cases = [
        prefix("SELECT 1", true, Some(1)),
        prefix("SELECT 1", true, Some(2)),
        prefix("SELECT 2", true, Some(1)),
        prefix("SELECT 1", false, Some(1)),
        prefix("SELECT 1", true, None),
    ];
    for a in &cases {
        for b in &cases {
            assert_eq!(a == b, legacy::equal(a, b));
        }
    }
    let independent = prefix("SELECT 1", true, Some(1));
    assert_eq!(
        hooks::measure_sql(false, || cases[0] == independent),
        (true, 1)
    );
    assert_eq!(
        hooks::measure_sql(false, || cases[0] == cases[1]),
        (false, 0)
    );
    assert_eq!(
        hooks::measure_sql(false, || cases[0] == cases[2]),
        (false, 1)
    );
    assert_eq!(
        hooks::measure_sql(false, || cases[0] == cases[3]),
        (false, 0)
    );
    assert_eq!(
        hooks::measure_sql(false, || cases[0] == cases[4]),
        (false, 0)
    );
}
