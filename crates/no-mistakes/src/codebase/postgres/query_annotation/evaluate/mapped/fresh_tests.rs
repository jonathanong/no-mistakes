use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn definite_parameter_assignments_remain_fresh_until_another_container_escape() {
    for (name, known) in [
        ("callback.cjs", false),
        ("nested-callbacks.cjs", false),
        ("direct.cjs", true),
        ("opaque-read.cjs", true),
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
            next_builder: 0,
            invalidated_builders: Default::default(),
            builder_updates: Default::default(),
            captured_bindings: Default::default(),
            deleted_argument_slots: Default::default(),
            mapped_arguments: Default::default(),
            argument_objects: Default::default(),
            definite_deleted_argument_slots: Default::default(),
            fresh_mapped_parameters: Default::default(),
        };
        let env = evaluator.module_environment(&path);
        let result = &evaluator.scopes[env]["result"];
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
            assert!(matches!(projected, Value::Aggregate(ref values) if values.len() == 3));
            evaluator.definite_deleted_argument_slots.insert((id, 1));
            assert!(
                matches!(evaluator.mapped_parameter_value(env, "probe"), Some(Value::Aggregate(ref values)) if values.len() == 2)
            );
            assert!(evaluator.mapped_parameter_value(env, "unmapped").is_none());
            // A proof without mapping metadata belongs to no escaping object.
            evaluator
                .fresh_mapped_parameters
                .insert(999, crate::fx::FxHashSet::from_iter(["local".into()]));
            evaluator.invalidate_mapped_freshness(&crate::fx::FxHashSet::from_iter([id]));
            assert!(evaluator.fresh_mapped_parameters[&999].contains("local"));
            evaluator.scopes[env].insert("probe".into(), callback.clone());
            evaluator.disconnect_mapped_slot(id, 0);
            assert!(matches!(
                evaluator.scopes[env]["probe"],
                Value::Aggregate(_)
            ));
            continue;
        }
        if known {
            assert!(
                matches!(result, Value::Prefix(text, true, None) if text == "/* fresh annotation */ SELECT 1"),
                "{name}"
            );
        } else {
            assert!(
                matches!(result, Value::Unknown),
                "{name} must remain unproven"
            );
        }
    }
}
