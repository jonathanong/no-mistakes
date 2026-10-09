use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn only_deletions_on_every_alternative_disconnect_mapped_parameters() {
    for (name, definite) in [
        ("single.cjs", false),
        ("both.cjs", true),
        ("nested.cjs", false),
        ("logical.cjs", false),
        (
            "../helper-tracing-mapped-context/escaped-then-deleted-alternatives.cjs",
            false,
        ),
        (
            "../helper-tracing-mapped-context/escaped-on-one-then-deleted-alternatives.cjs",
            false,
        ),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-deletion-merge",
        ).join(name);
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
            next_builder: 0,
            invalidated_builders: Default::default(),
            builder_updates: Default::default(),
            captured_bindings: Default::default(),
            captured_binding_readers: Default::default(),
            mapped_argument_owners: Default::default(),
            deleted_argument_slots: Default::default(),
            mapped_arguments: Default::default(),
            fresh_mapped_parameters: Default::default(),
            fresh_mapped_argument_bindings: Default::default(),
            argument_objects: Default::default(),
            argument_extra_slots: Default::default(),
            definite_deleted_argument_slots: Default::default(),
            disconnected_argument_slots: Default::default(),
        };
        let env = evaluator.module_environment(&path);
        let result = &evaluator.scopes[env]["result"];
        if definite {
            assert!(
                matches!(result, Value::Prefix(text, true, None) if text == "/* definite deletion */ SELECT 1"),
                "{name}"
            );
        } else if let Some(expected) = match name {
            "single.cjs" => Some("/* possible deletion */ SELECT 1"),
            "nested.cjs" => Some("/* nested possible deletion */ SELECT 1"),
            "logical.cjs" => Some("/* logical possible deletion */ SELECT 1"),
            _ => None,
        } {
            assert!(
                matches!(result, Value::Possible(_)),
                "{name} remains an uncertain value"
            );
            fn contains(value: &Value, expected: &str) -> bool {
                match value {
                    Value::Prefix(text, true, None) => text == expected,
                    Value::Possible(values) | Value::Aggregate(values) => {
                        values.iter().any(|value| contains(value, expected))
                    }
                    _ => false,
                }
            }
            assert!(
                contains(result, expected),
                "{name} retains its possible original prefix"
            );
        } else {
            assert!(
                matches!(result, Value::Unknown),
                "{name} cannot prove disconnection"
            );
        }
    }
}
