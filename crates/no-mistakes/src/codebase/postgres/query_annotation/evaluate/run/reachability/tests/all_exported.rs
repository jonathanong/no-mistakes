// Multi-global modules whose local exports cover every global take All.
// Visit counts for that shortcut go through Evaluator::run, which applies the
// specifier filter itself. A single exported seed still walks the cycle.
use super::super::super::super::{Evaluator, File};
use super::super::{visits, ExportReachability};
use super::reference::exported_reaches;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn every_global_export_takes_all_without_traversal() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/postgres-facts/executed-source/export-reachability-all-exported-cycle.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let (mut facts, exported_bindings) = crate::ast::with_program(&path, &source, |program, _| {
        let facts = query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        let exported_bindings =
            extract_import_facts_from_program_with_source(program, &source).exported_bindings;
        (facts, exported_bindings)
    })
    .unwrap();
    // The singleton shortcut returns All for one global. This fixture must not.
    assert!(facts.globals.len() >= 2);
    assert_eq!(
        facts.globals.keys().map(String::as_str).collect::<Vec<_>>(),
        ["left", "middle", "right"]
    );
    assert!(
        returns_name(&facts.globals["left"], "right"),
        "{:?}",
        facts.globals["left"]
    );
    assert!(
        returns_name(&facts.globals["right"], "left"),
        "{:?}",
        facts.globals["right"]
    );
    assert!(
        returns_name(&facts.globals["right"], "middle"),
        "{:?}",
        facts.globals["right"]
    );
    assert!(
        matches!(&facts.globals["middle"], Expr::Name(name) if name == "left"),
        "{:?}",
        facts.globals["middle"]
    );
    // Oracle inputs only. `run` receives every binding, including the re-export.
    assert!(exported_bindings.iter().any(|binding| {
        binding.specifier.is_some() && !facts.globals.contains_key(&binding.local)
    }));
    let local_exports: Vec<String> = exported_bindings
        .iter()
        .filter(|binding| binding.specifier.is_none())
        .map(|binding| binding.local.clone())
        .collect();
    assert!(facts
        .globals
        .keys()
        .all(|name| local_exports.iter().any(|exported| exported == name)));

    visits::reset();
    let one = super::super::export_reachable_globals(&facts.globals, ["left"]);
    assert!(matches!(one, ExportReachability::Selected(_)));
    assert!(facts.globals.keys().all(|name| one.contains(name)));
    assert_eq!(visits::count(), facts.globals.len());

    visits::reset();
    let reachable = super::super::export_reachable_globals(
        &facts.globals,
        local_exports.iter().map(String::as_str),
    );
    assert!(matches!(reachable, ExportReachability::All));
    assert_eq!(visits::count(), 0);
    for target in facts.globals.keys().map(String::as_str) {
        assert_eq!(
            reachable.contains(target),
            local_exports.iter().any(|exported| {
                exported_reaches(&facts.globals, exported, target, &mut BTreeSet::new())
            }),
            "all local exports -> {target}"
        );
    }

    // One local export is the positive control. Filtering after the move would
    // not be able to rebuild this seed.
    let limited: Vec<_> = exported_bindings
        .iter()
        .filter(|binding| binding.specifier.is_none() && binding.local == "left")
        .cloned()
        .collect();
    assert_eq!(limited.len(), 1, "the positive control seeds only left");
    facts.precise_records = true;
    let ts = TsFileFacts {
        exported_bindings,
        ..TsFileFacts::default()
    };
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(ts))),
            executors: Default::default(),
        },
    )]);
    let mut all_evaluator = evaluator(&files);
    visits::reset();
    all_evaluator.run(&path);
    assert_eq!(
        visits::count(),
        0,
        "Evaluator::run must take All instead of walking each export"
    );
    // The zero above would also pass if run returned before the walk. The same
    // evaluator with only `left` exported has to visit.
    let limited_ts = TsFileFacts {
        exported_bindings: limited,
        ..TsFileFacts::default()
    };
    let limited_files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(limited_ts))),
            executors: Default::default(),
        },
    )]);
    let mut limited_evaluator = evaluator(&limited_files);
    visits::reset();
    limited_evaluator.run(&path);
    assert!(
        visits::count() > 0,
        "limiting exports to left must walk; visits stay 0 if run returns early"
    );
}

fn returns_name(value: &Expr, name: &str) -> bool {
    let Expr::Function(function) = value else {
        return false;
    };
    function.body.iter().any(|step| match step {
        Step::Return(Expr::Name(returned)) => returned == name,
        Step::Return(Expr::Array(items)) => items
            .iter()
            .any(|item| matches!(item, Expr::Name(returned) if returned == name)),
        _ => false,
    })
}

fn evaluator<'a>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
) -> Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>> {
    Evaluator {
        files,
        resolve: |_specifier, _from| None,
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    }
}
