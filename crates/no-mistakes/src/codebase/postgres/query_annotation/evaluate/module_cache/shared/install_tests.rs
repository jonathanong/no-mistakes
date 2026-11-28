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
        reads_untrusted: false,
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
    let make_shared = || {
        let shared = Arc::new(SharedInit {
            untrusted_array_intrinsics: false,
            eligible: [d1_id, d2_id, top_id].into_iter().collect(),
            direct_deps: FxHashMap::default(),
            entries: dashmap::DashMap::new(),
        });
        shared.entries.insert(
            d1_id,
            SharedEntry::Pure(Arc::new(CachedModule {
                deps: Vec::new(),
                dep_refs: Vec::new(),
                dep_scopes: Vec::new(),
                own_scopes: vec![d1_scope.clone()],
                reads_untrusted: false,
            })),
        );
        shared.entries.insert(
            d2_id,
            SharedEntry::Pure(Arc::new(CachedModule {
                deps: Vec::new(),
                dep_refs: Vec::new(),
                dep_scopes: Vec::new(),
                own_scopes: vec![d2_scope.clone()],
                reads_untrusted: false,
            })),
        );
        shared.entries.insert(
            top_id,
            SharedEntry::Pure(Arc::new(CachedModule {
                deps: vec![d1_id, d2_id],
                dep_refs: vec![(d1_id, 0), (d2_id, 0)],
                dep_scopes: vec![d1_scope.clone(), d2_scope.clone()],
                own_scopes: vec![Scope::default()],
                reads_untrusted: true,
            })),
        );
        shared
    };

    // Untrusted mode mismatch rejects install.
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Impure target entry rejects install.
    let shared = make_shared();
    shared.entries.insert(d1_id, SharedEntry::Impure);
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[0], d1_id).is_none());
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Loaded d2 referencing an unloaded d1 slot fails remapped scope comparison.
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    let d2_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "f".into(),
        dummy_fn(1),
    )])));
    eval.modules.insert(d2_id, d2_env);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Loaded d1 with a remapped self-closure still matches when sibling d2 is unloaded.
    let shared = make_shared();
    let d1_fn_scope = Scope::from(FxHashMap::from_iter([("g".into(), dummy_fn(0))]));
    shared.entries.insert(
        d1_id,
        SharedEntry::Pure(Arc::new(CachedModule {
            deps: Vec::new(),
            dep_refs: Vec::new(),
            dep_scopes: Vec::new(),
            own_scopes: vec![d1_fn_scope.clone()],
            reads_untrusted: false,
        })),
    );
    shared.entries.insert(
        top_id,
        pure_entry(
            vec![d1_id, d2_id],
            vec![(d1_id, 0), (d2_id, 0)],
            vec![d1_fn_scope, d2_scope.clone()],
        ),
    );
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    let _pad = eval.environment_scope(Scope::default());
    let d1_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "g".into(),
        dummy_fn(1),
    )])));
    eval.modules.insert(d1_id, d1_env);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_some());

    // Out-of-bounds slot in deps_match_cached falls back cleanly.
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    eval.modules.insert(d2_id, 999);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Missing shared entry for dependency falls back cleanly.
    let shared = make_shared();
    shared.entries.remove(&d1_id);
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Missing loaded dep_ref during remap falls back cleanly.
    let shared = make_shared();
    shared.entries.insert(
        top_id,
        pure_entry(Vec::new(), vec![(d1_id, 0)], vec![Scope::default()]),
    );
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Pre-installing a dependency whose own deps_match_cached fails falls back cleanly.
    let shared = make_shared();
    shared.entries.insert(
        d1_id,
        pure_entry(Vec::new(), vec![(d2_id, 0)], vec![Scope::default()]),
    );
    shared
        .entries
        .insert(top_id, pure_entry(vec![d1_id], Vec::new(), Vec::new()));
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    let d2_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "f".into(),
        dummy_fn(1),
    )])));
    eval.modules.insert(d2_id, d2_env);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());
}

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
