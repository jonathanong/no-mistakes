use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn imported_callback_state_is_observed_without_another_fact_pass() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-imported-callback-revisit/src");
    let path = base.join("query.mts");
    let helper = base.join("helper.mts");
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let prepared = [&path, &helper]
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).unwrap();
            let values = crate::ast::with_program(path, &source, |program, _| {
                (
                    query_annotation::collect(program, &source, &options),
                    extract_embedded_sql_from_program(path, program, &source, &options),
                    collect_file_facts_from_program(
                        path,
                        TsFactPlan {
                            imports: true,
                            module_bindings: true,
                            ..Default::default()
                        },
                        &TsFactContext::default(),
                        &source,
                        program,
                        None,
                        None,
                    ),
                )
            })
            .unwrap();
            (path.clone(), source, values)
        })
        .collect::<Vec<_>>();
    let files = prepared
        .iter()
        .map(|(path, _, (facts, embedded, ts))| {
            (
                path.clone(),
                File {
                    facts,
                    ts,
                    executors: embedded.call_starts.iter().copied().collect(),
                    imports: ts
                        .imported_bindings
                        .iter()
                        .enumerate()
                        .map(|(index, binding)| (binding.local.clone(), index))
                        .collect(),
                    exports: ts
                        .exported_bindings
                        .iter()
                        .enumerate()
                        .map(|(index, binding)| (binding.exported.clone(), index))
                        .collect(),
                },
            )
        })
        .collect::<FxHashMap<_, _>>();
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |specifier: &str, _: &std::path::Path| -> Option<PathBuf> {
            (specifier == "./helper.mjs").then(|| helper.clone())
        },
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
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.module_environment(&path);
    let source = &prepared
        .iter()
        .find(|(path, _, _)| path == &helper)
        .unwrap()
        .1;
    let start = source.find("write(saved)").unwrap() as u32;
    let events = evaluator
        .events
        .get(&(helper, start))
        .expect("saved imported executor");
    assert!(
        events.len() >= 2,
        "imported captured state changes trigger a bounded revisit"
    );
    fn unsafe_candidate(value: &Value) -> bool {
        match value {
            Value::Prefix(text, true, None) => text == "SELECT 1",
            Value::Possible(values) => values.iter().any(unsafe_candidate),
            _ => false,
        }
    }
    assert!(events.iter().any(|(_, value)| unsafe_candidate(value)));
}
