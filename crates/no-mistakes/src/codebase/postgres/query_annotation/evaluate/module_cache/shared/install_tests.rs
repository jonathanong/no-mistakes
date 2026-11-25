use super::{CachedModule, SharedEntry, SharedInit};
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope, Value};
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
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

#[test]
fn try_install_shared_module_rejects_untrusted_impure_and_mismatched_deps() {
    let paths: Vec<PathBuf> = ["d1.mts", "d2.mts", "top.mts"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let facts = QueryAnnotationFileFacts::default();
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let module =
        Arc::new(Module::new(Arc::new(TsFileFacts::default())).with_module_ids(Arc::clone(&ids)));
    let mut files = FxHashMap::default();
    for path in &paths {
        files.insert(
            path.clone(),
            File {
                facts: &facts,
                module: Arc::clone(&module),
                executors: FxHashSet::default(),
            },
        );
    }
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
        SharedEntry::Pure(Arc::new(CachedModule {
            deps: vec![d1_id, d2_id],
            dep_refs: vec![(d1_id, 0), (d2_id, 0)],
            dep_scopes: vec![d1_scope, d2_scope],
            own_scopes: vec![Scope::default()],
        })),
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
}
