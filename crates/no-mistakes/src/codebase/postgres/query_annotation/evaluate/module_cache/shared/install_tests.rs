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

pub(super) fn empty_files<'a>(
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

pub(super) fn pure_entry(
    deps: Vec<ModuleId>,
    dep_refs: Vec<(ModuleId, usize)>,
    dep_scopes: Vec<Scope>,
) -> SharedEntry {
    pure_with_scope(deps, dep_refs, dep_scopes, Scope::default(), false)
}

fn pure_with_scope(
    deps: Vec<ModuleId>,
    dep_refs: Vec<(ModuleId, usize)>,
    dep_scopes: Vec<Scope>,
    own: Scope,
    reads_untrusted: bool,
) -> SharedEntry {
    SharedEntry::Pure(Arc::new(CachedModule {
        deps,
        dep_refs,
        dep_scopes,
        own_scopes: vec![own],
        reads_untrusted,
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
            pure_with_scope(Vec::new(), Vec::new(), Vec::new(), d1_scope.clone(), false),
        );
        shared.entries.insert(
            d2_id,
            pure_with_scope(Vec::new(), Vec::new(), Vec::new(), d2_scope.clone(), false),
        );
        shared.entries.insert(
            top_id,
            pure_with_scope(
                vec![d1_id, d2_id],
                vec![(d1_id, 0), (d2_id, 0)],
                vec![d1_scope.clone(), d2_scope.clone()],
                Scope::default(),
                true,
            ),
        );
        shared
    };

    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    let shared = make_shared();
    shared.entries.insert(d1_id, SharedEntry::Impure);
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[0], d1_id).is_none());
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    let d2_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "f".into(),
        dummy_fn(1),
    )])));
    eval.modules.insert(d2_id, d2_env);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    // Loaded d1 with a remapped self-closure matches when sibling d2 is unloaded.
    let shared = make_shared();
    let d1_fn = Scope::from(FxHashMap::from_iter([("g".into(), dummy_fn(0))]));
    shared.entries.insert(
        d1_id,
        pure_with_scope(Vec::new(), Vec::new(), Vec::new(), d1_fn.clone(), false),
    );
    shared.entries.insert(
        top_id,
        pure_entry(
            vec![d1_id, d2_id],
            vec![(d1_id, 0), (d2_id, 0)],
            vec![d1_fn, d2_scope.clone()],
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

    // Out-of-bounds slot, missing dep entry, missing dep_ref, and failing nested dep check.
    let shared = make_shared();
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    eval.modules.insert(d2_id, 999);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    let shared = make_shared();
    shared.entries.remove(&d1_id);
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

    let shared = make_shared();
    shared.entries.insert(
        top_id,
        pure_entry(Vec::new(), vec![(d1_id, 0)], vec![Scope::default()]),
    );
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(shared);
    assert!(eval.try_install_shared_module(&paths[2], top_id).is_none());

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
