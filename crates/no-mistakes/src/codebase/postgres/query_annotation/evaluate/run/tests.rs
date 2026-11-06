use super::super::{Evaluator, File};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[test]
fn run_prunes_dead_argument_objects_before_speculative_snapshots() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-arena-pruning/many.cjs",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
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
    };

    evaluator.run(&path);

    assert!(
        evaluator.argument_objects.len() <= 1,
        "dead argument objects must not accumulate in run snapshots"
    );
}

#[test]
fn run_drops_dead_frames_before_later_imported_module_initialization() {
    use crate::codebase::dependencies::extract::{
        ExportedBinding, ImportedBinding, ImportedBindingKind,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-arena-pruning/import-order/root.mts",
    );
    let later = root.with_file_name("later.mts");
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let collect = |path: &PathBuf| {
        let source = std::fs::read_to_string(path).unwrap();
        crate::ast::with_program(path, &source, |program, _| {
            query_annotation::collect(program, &source, &options)
        })
        .unwrap()
    };
    let root_facts = collect(&root);
    let later_facts = collect(&later);
    let root_ts = TsFileFacts {
        imported_bindings: vec![ImportedBinding {
            specifier: "./later.mts".into(),
            local: "initialize".into(),
            imported: "initialize".into(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        }],
        ..Default::default()
    };
    let later_ts = TsFileFacts {
        exported_bindings: vec![ExportedBinding {
            specifier: None,
            local: "initialize".into(),
            exported: "initialize".into(),
        }],
        ..Default::default()
    };
    let files = FxHashMap::from_iter([
        (
            root.clone(),
            File {
                facts: &root_facts,
                module: std::sync::Arc::new(Module::new(root_ts.clone().into())),
                executors: Default::default(),
            },
        ),
        (
            later.clone(),
            File {
                facts: &later_facts,
                module: std::sync::Arc::new(Module::new(later_ts.clone().into())),
                executors: Default::default(),
            },
        ),
    ]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |specifier: &str, from: &Path| {
            (specifier == "./later.mts" && from == root.as_path()).then(|| later.clone())
        },
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
    };

    evaluator.module_environment(&root);
    assert_eq!(evaluator.modules.len(), 2, "both modules initialize");
    assert!(
        evaluator.argument_objects.len() > 16,
        "the fixture creates many dead calls before the imported module gets an environment"
    );

    evaluator.prune_snapshot_state();

    assert_eq!(evaluator.modules.len(), 2);
    assert!(evaluator
        .modules
        .values()
        .all(|env| *env < evaluator.scopes.len()));
    assert!(
        evaluator.argument_objects.len() <= 1,
        "dead calls before the later module environment must not become permanent roots"
    );
    assert!(
        evaluator.scopes.len() <= 4,
        "only module/captured frames remain"
    );
}

#[test]
fn missing_prepared_eager_and_runtime_modules_remain_observed_on_cache_hits() {
    use super::super::module_cache::source_dependencies::SourceDependencies;
    use crate::codebase::postgres::query_annotation::{Expr, QueryAnnotationFileFacts};
    use crate::codebase::ts_source::facts::{
        collect_file_facts_from_program, TsFactContext, TsFactPlan,
    };
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-source-prerequisites");
    let path = root.join("cached.mts");
    let broken = root.join("broken.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let ts = crate::ast::with_program(&path, &source, |program, source| {
        collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            source,
            program,
            None,
            None,
        )
    })
    .unwrap();
    let failed_source = std::fs::read_to_string(&broken).unwrap();
    assert!(crate::ast::with_program(&broken, &failed_source, |_, _| ()).is_err());
    let facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |specifier: &str, from: &Path| {
            (specifier == "./broken.mjs").then(|| from.with_file_name("broken.mts"))
        },
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
    };
    evaluator.modules.sources =
        SourceDependencies::new(std::sync::Arc::new([broken.clone()].into_iter().collect()));
    let environment = evaluator.module_environment(&path);
    assert_eq!(
        evaluator.modules.sources.failures(),
        [broken.clone()].into_iter().collect()
    );
    assert_eq!(evaluator.module_environment(&path), environment);
    assert_eq!(
        evaluator.modules.sources.failures(),
        [broken.clone()].into_iter().collect()
    );
    evaluator.await_dynamic_import("./broken.mjs", &path);
    assert_eq!(
        evaluator.modules.sources.failures(),
        [broken].into_iter().collect()
    );
    assert!(matches!(
        evaluator.await_expr(
            &Expr::ModuleLoad("./broken.mjs".into()),
            &path,
            &environment,
            16,
            true
        ),
        super::super::Value::Unknown
    ));
}
