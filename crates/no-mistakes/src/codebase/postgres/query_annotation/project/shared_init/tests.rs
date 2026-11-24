use super::{candidates, executor_scan};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::postgres::query_annotation::evaluate::module_cache::shared::{
    CachedModule, SharedInit,
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
    let r1 = PathBuf::from("r1.mts");
    let r2 = PathBuf::from("r2.mts");
    let a = PathBuf::from("a.mts");
    let b = PathBuf::from("b.mts");
    let ids = Arc::new(ModuleIds::new([
        r1.clone(),
        r2.clone(),
        a.clone(),
        b.clone(),
    ]));
    let precise = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let mut files = FxHashMap::default();
    files.insert(r1.clone(), make_file(&precise, vec!["./a.mts"], &ids));
    files.insert(r2.clone(), make_file(&precise, vec!["./a.mts"], &ids));
    files.insert(a.clone(), make_file(&precise, vec!["./b.mts"], &ids));
    files.insert(b.clone(), make_file(&precise, vec!["./a.mts"], &ids));
    let resolve = |specifier: &str, _: &std::path::Path| match specifier {
        "./a.mts" => Some(a.clone()),
        "./b.mts" => Some(b.clone()),
        _ => None,
    };
    let graph = candidates::discover(&files, &[r1, r2], &ids, &resolve);
    assert!(graph.cyclic.contains(&ids.id(&a)));
    assert!(graph.cyclic.contains(&ids.id(&b)));
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
    let mut modules = FxHashMap::default();
    modules.insert(
        ids.id(&target),
        Arc::new(CachedModule {
            deps: vec![ids.id(&dep)],
            dep_refs: vec![(ids.id(&dep), 0)],
            dep_scopes: vec![dep_scope],
            own_scopes: vec![Scope::default()],
        }),
    );
    let shared = Arc::new(SharedInit {
        untrusted_array_intrinsics: false,
        modules,
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
    let resolve = |specifier: &str, _: &std::path::Path| match specifier {
        "./a.mts" => Some(a.clone()),
        "./b.mts" => Some(b.clone()),
        _ => None,
    };
    let status = super::build(&files, &[r1, r2], &resolve, false, &empty_failures, None);
    assert!(matches!(
        status,
        super::SharedInitStatus::Enabled {
            cached: 0,
            misses: 2,
            ..
        }
    ));
    let quiet = crate::diagnostics::InvocationObserver::new(false);
    quiet.record_note("ignored".into());
    super::record(&quiet, 0, 2, &status);
}
