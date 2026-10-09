use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn returned_closures_observe_live_bindings_without_crossing_parameter_shadows() {
    for (name, expected) in [
        ("live.cjs", Some("SELECT 1")),
        ("fresh-index.cjs", Some("/* fresh annotation */ SELECT 1")),
        ("bounded.cjs", Some("/* annotation */ SELECT 1")),
        ("strict.cjs", Some("SELECT 1")),
        ("shadow.cjs", Some("/* local annotation */ SELECT 1")),
        ("conditional.cjs", None),
        ("sparse.cjs", None),
        ("rebind.cjs", Some("/* surviving alias */ SELECT 1")),
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
            mapped_arguments: Default::default(),
            argument_objects: Default::default(),
            argument_extra_slots: Default::default(),
            definite_deleted_argument_slots: Default::default(),
            recreated_argument_slots: Default::default(),
            disconnected_argument_slots: Default::default(),
            fresh_mapped_parameters: Default::default(),
            fresh_mapped_argument_bindings: Default::default(),
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
        if name == "fresh-index.cjs" {
            assert!(evaluator.fresh_mapped_argument_bindings.len() >= 32);
            let mut ids = evaluator
                .fresh_mapped_argument_bindings
                .keys()
                .copied()
                .collect::<Vec<_>>();
            ids.sort_unstable();
            let target = ids[0];
            let affected = evaluator.fresh_mapped_argument_bindings[&target].clone();
            assert!(
                affected.len() <= 2,
                "one container targets only its actual formal readers"
            );
            let before = evaluator.fresh_mapped_parameters.clone();
            evaluator.invalidate_mapped_freshness(&crate::fx::FxHashSet::from_iter([target]));
            assert!(!evaluator
                .fresh_mapped_argument_bindings
                .contains_key(&target));
            for (frame, names) in &before {
                for name in names {
                    let should_survive = !affected
                        .get(frame)
                        .is_some_and(|affected| affected.contains(name));
                    assert_eq!(
                        evaluator
                            .fresh_mapped_parameters
                            .get(frame)
                            .is_some_and(|names| names.contains(name)),
                        should_survive
                    );
                }
            }
            // Clear every remaining proof for one real captured formal environment.
            // Empty proof entries must disappear rather than accumulate across calls.
            let (frame, names) = evaluator
                .fresh_mapped_parameters
                .iter()
                .find(|(frame, names)| {
                    !names.is_empty()
                        && names.iter().all(|name| {
                            evaluator
                                .fresh_mapped_argument_bindings
                                .values()
                                .any(|bindings| {
                                    bindings
                                        .get(frame)
                                        .is_some_and(|bound| bound.contains(name))
                                })
                        })
                })
                .map(|(frame, names)| (*frame, names.clone()))
                .expect("saved fixture retains another mapped formal proof");
            let remaining = evaluator
                .fresh_mapped_argument_bindings
                .iter()
                .filter(|(_, bindings)| {
                    bindings
                        .get(&frame)
                        .is_some_and(|bound| !bound.is_disjoint(&names))
                })
                .map(|(id, _)| *id)
                .collect();
            evaluator.invalidate_mapped_freshness(&remaining);
            assert!(!evaluator.fresh_mapped_parameters.contains_key(&frame));
            // A derived bucket can outlive its proof after another identity's escape.
            // Invalidating that stale reader must never recreate the expired proof.
            let stale_id = *evaluator
                .fresh_mapped_argument_bindings
                .keys()
                .next()
                .expect("saved fixture retains independent argument identities");
            let stale_frames = evaluator.fresh_mapped_argument_bindings[&stale_id]
                .keys()
                .copied()
                .collect::<Vec<_>>();
            for stale_frame in &stale_frames {
                evaluator.fresh_mapped_parameters.remove(stale_frame);
            }
            evaluator.invalidate_mapped_freshness(&crate::fx::FxHashSet::from_iter([stale_id]));
            assert!(!evaluator
                .fresh_mapped_argument_bindings
                .contains_key(&stale_id));
            for stale_frame in stale_frames {
                assert!(!evaluator.fresh_mapped_parameters.contains_key(&stale_frame));
            }
        }
        let result = &evaluator.scopes[env]["result"];
        if name == "rebind.cjs" {
            assert!(
                matches!(result, Value::Prefix(text, true, Some(_)) if text == "/* surviving alias */ SELECT 1")
            );
            continue;
        }
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
