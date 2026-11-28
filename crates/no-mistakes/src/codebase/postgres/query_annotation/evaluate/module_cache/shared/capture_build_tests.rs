use super::fixture::empty_files;
use super::{CachedModule, SharedEntry, SharedInit};
use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::{Function, QueryAnnotationFileFacts};
use crate::fx::FxHashMap;
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
fn build_cached_module_handles_defer_impure_dep_mutation_foreign_env_and_diamond() {
    let paths = ["base.mts", "a.mts", "b.mts", "top.mts"].map(PathBuf::from);
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(&paths, &facts);
    let [base_id, a_id, b_id, top_id] = [
        ids.id(&paths[0]),
        ids.id(&paths[1]),
        ids.id(&paths[2]),
        ids.id(&paths[3]),
    ];
    let resolve = |_: &str, _: &std::path::Path| None;

    let make_shared = |deps: FxHashMap<_, Vec<_>>| {
        Arc::new(SharedInit {
            untrusted_array_intrinsics: false,
            eligible: [base_id, a_id, b_id, top_id].into_iter().collect(),
            direct_deps: deps,
            entries: dashmap::DashMap::new(),
        })
    };

    // Deferred when direct dep not yet cached; Impure when direct dep is Impure.
    let shared = make_shared(FxHashMap::from_iter([(top_id, vec![base_id])]));
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    let env = eval.environment_scope(Scope::default());
    let guard = eval.begin_shared_capture(top_id, env).unwrap();
    eval.finish_shared_capture(top_id, env, guard);
    assert!(shared.entries.get(&top_id).is_none());

    shared.entries.insert(base_id, SharedEntry::Impure);
    let guard = eval.begin_shared_capture(top_id, env).unwrap();
    eval.finish_shared_capture(top_id, env, guard);
    assert!(matches!(
        shared.entries.get(&top_id).as_deref(),
        Some(SharedEntry::Impure)
    ));

    // Mutated dep scope and foreign-env function in own scope both mark Impure.
    let shared = make_shared(FxHashMap::from_iter([(a_id, vec![base_id])]));
    shared.entries.insert(
        base_id,
        SharedEntry::Pure(Arc::new(CachedModule {
            deps: Vec::new(),
            dep_refs: Vec::new(),
            dep_scopes: Vec::new(),
            own_scopes: vec![Scope::from(FxHashMap::from_iter([(
                "k".into(),
                Value::Number(1),
            )]))],
            reads_untrusted: false,
        })),
    );
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    let base_env = eval.environment_scope(Scope::from(FxHashMap::from_iter([(
        "k".into(),
        Value::Number(2),
    )])));
    eval.modules.insert(base_id, base_env);
    let a_env = eval.environment_scope(Scope::default());
    let guard = eval.begin_shared_capture(a_id, a_env).unwrap();
    eval.finish_shared_capture(a_id, a_env, guard);
    assert!(matches!(
        shared.entries.get(&a_id).as_deref(),
        Some(SharedEntry::Impure)
    ));

    shared.entries.remove(&a_id);
    eval.scopes[base_env] = Scope::from(FxHashMap::from_iter([("k".into(), Value::Number(1))]));
    eval.scopes[a_env] = Scope::from(FxHashMap::from_iter([("f".into(), dummy_fn(999))]));
    let guard = eval.begin_shared_capture(a_id, a_env).unwrap();
    eval.finish_shared_capture(a_id, a_env, guard);
    assert!(matches!(
        shared.entries.get(&a_id).as_deref(),
        Some(SharedEntry::Impure)
    ));

    // Diamond dependency (top -> a, b; both -> base) deduplicates base in collect_dep_refs.
    let shared = make_shared(FxHashMap::from_iter([
        (a_id, vec![base_id]),
        (b_id, vec![base_id]),
        (top_id, vec![a_id, b_id]),
    ]));
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    for id in [base_id, a_id, b_id, top_id] {
        let env = eval.environment_scope(Scope::default());
        eval.modules.insert(id, env);
        let guard = eval.begin_shared_capture(id, env).unwrap();
        eval.finish_shared_capture(id, env, guard);
    }
    assert!(matches!(
        shared.entries.get(&top_id).as_deref(),
        Some(SharedEntry::Pure(cached)) if cached.dep_refs.len() == 3
    ));
}
