use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn returned_closures_observe_live_bindings_without_crossing_parameter_shadows() {
    for (name, expected) in [
        ("live.cjs", Some("SELECT 1")),
        ("bounded.cjs", Some("/* annotation */ SELECT 1")),
        ("strict.cjs", Some("SELECT 1")),
        ("shadow.cjs", Some("/* local annotation */ SELECT 1")),
        ("conditional.cjs", None),
        ("sparse.cjs", None),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-live-binding/src",
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
            next_builder: 0,
            invalidated_builders: Default::default(),
            builder_updates: Default::default(),
            captured_bindings: Default::default(),
            captured_binding_readers: Default::default(),
            mapped_argument_owners: Default::default(),
            deleted_argument_slots: Default::default(),
            mapped_arguments: Default::default(),
            argument_objects: Default::default(),
            argument_extra_slots: Default::default(),
            definite_deleted_argument_slots: Default::default(),
            fresh_mapped_parameters: Default::default(),
        };
        let env = evaluator.module_environment(&path);
        if name == "bounded.cjs" {
            assert!(
                evaluator.scopes.len() >= 33,
                "many earlier invocation frames exist"
            );
            for frame in 0..evaluator.scopes.len() {
                assert_eq!(
                    evaluator.captured_write_targets(frame, "unrelated_local"),
                    vec![frame]
                );
            }
            let outer = evaluator.scopes[env]["make"].clone();
            evaluator.write_captured_binding(env, "make", &outer);
            assert!(evaluator.captured_write_targets(env, "make").len() > 1);
            assert_eq!(
                evaluator.captured_write_targets(env, "make").len(),
                1 + evaluator.captured_binding_readers[&env]["make"].len()
            );
            assert!(!evaluator.captured_binding_readers[&env].contains_key("unrelated_local"));
        }
        let result = &evaluator.scopes[env]["result"];
        if let Some(expected) = expected {
            assert!(
                matches!(result, Value::Prefix(text, true, None) if text == expected),
                "{name}"
            );
        } else {
            assert!(
                !matches!(result, Value::Prefix(_, _, _)),
                "conditional writes cannot retain old SQL proof"
            );
        }
    }
}
