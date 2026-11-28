use super::fixture::empty_files;
use super::{SharedEntry, SharedInit};
use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn is_clean_shared_init_rejects_every_mutated_state_condition() {
    let target = PathBuf::from("target.mts");
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(std::slice::from_ref(&target), &facts);
    let target_id = ids.id(&target);
    let resolve = |_: &str, _: &std::path::Path| None;

    for case in 0..8 {
        let started_untrusted = case == 6;
        let shared = Arc::new(SharedInit {
            untrusted_array_intrinsics: started_untrusted,
            eligible: [target_id].into_iter().collect(),
            direct_deps: FxHashMap::default(),
            entries: dashmap::DashMap::new(),
        });
        let mut eval = evaluator::new(&files, resolve, started_untrusted);
        eval.modules.shared = Some(Arc::clone(&shared));
        let env = eval.environment_scope(Scope::default());
        if case == 7 {
            eval.builder_updates
                .insert(1, Value::Prefix("a".into(), false, Some(1)));
        }
        let guard = eval.begin_shared_capture(target_id, env).unwrap();
        match case {
            0 => eval.untrusted_array_intrinsics = true,
            1 => eval.event_log.push((target.clone(), (0, 1))),
            2 => {
                eval.invalidated_builders.insert(1);
            }
            3 => {
                eval.builder_updates.insert(1, Value::Unknown);
            }
            4 => {
                eval.captured_bindings
                    .insert(env, FxHashMap::from_iter([("x".into(), env)]).into());
            }
            5 => {
                eval.scopes[env].insert("arr".into(), Value::Array(1));
            }
            6 => eval.mark_untrusted_array_intrinsics(),
            _ => eval.replace_builder(&Value::Prefix("ab".into(), false, Some(1))),
        }
        eval.finish_shared_capture(target_id, env, guard);
        assert!(
            matches!(
                shared.entries.get(&target_id).as_deref(),
                Some(SharedEntry::Impure)
            ),
            "case {case} must mark target Impure"
        );
    }
}

#[test]
fn mismatched_mode_stays_pure_until_initialization_reads_the_flag() {
    let target = PathBuf::from("target.mts");
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(std::slice::from_ref(&target), &facts);
    let target_id = ids.id(&target);
    let resolve = |_: &str, _: &std::path::Path| None;
    let make_shared = || {
        Arc::new(SharedInit {
            untrusted_array_intrinsics: false,
            eligible: [target_id].into_iter().collect(),
            direct_deps: FxHashMap::default(),
            entries: dashmap::DashMap::new(),
        })
    };
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(Arc::clone(&shared));
    let env = eval.environment_scope(Scope::default());
    let guard = eval.begin_shared_capture(target_id, env).unwrap();
    eval.finish_shared_capture(target_id, env, guard);
    assert!(matches!(
        shared.entries.get(&target_id).as_deref(),
        Some(SharedEntry::Pure(_))
    ));

    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(Arc::clone(&shared));
    let env = eval.environment_scope(Scope::default());
    let guard = eval.begin_shared_capture(target_id, env).unwrap();
    eval.modules.untrusted_intrinsic_reads += 1;
    eval.finish_shared_capture(target_id, env, guard);
    assert!(matches!(
        shared.entries.get(&target_id).as_deref(),
        Some(SharedEntry::Impure)
    ));
}
