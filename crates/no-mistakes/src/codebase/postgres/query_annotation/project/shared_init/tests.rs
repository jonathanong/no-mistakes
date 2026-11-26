use super::{candidates, executor_scan};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::postgres::query_annotation::evaluate::module_cache::shared::{
    CachedModule, SharedEntry, SharedInit,
};
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope, Value};
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::{Expr, Function, QueryAnnotationFileFacts, Step};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn make_file<'a>(
    facts: &'a QueryAnnotationFileFacts,
    imports: Vec<&str>,
    ids: &Arc<ModuleIds>,
) -> File<'a> {
    let ts = Arc::new(TsFileFacts {
        imports: imports
            .into_iter()
            .map(|specifier| ExtractedImport {
                specifier: specifier.to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: None,
                function_scope_id: None,
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
                computed: false,
            })
            .collect(),
        ..Default::default()
    });
    File {
        facts,
        module: Arc::new(Module::new(ts).with_module_ids(Arc::clone(ids))),
        executors: FxHashSet::default(),
    }
}

#[test]
fn discover_handles_precise_records_and_cycles() {
    let [r1, r2, a, b, c] = ["r1.mts", "r2.mts", "a.mts", "b.mts", "c.mts"].map(PathBuf::from);
    let ids = Arc::new(ModuleIds::new([&r1, &r2, &a, &b, &c].into_iter().cloned()));
    let precise = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let mut files = FxHashMap::default();
    files.insert(r1.clone(), make_file(&precise, vec!["./c.mts"], &ids));
    files.insert(r2.clone(), make_file(&precise, vec!["./c.mts"], &ids));
    files.insert(a.clone(), make_file(&precise, vec!["./b.mts"], &ids));
    files.insert(b.clone(), make_file(&precise, vec!["./a.mts"], &ids));
    files.insert(c.clone(), make_file(&precise, vec!["./a.mts"], &ids));
    let targets = FxHashMap::from_iter([
        ("./a.mts", a.clone()),
        ("./b.mts", b.clone()),
        ("./c.mts", c.clone()),
    ]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    let graph = candidates::discover(&files, &[r1, r2], &ids, &resolve);
    assert!(graph.cyclic.contains(&ids.id(&a)));
    assert!(graph.cyclic.contains(&ids.id(&b)));
    assert!(graph.cyclic.contains(&ids.id(&c)));
}

#[test]
fn purity_check_accepts_pure_local_scalar_call_and_rejects_impure_constructs() {
    let path = PathBuf::from("m.mts");
    let ids = Arc::new(ModuleIds::new([path.clone()]));
    let resolve = |_: &str, _: &std::path::Path| None;
    let mut globals = std::collections::BTreeMap::new();
    globals.insert(
        "pure_fn".to_string(),
        Expr::Function(Arc::new(Function {
            start: 0,
            params: vec!["x".into()],
            body: vec![
                Step::Reserve(vec!["y".into()]),
                Step::Bind(
                    "y".into(),
                    Expr::Template(vec![Expr::Name("x".into()), Expr::Text("!".into())]),
                ),
                Step::Return(Expr::Binary(
                    "+".into(),
                    Box::new(Expr::Name("y".into())),
                    Box::new(Expr::Text(".".into())),
                )),
            ],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
            append_callback: None,
        })),
    );
    globals.insert(
        "stateful".to_string(),
        Expr::Function(Arc::new(Function {
            start: 10,
            params: Vec::new(),
            body: vec![Step::Effect(Expr::Name("arguments".into()))],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
            append_callback: None,
        })),
    );
    let pure_facts = QueryAnnotationFileFacts {
        globals: globals.clone(),
        roots: vec![Step::Bind(
            "label".into(),
            Expr::Call {
                callee: Box::new(Expr::Name("pure_fn".into())),
                args: vec![Expr::Text("ok".into())],
                start: 0,
                end: 1,
            },
        )],
        ..Default::default()
    };
    let pure_file = make_file(&pure_facts, Vec::new(), &ids);
    assert!(executor_scan::is_statically_pure_top_level(
        &pure_file, &path, &resolve
    ));

    let unresolved_facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let unresolved_file = make_file(&unresolved_facts, vec!["./missing.mts"], &ids);
    assert!(!executor_scan::is_statically_pure_top_level(
        &unresolved_file,
        &path,
        &resolve
    ));

    let impure_facts = QueryAnnotationFileFacts {
        globals,
        roots: vec![Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("stateful".into())),
            args: Vec::new(),
            start: 2,
            end: 3,
        })],
        ..Default::default()
    };
    let impure_file = make_file(&impure_facts, Vec::new(), &ids);
    assert!(!executor_scan::is_statically_pure_top_level(
        &impure_file,
        &path,
        &resolve
    ));

    let array_facts = QueryAnnotationFileFacts {
        roots: vec![Step::Bind("arr".into(), Expr::Array(vec![Expr::Number(1)]))],
        ..Default::default()
    };
    let array_file = make_file(&array_facts, Vec::new(), &ids);
    assert!(!executor_scan::is_statically_pure_top_level(
        &array_file,
        &path,
        &resolve
    ));
}

