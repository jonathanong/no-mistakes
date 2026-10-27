use super::super::super::{ArgumentObjects, Evaluator, Scope, Value};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::postgres::query_annotation::evaluate::captures::CapturedOrigins;
use crate::codebase::postgres::query_annotation::evaluate::File as QueryFile;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::{self, Expr};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

#[test]
fn alternatives_do_not_clone_an_unchanged_prefix_capture_during_resolver_work() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-scalar-alternatives/root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@data-stores/psql", &[]);
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    facts.precise_records = true;
    let ts = TsFileFacts {
        imports: vec![ExtractedImport {
            specifier: "./virtual.mts".into(),
            kind: ImportKind::Dynamic,
            line: 1,
            function_scope: None,
            function_scope_id: None,
            side_effect_only: false,
            re_export: false,
            runtime_reachable: false,
            computed: false,
        }],
        ..Default::default()
    };
    let files = FxHashMap::from_iter([(
        path.clone(),
        QueryFile {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(ts))),
            executors: Default::default(),
        },
    )]);
    let origins: CapturedOrigins = FxHashMap::from_iter([("captured".into(), 0)]).into();
    let probe = origins.strong_count_probe();
    let observed = Rc::new(Cell::new(0));
    let resolver_observed = Rc::clone(&observed);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: move |_: &str, _: &Path| -> Option<PathBuf> {
            resolver_observed.set(probe());
            None
        },
        events: Default::default(),
        scopes: vec![Scope::default(), Scope::default()],
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: FxHashMap::from_iter([(1, origins)]),
        captured_binding_readers: FxHashMap::from_iter([(
            0,
            FxHashMap::from_iter([("captured".into(), FxHashSet::from_iter([1]))]),
        )]),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: ArgumentObjects::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.scopes[0].insert("captured".into(), Value::Unknown);
    evaluator.scopes[1].insert("captured".into(), Value::Unknown);
    let expected_captures = evaluator
        .captured_bindings
        .iter()
        .map(|(frame, origins)| {
            (
                *frame,
                origins
                    .iter()
                    .map(|(name, origin)| (name.clone(), *origin))
                    .collect::<FxHashMap<_, _>>()
                    .into(),
            )
        })
        .collect::<FxHashMap<_, _>>();
    let expected_readers = evaluator.captured_binding_readers.clone();
    let expected_scopes = evaluator.scopes.clone();
    let arms = vec![Expr::Await(Box::new(Expr::ModuleLoad(
        "./virtual.mts".into(),
    )))];

    let result = evaluator.expr(&Expr::Alternatives(arms), &path, &0, 16, true);

    assert!(matches!(result, Value::Unknown));
    assert_eq!(
        observed.get(),
        1,
        "resolver runs with only evaluator ownership"
    );
    assert_eq!(evaluator.captured_bindings, expected_captures);
    assert_eq!(evaluator.captured_binding_readers, expected_readers);
    assert!(evaluator.scopes == expected_scopes);
    assert!(evaluator.modules.is_empty());
    assert!(evaluator.active_module_initials.is_empty());
}
