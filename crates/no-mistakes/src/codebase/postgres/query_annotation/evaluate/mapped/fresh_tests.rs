use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn definite_parameter_assignments_remain_fresh_until_another_container_escape() {
    for (name, known) in [
        ("callback.cjs", false),
        ("scalar-disconnect.cjs", false),
        ("escaped-dynamic-disconnect.cjs", false),
        ("callback-reescape.cjs", false),
        ("rhs-before-target.cjs", false),
        ("nested-callbacks.cjs", false),
        ("direct.cjs", true),
        ("opaque-read.cjs", true),
        ("arguments-write.cjs", false),
        ("arguments-write-formal.cjs", true),
        ("captured-arguments-write.cjs", false),
        ("escape-again.cjs", false),
        ("inherited.cjs", true),
        ("deleted.cjs", true),
        ("unrelated.cjs", true),
        ("conditional-retain.cjs", true),
        ("conditional-escape.cjs", false),
        ("frame.cjs", true),
        ("unsupported-write.cjs", false),
        ("update-write.cjs", false),
        ("destructure-write.cjs", false),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-fresh-mapped-parameter",
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
            active_callback_executions: Default::default(),
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
            disconnected_argument_slots: Default::default(),
            fresh_mapped_parameters: Default::default(),
            fresh_mapped_argument_bindings: Default::default(),
        };
        let env = evaluator.module_environment(&path);
        let result = &evaluator.scopes[env]["result"];
        if name == "scalar-disconnect.cjs" || name == "escaped-dynamic-disconnect.cjs" {
            assert!(
                matches!(result, Value::Possible(values) if values.iter().any(|value| matches!(value, Value::Prefix(text, true, None) if text == "SELECT 1")))
            );
            continue;
        }
        if name == "arguments-write-formal.cjs" {
            assert!(
                matches!(result, Value::Prefix(text, true, Some(_)) if text == "/* detached formal */ SELECT 1")
            );
            continue;
        }
        if name.ends_with("arguments-write.cjs") {
            assert!(
                matches!(result, Value::Aggregate(values) if values.iter().all(|value| matches!(value, Value::Unknown)))
            );
            continue;
        }
        if name == "frame.cjs" {
            let Value::Aggregate(values) = result else {
                panic!("conditional wrapper");
            };
            let Value::Aggregate(arms) = &values[1] else {
                panic!("alternative wrapper");
            };
            let Value::Function(_, _, captured) = &arms[0] else {
                panic!("retained callback");
            };
            assert_eq!(*captured, 1);
            assert!(evaluator.fresh_mapped_parameters[captured].contains("parameter"));
            let callback = arms[0].clone();
            // Probe the private read boundary using the saved factory's real
            // callback identity, including multiple inherited mapping sets.
            let id = 1000;
            evaluator.argument_objects.insert(
                id,
                vec![
                    Value::Evaluated(Box::new(Value::Promise(Box::new(callback.clone()))), true),
                    Value::Aggregate(vec![callback.clone()]),
                ],
            );
            evaluator.mapped_arguments.insert(
                env,
                vec![
                    (id, vec!["probe".into(), "probe".into()]),
                    (id, vec!["probe".into(), "other".into()]),
                    (id, vec!["probe".into(), "other".into()]),
                ],
            );
            evaluator.invalidated_builders.insert(id);
            let projected = evaluator.mapped_parameter_value(env, "probe").unwrap();
            assert!(matches!(projected, Value::Possible(ref values) if values.len() == 2));
            evaluator.disconnected_argument_slots.insert((id, 1));
            assert!(
                matches!(evaluator.mapped_parameter_value(env, "probe"), Some(Value::Possible(ref values)) if values.len() == 1)
            );
            assert!(evaluator.mapped_parameter_value(env, "unmapped").is_none());
            // A proof without mapping metadata belongs to no escaping object.
            evaluator
                .fresh_mapped_parameters
                .insert(999, crate::fx::FxHashSet::from_iter(["local".into()]));
            evaluator.invalidate_mapped_freshness(&crate::fx::FxHashSet::from_iter([id]));
            assert!(evaluator.fresh_mapped_parameters[&999].contains("local"));
            // Disconnection uses the invocation's canonical owner projection;
            // the overlapping read probes above are not actual invocation maps.
            evaluator
                .mapped_arguments
                .insert(env, vec![(id, vec!["probe".into(), "other".into()])]);
            evaluator.rebuild_mapped_argument_owners();
            // A stale mapped reader may lack a scoped copy; live candidates
            // still survive without inventing a missing formal value.
            evaluator.deleted_argument_slots.insert((id, None));
            evaluator.scopes[env].remove("probe");
            assert!(matches!(
                evaluator.mapped_parameter_value(env, "probe"),
                Some(Value::Possible(_))
            ));
            evaluator.scopes[env].insert("probe".into(), Value::Unknown);
            assert!(
                matches!(
                    evaluator.mapped_parameter_value(env, "probe"),
                    Some(Value::Possible(_))
                ),
                "unknown copied formal does not discard live callback possibilities"
            );
            evaluator.scopes[env].insert("probe".into(), callback.clone());
            evaluator.disconnect_mapped_slot(id, 0);
            assert!(matches!(evaluator.scopes[env]["probe"], Value::Possible(_)));
            continue;
        }
        if name == "callback-reescape.cjs" {
            assert!(
                matches!(result, Value::Possible(values) if values.iter().any(|value| matches!(value, Value::Prefix(text, true, None) if text == "/* re-escaped value */ SELECT 1"))),
                "callback effects must not restore a guaranteed annotation"
            );
            continue;
        }
        if known {
            assert!(
                matches!(result, Value::Prefix(text, true, None) if text == "/* fresh annotation */ SELECT 1"),
                "{name}"
            );
        } else {
            assert!(
                matches!(result, Value::Unknown | Value::Possible(_)),
                "{name} must remain unproven"
            );
        }
    }
}
