use super::tests::{empty_files, pure_entry};
use super::{SharedEntry, SharedInit};
use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn capture_defers_without_panicking_when_transitive_dep_is_unloaded_or_invalid() {
    let paths = ["leaf.mts", "mid.mts", "top.mts"].map(PathBuf::from);
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(&paths, &facts);
    let [leaf_id, mid_id, top_id] = [ids.id(&paths[0]), ids.id(&paths[1]), ids.id(&paths[2])];
    let resolve = |_: &str, _: &std::path::Path| None;

    let make_shared = || {
        let shared = Arc::new(SharedInit {
            untrusted_array_intrinsics: false,
            eligible: [leaf_id, mid_id, top_id].into_iter().collect(),
            direct_deps: FxHashMap::from_iter([(mid_id, vec![leaf_id]), (top_id, vec![mid_id])]),
            entries: dashmap::DashMap::new(),
        });
        shared
            .entries
            .insert(leaf_id, pure_entry(Vec::new(), Vec::new(), Vec::new()));
        shared
    };

    // Unloaded lazy dep stays absent from both CachedModule.deps and dep_refs
    // so installing mid or top never pre-initializes leaf before first access.
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    let mid_env = eval.environment_scope(Scope::default());
    eval.modules.insert(mid_id, mid_env);
    let guard = eval.begin_shared_capture(mid_id, mid_env).unwrap();
    eval.finish_shared_capture(mid_id, mid_env, guard);
    assert!(matches!(
        shared.entries.get(&mid_id).as_deref(),
        Some(SharedEntry::Pure(cached)) if cached.deps.is_empty() && cached.dep_refs.is_empty()
    ));
    let top_env = eval.environment_scope(Scope::default());
    eval.modules.insert(top_id, top_env);
    let guard = eval.begin_shared_capture(top_id, top_env).unwrap();
    eval.finish_shared_capture(top_id, top_env, guard);
    assert!(matches!(
        shared.entries.get(&top_id).as_deref(),
        Some(SharedEntry::Pure(cached)) if cached.deps == vec![mid_id] && cached.dep_refs == vec![(mid_id, 0)]
    ));
    let mut eval_install = evaluator::new(&files, resolve, false);
    eval_install.modules.shared = Some(Arc::clone(&shared));
    assert!(eval_install
        .try_install_shared_module(&paths[2], top_id)
        .is_some());
    assert!(eval_install.modules.contains_key(&mid_id));
    assert!(!eval_install.modules.contains_key(&leaf_id));
    // Deferred lazy dependency only installs on first access and still checks its own deps.
    let mut scalar_scope = Scope::from(FxHashMap::from_iter([("v".into(), Value::Number(1))]));
    shared.entries.insert(
        leaf_id,
        pure_entry(Vec::new(), vec![(mid_id, 0)], vec![scalar_scope.clone()]),
    );
    scalar_scope.insert("v".into(), Value::Number(2));
    eval_install.scopes[eval_install.modules[&mid_id]] = scalar_scope;
    assert!(eval_install
        .try_install_shared_module(&paths[0], leaf_id)
        .is_none());

    // Isolate each invalid transitive-dependency case so earlier leaf state cannot mask it.
    for step in 0..5 {
        let shared = make_shared();
        shared.entries.insert(
            mid_id,
            pure_entry(vec![leaf_id], vec![(leaf_id, 0)], vec![Scope::default()]),
        );
        let mut eval = evaluator::new(&files, resolve, false);
        eval.modules.shared = Some(Arc::clone(&shared));
        let mid_env = eval.environment_scope(Scope::default());
        eval.modules.insert(mid_id, mid_env);
        let top_env = eval.environment_scope(Scope::default());
        eval.modules.insert(top_id, top_env);
        match step {
            0 => {}
            1 => {
                eval.modules.insert(leaf_id, 999);
            }
            2 => {
                let leaf_env = eval.environment_scope(Scope::default());
                eval.modules.insert(leaf_id, leaf_env);
                shared.entries.remove(&leaf_id);
            }
            3 => {
                let leaf_env = eval.environment_scope(Scope::default());
                eval.modules.insert(leaf_id, leaf_env);
                shared.entries.insert(leaf_id, SharedEntry::Impure);
            }
            _ => {
                let leaf_env = eval.environment_scope(Scope::default());
                eval.modules.insert(leaf_id, leaf_env);
                shared.entries.insert(
                    mid_id,
                    pure_entry(vec![leaf_id], vec![(leaf_id, 1)], vec![Scope::default()]),
                );
            }
        }
        let guard = eval.begin_shared_capture(top_id, top_env).unwrap();
        eval.finish_shared_capture(top_id, top_env, guard);
        assert!(shared.entries.get(&top_id).is_none(), "step {step}");
    }
}
