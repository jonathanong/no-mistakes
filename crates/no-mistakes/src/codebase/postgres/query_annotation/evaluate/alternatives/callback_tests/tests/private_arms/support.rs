use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::query_annotation::evaluate::{Environment, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

fn resolve(_: &str, _: &Path) -> Option<PathBuf> {
    None
}
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn evaluator<'a>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
) -> Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>> {
    Evaluator {
        parallel_arms: None,
        files,
        resolve,
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
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    }
}

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-arm-private-callbacks")
        .canonicalize()
        .unwrap()
}

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn functions(
    value: &Value,
) -> Vec<(u32, Environment)> {
    let mut result = Vec::new();
    match value {
        Value::Function(function, _, env) => result.push((function.start, *env)),
        Value::Evaluated(value, _) => result.extend(functions(value)),
        Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
            for value in values {
                result.extend(functions(value));
            }
        }
        _ => panic!("saved returned callbacks must remain functions"),
    }
    result
}

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn files<'a>(
    path: &Path,
    facts: &'a query_annotation::QueryAnnotationFileFacts,
    embedded: &crate::codebase::postgres::EmbeddedSqlFileFacts,
) -> FxHashMap<PathBuf, File<'a>> {
    FxHashMap::from_iter([(
        path.to_path_buf(),
        File {
            facts,
            module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )])
}

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn fixture(
    name: &str,
) -> (std::path::PathBuf, String, CheckFileFacts) {
    let path = root().join(format!("src/{name}.mts"));
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let file = crate::ast::with_program(&path, &source, |program, _| {
        let facts = query_annotation::collect(program, &source, &options);
        CheckFileFacts {
            ts: std::sync::Arc::new(collect_file_facts_from_program(
                &path,
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
            )),
            query_annotation: vec![(options.clone(), facts)],
            embedded_sql: vec![(
                options.clone(),
                extract_embedded_sql_from_program(&path, program, &source, &options),
            )],
            source: Some(source.clone().into()),
            ..Default::default()
        }
    })
    .unwrap();
    (path, source, file)
}

#[cfg(feature = "test-instrumentation")]
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn assert_report() {
    let off = crate::benchmark_support::check_json(&root()).unwrap();
    let (on, _) = crate::benchmark_support::check_json_observed(&root(), true).unwrap();
    assert_eq!(off, on);
    let report: serde_json::Value = serde_json::from_str(&off).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2);
    assert!(report["warnings"].as_array().unwrap().is_empty());
    assert!(report["advisories"].as_array().unwrap().is_empty());
    for (name, line) in [("src/no-deletion.mts", 4), ("src/deletion.mts", 4)] {
        assert!(findings.iter().any(|finding| finding["file"] == name
            && finding["line"] == line
            && finding["rule"] == "postgres-require-query-annotation"));
    }
}