#[test]
fn try_install_shared_module_rejects_flag_mismatch_and_mutated_dep() {
    let dep = PathBuf::from("dep.mts");
    let target = PathBuf::from("target.mts");
    let ids = Arc::new(ModuleIds::new([dep.clone(), target.clone()]));
    let facts = QueryAnnotationFileFacts::default();
    let mut files = FxHashMap::default();
    files.insert(dep.clone(), make_file(&facts, Vec::new(), &ids));
    files.insert(target.clone(), make_file(&facts, Vec::new(), &ids));
    let mut dep_vals = FxHashMap::default();
    dep_vals.insert("x".into(), Value::Number(1));
    let dep_scope = Scope::from(dep_vals);
    let entries = dashmap::DashMap::new();
    entries.insert(
        ids.id(&target),
        SharedEntry::Pure(Arc::new(CachedModule {
            deps: vec![ids.id(&dep)],
            dep_refs: vec![(ids.id(&dep), 0)],
            dep_scopes: vec![dep_scope],
            own_scopes: vec![Scope::default()],
        })),
    );
    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: FxHashSet::default(),
        direct_deps: FxHashMap::default(),
        entries,
    });
    let resolve = |_: &str, _: &std::path::Path| None;
    let mut eval = evaluator::new(&files, resolve, true);
    eval.modules.shared = Some(Arc::clone(&shared));
    assert!(eval
        .try_install_shared_module(&target, ids.id(&target))
        .is_none());

    let mut eval2 = evaluator::new(&files, resolve, false);
    eval2.modules.shared = Some(shared);
    let mut mutated = FxHashMap::default();
    mutated.insert("x".into(), Value::Number(2));
    let dep_env = eval2.environment_scope(Scope::from(mutated));
    eval2.modules.insert(ids.id(&dep), dep_env);
    assert!(eval2
        .try_install_shared_module(&target, ids.id(&target))
        .is_none());
}

#[test]
fn build_handles_missing_module_ids_cyclic_candidates_and_non_verbose_notes() {
    let r1 = PathBuf::from("r1.mts");
    let r2 = PathBuf::from("r2.mts");
    let a = PathBuf::from("a.mts");
    let b = PathBuf::from("b.mts");
    let facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let mut no_ids = FxHashMap::default();
    no_ids.insert(
        r1.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: FxHashSet::default(),
        },
    );
    let empty_failures = Default::default();
    let status = super::build(
        &no_ids,
        &[r1.clone(), r2.clone()],
        &|_, _| None,
        false,
        &empty_failures,
        None,
    );
    assert!(matches!(
        status,
        super::SharedInitStatus::Disabled {
            reason: "no_shared_imports"
        }
    ));
    assert_eq!(status.seed_root(), None);

    let ids = Arc::new(ModuleIds::new([
        r1.clone(),
        r2.clone(),
        a.clone(),
        b.clone(),
    ]));
    let mut files = FxHashMap::default();
    files.insert(r1.clone(), make_file(&facts, vec!["./a.mts"], &ids));
    files.insert(r2.clone(), make_file(&facts, vec!["./a.mts"], &ids));
    files.insert(a.clone(), make_file(&facts, vec!["./b.mts"], &ids));
    files.insert(b.clone(), make_file(&facts, vec!["./a.mts"], &ids));
    let targets = FxHashMap::from_iter([("./a.mts", a.clone()), ("./b.mts", b.clone())]);
    let resolve = |specifier: &str, _: &std::path::Path| targets.get(specifier).cloned();
    let status = super::build(&files, &[r1, r2], &resolve, false, &empty_failures, None);
    assert!(matches!(
        status,
        super::SharedInitStatus::Enabled {
            candidates: 2,
            seed_root: None,
            ..
        }
    ));
    assert_eq!(status.shared().unwrap().cached_count(), 0);
    let quiet = crate::diagnostics::InvocationObserver::new(false);
    quiet.record_note("ignored".into());
    super::record(&quiet, 0, 2, &status);
}

