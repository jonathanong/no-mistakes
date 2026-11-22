use super::super::{temporaries::TransientRoot, Environment, Evaluator, File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

type Resolve = fn(&str, &Path) -> Option<PathBuf>;

fn with_callback_boundary(test: impl FnOnce(&mut Evaluator<'_, Resolve>, Environment)) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-returned-callbacks/src/boundary.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let mut evaluator: Evaluator<'_, Resolve> = Evaluator {
        files: &files,
        resolve: |_, _| None,
        events: Default::default(),
        event_log: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 0,
        builder_tags: Default::default(),
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
    let env = evaluator.module_environment(&path);
    test(&mut evaluator, env);
}

#[test]
fn callback_depth_limit_invalidates_captures_without_executing_the_body() {
    with_callback_boundary(|evaluator, env| {
        let callback = evaluator.scopes[env]["callback"].clone();
        let Some(Value::Prefix(_, _, Some(id))) = evaluator.scopes[env].get("statement") else {
            panic!("fixture builder");
        };
        let id = *id;
        let frame_count = evaluator.scopes.len();
        assert!(evaluator.events.is_empty());
        let nested = Value::Aggregate(vec![callback].into());
        evaluator.opaque_callbacks(&[nested], 0);
        assert!(matches!(
            evaluator.scopes[env].get("statement"),
            Some(Value::Unknown)
        ));
        assert!(evaluator.invalidated_builders.contains(&id));
        assert_eq!(evaluator.scopes.len(), frame_count);
        assert!(evaluator.events.is_empty());
        assert!(evaluator.active_callback_functions.is_none());
    });
}

#[test]
fn builder_only_aggregate_retains_the_transient_builder() {
    with_callback_boundary(|evaluator, env| {
        let statement = evaluator.scopes[env]
            .remove("statement")
            .expect("saved builder");
        let Value::Prefix(_, _, Some(id)) = &statement else {
            panic!("saved builder identity");
        };
        let id = *id;
        let aggregate = Value::Aggregate(vec![statement].into());
        assert!(evaluator.transient_roots.is_empty());
        evaluator.callback_values(&[aggregate], 8, &mut super::CallbackState::default());
        assert!(evaluator
            .transient_roots
            .iter()
            .any(|root| matches!(root, TransientRoot::Builder(root_id) if *root_id == id)));
    });
}

#[path = "array_tests.rs"]
mod array_tests;
