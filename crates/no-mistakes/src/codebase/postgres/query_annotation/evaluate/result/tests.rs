use super::super::{Evaluator, File, Value};
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
        .zip(&embedded.call_starts)
        .filter_map(|(call, start)| {
            let sql = format!(
                "{} {}",
                call.sql_text.as_deref().unwrap_or(""),
                source[*start as usize..].lines().next().unwrap()
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

#[test]
fn opaque_consumers_revisit_mutated_argument_slots_without_repeating_installers() {
    let values = outputs("helper-tracing-callback-installers");
    for (marker, text) in [
        ("object-first", "SELECT 1"),
        ("installer-first", "SELECT 2"),
        ("cyclic-container", "SELECT 3"),
        ("duplicate-installed-callback", "SELECT 4"),
        ("duplicate-installer-body", "SELECT 5"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if sql == text),
            "{marker}"
        );
    }
    assert_eq!(
        values
            .iter()
            .filter(|(sql, _)| sql.contains("separate-consumers"))
            .count(),
        2
    );
    assert_eq!(values.len(), 7);
}

#[test]
fn numeric_unary_operators_preserve_coercion_effects() {
    let values = outputs("helper-tracing-discarded-values");
    for marker in [
        "unary plus coercion",
        "unary negation coercion",
        "bitwise not coercion",
    ] {
        assert!(matches!(event(&values, marker), Value::Unknown), "{marker}");
    }
    for marker in [
        "typeof is noncoercive",
        "logical not is noncoercive",
        "void callback",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, _) if text.contains("SELECT 1")),
            "{marker}"
        );
    }
}

#[test]
fn unary_zero_indices_match_canonical_arguments_slot_zero_for_reads_and_writes() {
    let values = outputs("helper-tracing-unary-argument-index");
    for (marker, sql) in [
        ("positive-zero-read", "SELECT 1"),
        ("negative-zero-read", "SELECT 2"),
        ("positive-zero-write", "SELECT 3"),
        ("negative-zero-write", "SELECT 4"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(value, true, None) if value == sql),
            "{marker}"
        );
    }
}

#[test]
fn effect_references_preserve_callbacks_mutations_and_single_operand_evaluation() {
    let values = outputs("reference-fragment-effects");
    for (marker, text) in [
        ("comparison operand effect", "SELECT 1"),
        ("first object callback", "SELECT 2"),
        ("second object callback", "SELECT 3"),
        ("scalar comparison bind", "sql_placeholder_1"),
        ("mutation comparison bind", "sql_placeholder_1"),
        ("interpolation mutation effect", "SELECT 5"),
        ("dynamic-index operand effect", "SELECT 6"),
        ("computed-object key effect", "SELECT 7"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if sql == text),
            "{marker}"
        );
        assert_eq!(
            values
                .iter()
                .filter(|(sql, _)| sql.contains(marker))
                .count(),
            1
        );
    }
    assert!(matches!(
        event(&values, "object builder escape"),
        Value::Unknown
    ));
    for marker in [
        "dynamic-index fragment candidate",
        "object-property fragment candidate",
    ] {
        assert!(matches!(event(&values, marker), Value::Prefix(text, false, _) if text.is_empty()));
        assert_eq!(
            values
                .iter()
                .filter(|(sql, _)| sql.contains(marker))
                .count(),
            1
        );
    }
    assert_eq!(values.len(), 11);
}

#[test]
fn scalar_and_object_reference_containers_survive_callback_frames_and_slot_joins() {
    let values = outputs_file("reference-fragment-ownership", "query.cjs");
    for (marker, text) in [
        ("refreshed comparison bind", "sql_placeholder_1"),
        ("mutation through retained containers", "SELECT 10"),
        ("retained comparison callback", "SELECT 9"),
        ("callback comparison bind", "sql_placeholder_1"),
        ("callback selected object bind", "sql_placeholder_1"),
        ("joined references retain builder mutation", "SELECT 11"),
        (
            "joined scalar containers remain binds",
            "sql_placeholder_1 sql_placeholder_2",
        ),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if sql == text),
            "{marker}"
        );
    }
    assert!(
        matches!(event(&values, "callback object fragment"), Value::Prefix(text, false, _) if text.is_empty())
    );
    assert!(
        matches!(event(&values, "joined object fragment"), Value::Prefix(text, false, _) if text.is_empty())
    );
    let Value::Aggregate(candidates) = event(&values, "possible slot property") else {
        panic!("a possibly replaced object property cannot prove a SQL prefix");
    };
    assert!(candidates.iter().any(|candidate| matches!(candidate,
        Value::Aggregate(properties) if properties.iter().any(|value|
            matches!(value, Value::Prefix(text, true, _) if text == "SELECT 11")
        )
    )));
    assert_eq!(candidates.len(), 2, "both property-read paths must survive");
    assert!(matches!(
        event(&values, "joined comparison escape"),
        Value::Unknown
    ));
    assert_eq!(
        values
            .iter()
            .filter(|(sql, _)| sql.contains("retained comparison callback"))
            .count(),
        1
    );
}