#[test]
fn purity_check_covers_step_and_callee_edge_cases() {
    let path = PathBuf::from("edge.mts");
    let ids = Arc::new(ModuleIds::new([path.clone()]));
    let resolve = |_: &str, _: &std::path::Path| None;
    let mut globals = std::collections::BTreeMap::new();
    globals.insert(
        "async_fn".into(),
        Expr::Function(Arc::new(Function {
            start: 0,
            params: Vec::new(),
            body: Vec::new(),
            supported: true,
            asynchronous: true,
            arrow: false,
            self_name: None,
            append_callback: None,
        })),
    );
    globals.insert(
        "named_fn".into(),
        Expr::Function(Arc::new(Function {
            start: 1,
            params: Vec::new(),
            body: vec![
                Step::Var("v".into()),
                Step::Hoisted("h".into(), Expr::Text("ok".into())),
                Step::Return(Expr::Name("h".into())),
            ],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: Some("self_fn".into()),
            append_callback: None,
        })),
    );
    let pure_exprs = QueryAnnotationFileFacts {
        globals: globals.clone(),
        roots: vec![
            Step::Var("x".into()),
            Step::Bind(
                "a".into(),
                Expr::Await(Box::new(Expr::ComputedMember(
                    Box::new(Expr::Member(Box::new(Expr::Name("obj".into())), "k".into())),
                    Box::new(Expr::Template(vec![Expr::Text("p".into())])),
                ))),
            ),
            Step::Bind(
                "b".into(),
                Expr::Call {
                    callee: Box::new(Expr::Name("named_fn".into())),
                    args: vec![Expr::Number(1)],
                    start: 0,
                    end: 1,
                },
            ),
        ],
        ..Default::default()
    };
    assert!(executor_scan::is_statically_pure_top_level(
        &make_file(&pure_exprs, Vec::new(), &ids),
        &path,
        &resolve
    ));

    for bad_root in [
        Step::Unsupported,
        Step::Effect(Expr::Call {
            callee: Box::new(Expr::Member(Box::new(Expr::Name("o".into())), "m".into())),
            args: Vec::new(),
            start: 0,
            end: 1,
        }),
        Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("missing_fn".into())),
            args: Vec::new(),
            start: 0,
            end: 1,
        }),
        Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("async_fn".into())),
            args: Vec::new(),
            start: 0,
            end: 1,
        }),
        Step::Effect(Expr::Call {
            callee: Box::new(Expr::Name("named_fn".into())),
            args: vec![Expr::Array(vec![Expr::Number(1)])],
            start: 0,
            end: 1,
        }),
    ] {
        let bad = QueryAnnotationFileFacts {
            globals: globals.clone(),
            roots: vec![bad_root],
            ..Default::default()
        };
        assert!(!executor_scan::is_statically_pure_top_level(
            &make_file(&bad, Vec::new(), &ids),
            &path,
            &resolve
        ));
    }
}

