use super::RunSnapshot;
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::project::evaluator;
use crate::codebase::postgres::query_annotation::{Expr, Function, QueryAnnotationFileFacts, Step};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn promote_reused_handles_foreign_env_and_static_dep_snapshots() {
    let dep = PathBuf::from("dep.mts");
    let unloaded = PathBuf::from("unloaded.mts");
    let external = PathBuf::from("external.mts");
    let target = PathBuf::from("target.mts");
    let dep_facts = QueryAnnotationFileFacts {
        roots: vec![Step::Bind("k".into(), Expr::Number(1))],
        ..Default::default()
    };
    let target_facts = QueryAnnotationFileFacts {
        roots: vec![Step::Bind("v".into(), Expr::Text("ok".into()))],
        ..Default::default()
    };
    let make_import =
        |specifier: &str, kind: ImportKind, function_scope: Option<&str>| ExtractedImport {
            specifier: specifier.into(),
            kind,
            line: 1,
            function_scope: function_scope.map(str::to_string),
            function_scope_id: None,
            side_effect_only: false,
            re_export: false,
            runtime_reachable: false,
            computed: false,
        };
    let target_ts = TsFileFacts {
        imports: vec![
            make_import("./type.mts", ImportKind::Type, None),
            make_import("./scoped.mts", ImportKind::Static, Some("fn")),
            make_import("./missing.mts", ImportKind::Static, None),
            make_import("./external.mts", ImportKind::Static, None),
            make_import("./dep.mts", ImportKind::Static, None),
            make_import("./unloaded.mts", ImportKind::Static, None),
        ],
        ..Default::default()
    };
    let mut files = FxHashMap::default();
    files.insert(
        dep.clone(),
        File {
            facts: &dep_facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
        },
    );
    files.insert(
        unloaded.clone(),
        File {
            facts: &dep_facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
        },
    );
    files.insert(
        target.clone(),
        File {
            facts: &target_facts,
            module: Arc::new(Module::new(Arc::new(target_ts))),
            executors: Default::default(),
        },
    );
    let resolve_map = FxHashMap::from_iter([
        ("./dep.mts", dep.clone()),
        ("./unloaded.mts", unloaded.clone()),
        ("./external.mts", external),
    ]);
    let mut evaluator = evaluator::new(
        &files,
        |specifier: &str, _: &std::path::Path| resolve_map.get(specifier).cloned(),
        false,
    );
    let dep_id = evaluator.module_ids().id(&dep);
    let target_id = evaluator.module_ids().id(&target);
    evaluator.module_environment(&dep);
    let mut snapshot = RunSnapshot::capture(&evaluator);
    let mut unpromotable = FxHashSet::default();

    // Unloaded static dep defers capture_on_init without marking unpromotable.
    evaluator.module_environment(&target);
    snapshot.promote_reused(&mut evaluator, &mut unpromotable, 2);
    assert!(!snapshot.modules.capture_on_init.contains_key(&target_id));
    assert!(!unpromotable.contains(&target_id));

    // Once all static deps are in snapshot, target schedules capture_on_init and verifies deps.
    snapshot.restore(&mut evaluator);
    evaluator.module_environment(&unloaded);
    let mut snapshot = RunSnapshot::capture(&evaluator);
    evaluator.module_environment(&target);
    snapshot.promote_reused(&mut evaluator, &mut unpromotable, 2);
    assert_eq!(snapshot.modules.capture_on_init[&target_id].len(), 2);
    snapshot.restore(&mut evaluator);
    let dep_env = evaluator.modules[&dep_id];
    let rebuilt: FxHashMap<String, Value> = evaluator.scopes[dep_env]
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    evaluator.scopes[dep_env] = Scope::from(rebuilt);
    evaluator.module_environment(&target);
    snapshot.promote_reused(&mut evaluator, &mut unpromotable, 2);
    assert!(snapshot.modules.contains_key(&target_id));

    // Foreign-env function in captured_inits is rejected by try_install_captured.
    let foreign_fn = Value::Function(
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
        999,
    );
    snapshot.modules.remove(&target_id);
    evaluator.modules.captured_inits.insert(
        target_id,
        Some((
            99,
            Scope::from(FxHashMap::from_iter([("f".into(), foreign_fn)])),
        )),
    );
    evaluator.modules.insert(target_id, 99);
    snapshot.promote_reused(&mut evaluator, &mut unpromotable, 1);
    assert!(unpromotable.contains(&target_id));
}
