use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn outputs() -> Vec<(String, Value)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-discarded-values/src/query.mts",
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
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.module_environment(&path);
    embedded
        .calls
        .iter()
        .zip(&embedded.call_starts)
        .filter_map(|(call, start)| {
            let sql = call.sql_text.as_ref()?;
            let values = evaluator.events.get(&(path.clone(), *start))?;
            Some(values.iter().map(|(_, value)| (sql.clone(), value.clone())))
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
    let values = outputs();
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