#[test]
fn shared_capture_rejects_impure_dep_mutated_dep_and_foreign_env() {
    let dep = PathBuf::from("dep.mts");
    let target = PathBuf::from("target.mts");
    let ids = Arc::new(ModuleIds::new([dep.clone(), target.clone()]));
    let facts = QueryAnnotationFileFacts::default();
    let mut files = FxHashMap::default();
    files.insert(dep.clone(), make_file(&facts, Vec::new(), &ids));
    files.insert(target.clone(), make_file(&facts, Vec::new(), &ids));
    let dep_id = ids.id(&dep);
    let target_id = ids.id(&target);
    let resolve = |_: &str, _: &std::path::Path| None;

    // 1. Impure entry in SharedInit returns None on try_install_shared_module,
    //    and a dependent whose direct dep is Impure is marked Impure.
    let entries = dashmap::DashMap::new();
    entries.insert(dep_id, SharedEntry::Impure);
    let mut direct_deps = FxHashMap::default();
    direct_deps.insert(target_id, vec![dep_id]);
    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [target_id].into_iter().collect(),
        direct_deps: direct_deps.clone(),
        entries,
    });
    let mut eval = evaluator::new(&files, resolve, false);
    eval.modules.shared = Some(Arc::clone(&shared));
    assert!(eval.try_install_shared_module(&dep, dep_id).is_none());
    let env = eval.environment_scope(Scope::default());
    let guard = eval.begin_shared_capture(target_id, env).unwrap();
    eval.finish_shared_capture(target_id, env, guard);
    assert!(matches!(
        shared.entries.get(&target_id).as_deref(),
        Some(SharedEntry::Impure)
    ));
    // Already-present entry returns early without overwriting.
    assert!(eval.begin_shared_capture(target_id, env).is_none());

    // 2. Mutated dependency scope during capture marks target Impure.
    let mut expected_dep_vals = FxHashMap::default();
    expected_dep_vals.insert("x".into(), Value::Number(1));
    let entries2 = dashmap::DashMap::new();
    entries2.insert(
        dep_id,
        SharedEntry::Pure(Arc::new(CachedModule {
            deps: Vec::new(),
            dep_refs: Vec::new(),
            dep_scopes: Vec::new(),
            own_scopes: vec![Scope::from(expected_dep_vals)],
        })),
    );
    let shared2 = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [target_id].into_iter().collect(),
        direct_deps,
        entries: entries2,
    });
    let mut eval2 = evaluator::new(&files, resolve, false);
    eval2.modules.shared = Some(Arc::clone(&shared2));
    let mut mutated_dep_vals = FxHashMap::default();
    mutated_dep_vals.insert("x".into(), Value::Number(2));
    let dep_env = eval2.environment_scope(Scope::from(mutated_dep_vals));
    eval2.modules.insert(dep_id, dep_env);
    let target_env = eval2.environment_scope(Scope::default());
    let guard3 = eval2.begin_shared_capture(target_id, target_env).unwrap();
    eval2.finish_shared_capture(target_id, target_env, guard3);
    assert!(matches!(
        shared2.entries.get(&target_id).as_deref(),
        Some(SharedEntry::Impure)
    ));

    // 3. Foreign function environment in target scope marks target Impure.
    let shared3 = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        eligible: [target_id].into_iter().collect(),
        direct_deps: FxHashMap::default(),
        entries: dashmap::DashMap::new(),
    });
    let mut eval3 = evaluator::new(&files, resolve, false);
    eval3.modules.shared = Some(Arc::clone(&shared3));
    let mut foreign_vals = FxHashMap::default();
    foreign_vals.insert(
        "f".into(),
        Value::Function(
            Arc::new(Function {
                start: 0,
                params: Vec::new(),
                body: Vec::new(),
                supported: true,
                asynchronous: false,
                arrow: true,
                self_name: None,
                append_callback: None,
            }),
            Arc::new(target.clone()),
            99,
        ),
    );
    let env3 = eval3.environment_scope(Scope::from(foreign_vals));
    let guard4 = eval3.begin_shared_capture(target_id, env3).unwrap();
    eval3.finish_shared_capture(target_id, env3, guard4);
    assert!(matches!(
        shared3.entries.get(&target_id).as_deref(),
        Some(SharedEntry::Impure)
    ));
}
