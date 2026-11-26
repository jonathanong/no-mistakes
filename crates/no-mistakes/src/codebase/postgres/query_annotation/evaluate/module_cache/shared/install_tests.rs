use super::{CachedModule, SharedEntry, SharedInit};
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope, Value};
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::{Function, QueryAnnotationFileFacts};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn dummy_fn(env: usize) -> Value {
    let function = Arc::new(Function {
        start: 0,
        params: Vec::new(),
        body: Vec::new(),
        supported: true,
        asynchronous: false,
        arrow: true,
        self_name: None,
        append_callback: None,
    });
    Value::Function(function, Arc::new(PathBuf::from("a.mts")), env)
}

fn empty_files<'a>(
    paths: &[PathBuf],
    facts: &'a QueryAnnotationFileFacts,
) -> (Arc<ModuleIds>, FxHashMap<PathBuf, File<'a>>) {
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let module =
        Arc::new(Module::new(Arc::new(TsFileFacts::default())).with_module_ids(Arc::clone(&ids)));
    let files = paths
        .iter()
        .map(|p| {
            (
                p.clone(),
                File {
                    facts,
                    module: Arc::clone(&module),
                    executors: FxHashSet::default(),
                },
            )
        })
        .collect();
    (ids, files)
}

fn pure_entry(
    deps: Vec<ModuleId>,
    dep_refs: Vec<(ModuleId, usize)>,
    dep_scopes: Vec<Scope>,
) -> SharedEntry {
    SharedEntry::Pure(Arc::new(CachedModule {
        deps,
        dep_refs,
        dep_scopes,
        own_scopes: vec![Scope::default()],
    }))
}

#[test]
fn try_install_shared_module_rejects_untrusted_impure_and_mismatched_deps() {
    let paths = ["d1.mts", "d2.mts", "top.mts"].map(PathBuf::from);
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(&paths, &facts);
    let [d1_id, d2_id, top_id] = [ids.id(&paths[0]), ids.id(&paths[1]), ids.id(&paths[2])];
    let resolve = |_: &str, _: &std::path::Path| None;

    let d1_scope = Scope::from(FxHashMap::from_iter([("x".into(), Value::Number(1))]));
    let d2_scope = Scope::from(FxHashMap::from_iter([("f".into(), dummy_fn(0))]));
    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [d1_id, d2_id, top_id].into_iter().collect(),
        direct_deps: FxHashMap::default(),
        entries: dashmap::DashMap::new(),
    });
    shared.entries.insert(d1_id, SharedEntry::Impure);
    shared.entries.insert(
        top_id,
        pure_entry(
            vec![d1_id, d2_id],
            vec![(d1_id, 0), (d2_id, 0)],
            vec![d1_scope, d2_scope],
        ),
    );

    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));

    eval.untrusted_array_intrinsics = true;
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
    eval.untrusted_array_intrinsics = false;
    assert!(eval.try_install_shared_module(&paths[0], d1_id).is_none());

    let d2_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "f".into(),
        dummy_fn(1),
    )])));
    eval.modules.insert(d2_id, d2_env);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Out-of-bounds slot in deps_match_cached, missing/impure dep, and missing dep_ref fall back.
    eval.modules.insert(d2_id, 999);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
    eval.modules.remove(&d2_id);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
    shared.entries.remove(&d1_id);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
    shared.entries.insert(
        top_id,
        pure_entry(Vec::new(), vec![(d1_id, 0)], vec![Scope::default()]),
    );
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
    eval.modules.insert(d2_id, d2_env);
    shared.entries.insert(
        d1_id,
        pure_entry(Vec::new(), vec![(d2_id, 0)], vec![Scope::default()]),
    );
    shared
        .entries
        .insert(top_id, pure_entry(vec![d1_id], Vec::new(), Vec::new()));
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
}

#[test]
fn capture_defers_without_panicking_when_transitive_dep_is_unloaded_or_invalid() {
    let paths = ["leaf.mts", "mid.mts", "top.mts"].map(PathBuf::from);
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(&paths, &facts);
    let [leaf_id, mid_id, top_id] = [ids.id(&paths[0]), ids.id(&paths[1]), ids.id(&paths[2])];
    let resolve = |_: &str, _: &std::path::Path| None;

    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [leaf_id, mid_id, top_id].into_iter().collect(),
        direct_deps: FxHashMap::from_iter([(mid_id, vec![leaf_id]), (top_id, vec![mid_id])]),
        entries: dashmap::DashMap::new(),
    });
    shared
        .entries
        .insert(leaf_id, pure_entry(Vec::new(), Vec::new(), Vec::new()));

    // Unloaded lazy dep stays in CachedModule.deps for preinstall, but is absent from dep_refs
    // so dependent top captures cleanly without leaf loaded in self.modules.
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    let mid_env = eval.environment_scope(Scope::default());
    eval.modules.insert(mid_id, mid_env);
    let guard = eval.begin_shared_capture(mid_id, mid_env).unwrap();
    eval.finish_shared_capture(mid_id, mid_env, guard);
    assert!(matches!(
        shared.entries.get(&mid_id).as_deref(),
        Some(SharedEntry::Pure(cached)) if cached.deps == vec![leaf_id] && cached.dep_refs.is_empty()
    ));
    let top_env = eval.environment_scope(Scope::default());
    eval.modules.insert(top_id, top_env);
    let guard = eval.begin_shared_capture(top_id, top_env).unwrap();
    eval.finish_shared_capture(top_id, top_env, guard);
    assert!(matches!(
        shared.entries.get(&top_id).as_deref(),
        Some(SharedEntry::Pure(cached)) if cached.dep_refs == vec![(mid_id, 0)]
    ));
    shared.entries.remove(&top_id);

    // When mid cached leaf in dep_refs under another root but this evaluator lacks leaf, top defers.
    shared.entries.insert(
        mid_id,
        pure_entry(vec![leaf_id], vec![(leaf_id, 0)], vec![Scope::default()]),
    );
    for step in 0..5 {
        match step {
            0 => {}
            1 => {
                eval.modules.insert(leaf_id, 999);
            }
            2 => {
                shared.entries.remove(&leaf_id);
            }
            3 => {
                shared.entries.insert(leaf_id, SharedEntry::Impure);
            }
            _ => {
                shared
                    .entries
                    .insert(leaf_id, pure_entry(Vec::new(), Vec::new(), Vec::new()));
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
