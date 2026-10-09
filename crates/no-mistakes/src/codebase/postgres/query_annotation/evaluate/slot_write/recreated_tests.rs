use super::super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn recreated_deleted_slot_callback_runs_only_after_the_arguments_object_escapes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-deleted-slot-callback/src/query.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: Default::default(),
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
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let call = |name: &str, sql: &str| query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name(name.into())),
        args: vec![query_annotation::Expr::Text(sql.into())],
        start: 0,
    };

    let no_escape = evaluator.expr(
        &call("recreatedCallbackWithoutEscape", "/* no escape */ SELECT 1"),
        &path,
        &root,
        16,
        false,
    );
    assert!(matches!(
        no_escape,
        Value::Prefix(text, true, None) if text == "/* no escape */ SELECT 1"
    ));

    let later_escape = evaluator.expr(
        &call("recreatedCallbackThenEscape", "/* later escape */ SELECT 1"),
        &path,
        &root,
        16,
        false,
    );
    assert!(matches!(later_escape, Value::Unknown));
}

#[test]
fn sloppy_formal_callback_writes_are_seen_after_escape_and_by_a_later_consumer() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-mapped-formal-callback/src/helper.cjs",
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
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.module_environment(&path);

    for (marker, expected) in [
        ("formal-write-after-escape", "SELECT 1"),
        ("second-consumer", "SELECT 3"),
    ] {
        let start = embedded
            .calls
            .iter()
            .zip(&embedded.call_starts)
            .find_map(|(call, start)| {
                source
                    .lines()
                    .nth(call.line as usize - 1)
                    .is_some_and(|line| line.contains(marker))
                    .then_some(*start)
            })
            .expect("marked nested executor call");
        assert!(
            matches!(
                evaluator.events.get(&(path.clone(), start)).and_then(|events| events.first()),
                Some((_, Value::Possible(values)))
                    if values.iter().any(|value| matches!(value, Value::Prefix(text, _, _) if text == expected))
            ),
            "missing modeled callback write for {marker}"
        );
    }

    let literal_start = embedded
        .calls
        .iter()
        .zip(&embedded.call_starts)
        .find_map(|(call, start)| {
            source
                .lines()
                .nth(call.line as usize - 1)
                .is_some_and(|line| line.contains("literal-formal-write-after-escape"))
                .then_some(*start)
        })
        .expect("marked literal callback executor call");
    assert!(matches!(
        evaluator.events.get(&(path.clone(), literal_start)).and_then(|events| events.first()),
        Some((_, Value::Prefix(text, true, None))) if text == "SELECT 4"
    ));

    let control_start = embedded
        .calls
        .iter()
        .zip(&embedded.call_starts)
        .find_map(|(call, start)| {
            source
                .lines()
                .nth(call.line as usize - 1)
                .is_some_and(|line| line.contains("known:noescape-control"))
                .then_some(*start)
        })
        .expect("marked no-escape write");
    assert!(matches!(
        evaluator.events.get(&(path, control_start)).and_then(|events| events.first()),
            Some((_, Value::Prefix(text, true, _))) if text == "/* no escape control */ SELECT 2"
    ));
}
