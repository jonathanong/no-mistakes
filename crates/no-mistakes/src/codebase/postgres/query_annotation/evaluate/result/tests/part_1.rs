use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn outputs(scenario: &str) -> Vec<(String, Value)> {
    outputs_file(scenario, "query.mts")
}

fn outputs_file(scenario: &str, name: &str) -> Vec<(String, Value)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/{scenario}/src/{name}",
    ));
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
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        transactions: Default::default(),
        scopes: Default::default(),
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
    evaluator.module_environment(&path);
    assert!(evaluator.active_callback_functions.is_none());
    embedded
        .calls
        .iter()
        .zip(&embedded.call_spans)
        .filter_map(|(call, start)| {
            let sql = format!(
                "{} {}",
                call.sql_text.as_deref().unwrap_or(""),
                source[start.0 as usize..].lines().next().unwrap()
            );
            let values = evaluator.events.get(&(path.clone(), *start))?;
            Some(
                values
                    .iter()
                    .map(move |(_, value)| (sql.clone(), value.clone())),
            )
        })
        .flatten()
        .collect()
}

fn event<'a>(values: &'a [(String, Value)], marker: &str) -> &'a Value {
    &values
        .iter()
        .find(|(sql, _)| sql.contains(marker))
        .unwrap_or_else(|| panic!("missing executor event for {marker}"))
        .1
}

#[test]
fn discarded_values_do_not_escape_callbacks_but_operand_effects_still_run() {
    let values = outputs("helper-tracing-discarded-values");
    for marker in [
        "initializer sequence callback",
        "initializer void callback",
        "sequence callback",
        "void callback",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, _) if text.contains("SELECT 1")),
            "{marker}"
        );
    }
    assert!(matches!(
        event(&values, "sequence side effect"),
        Value::Unknown
    ));
    assert!(matches!(event(&values, "void side effect"), Value::Unknown));
    assert!(matches!(
        event(&values, "surviving callback"),
        Value::Unknown
    ));
    assert!(matches!(
        event(&values, "unknown final argument"),
        Value::Unknown
    ));
}

#[test]
fn primitive_argument_length_proof_requires_an_unescaped_canonical_container() {
    let values = outputs("helper-tracing-argument-length");
    for marker in [
        "standalone length",
        "computed length",
        "sequence length",
        "primitive deletion",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, _) if text.contains("SELECT 1")),
            "{marker}"
        );
    }
    for marker in [
        "shadowed-callback",
        "escaped length",
        "mixed length",
        "deletion key effect",
    ] {
        assert!(matches!(event(&values, marker), Value::Unknown), "{marker}");
    }
}

#[test]
fn opaque_consumers_follow_returned_callbacks_with_bounded_recursion() {
    let values = outputs("helper-tracing-returned-callbacks");
    for (marker, text) in [
        ("nested-callback", "SELECT 1"),
        ("aggregate-callback", "SELECT 2"),
        (
            "annotated-returned-callback",
            "/* returned annotation */ SELECT 1",
        ),
        ("returned-callback-mutates-local-builder", "SELECT 3"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if sql == text),
            "{marker}"
        );
    }
    assert!(matches!(
        event(&values, "returned-callback-capture"),
        Value::Unknown
    ));
    assert_eq!(values.len(), 5);
}

