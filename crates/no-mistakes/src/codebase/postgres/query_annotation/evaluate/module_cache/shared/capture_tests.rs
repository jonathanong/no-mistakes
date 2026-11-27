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

fn empty_files<'a>(
    paths: &[PathBuf],
    facts: &'a QueryAnnotationFileFacts,
) -> (Arc<ModuleIds>, FxHashMap<PathBuf, File<'a>>) {
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let module =
        Arc::new(Module::new(Arc::new(TsFileFacts::default())).with_module_ids(Arc::clone(&ids)));
    let mut files = FxHashMap::default();
    for path in paths {
        files.insert(
            path.clone(),
            File {
                facts,
                module: Arc::clone(&module),
                executors: FxHashSet::default(),
            },
        );
    }
    (ids, files)
}

#[test]
fn is_clean_shared_init_rejects_every_mutated_state_condition() {
    let target = PathBuf::from("target.mts");
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(std::slice::from_ref(&target), &facts);
    let target_id = ids.id(&target);
    let resolve = |_: &str, _: &std::path::Path| None;

    for case in 0..7 {
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
            _ => eval.mark_untrusted_array_intrinsics(),
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
fn begin_shared_capture_skips_when_untrusted_mode_differs() {
    let target = PathBuf::from("target.mts");
    let facts = QueryAnnotationFileFacts::default();
    let (ids, files) = empty_files(std::slice::from_ref(&target), &facts);
    let target_id = ids.id(&target);
    let resolve = |_: &str, _: &std::path::Path| None;
    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [target_id].into_iter().collect(),
        direct_deps: FxHashMap::default(),
        entries: dashmap::DashMap::new(),
    });
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(Arc::clone(&shared));
    let env = eval.environment_scope(Scope::default());
    assert!(eval.begin_shared_capture(target_id, env).is_none());
    assert!(shared.entries.get(&target_id).is_none());
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
