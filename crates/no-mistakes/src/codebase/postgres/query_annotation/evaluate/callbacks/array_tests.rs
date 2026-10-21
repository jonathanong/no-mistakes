use super::super::super::{Evaluator, File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn with_arrays(
    test: impl FnOnce(
        &mut Evaluator<'_, fn(&str, &std::path::Path) -> Option<PathBuf>>,
        &std::path::Path,
        &str,
        usize,
    ),
) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-profile/arrays.cjs",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (mut facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    facts.precise_records = true;
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let mut evaluator = Evaluator {
        parallel_arms: None,
        files: &files,
        resolve: (|_: &str, _: &std::path::Path| -> Option<PathBuf> { None })
            as fn(&str, &std::path::Path) -> Option<PathBuf>,
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
    let root = evaluator.module_environment(&path);
    let call = query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name("createArrays".into())),
        args: Vec::new(),
        start: 0,
        end: 0,
    };
    let arrays = evaluator.expr(&call, &path, &root, 16, false).exposed();
    let nested = evaluator.index(arrays.clone(), 0);
    let child = evaluator.index(nested, 0);
    let Value::Function(_, _, captured) = evaluator.index(child, 0) else {
        panic!("saved shared callback");
    };
    assert!(
        matches!(evaluator.scopes[captured]["nested"], Value::Array(_)),
        "fixture uses modeled arrays, not aggregate fallback"
    );
    assert!(
        evaluator.events.is_empty(),
        "callback creation does not execute its body"
    );
    test(&mut evaluator, &path, &source, captured);
}

#[test]
fn array_callbacks_visit_live_aliases_and_cycles_once() {
    with_arrays(|evaluator, path, source, env| {
        let roots =
            ["nested", "removed", "recreated"].map(|name| evaluator.scopes[env][name].clone());
        let Value::Array(deleted) = roots[1] else {
            panic!("saved array identity");
        };
        // Protect the shared arena tombstone filter without widening JS delete modeling.
        evaluator
            .definite_deleted_argument_slots
            .insert((deleted, 0));
        evaluator.active_callback_functions = Some(Default::default());
        evaluator.active_callback_executions = Some(Default::default());
        let mut state = super::super::CallbackState::default();
        evaluator.callback_values(&roots, 8, &mut state);
        assert_eq!(
            state.functions.len(),
            1,
            "only the shared live callback participates"
        );
        assert_eq!(
            state.objects.len(),
            4,
            "each reachable array identity is visited once"
        );
        let start = source.find("database.query(statement)").unwrap() as u32;
        assert_eq!(
            evaluator.events[&(
                path.to_path_buf(),
                (start, start + "database.query(statement)".len() as u32)
            )]
                .len(),
            1
        );
        assert_eq!(
            evaluator.events.len(),
            1,
            "deleted and overwritten callbacks stay disconnected"
        );
    });
}

#[test]
fn array_callback_depth_zero_revokes_captured_proofs_without_executing() {
    with_arrays(|evaluator, _, _, env| {
        let root = evaluator.scopes[env]["nested"].clone();
        assert!(matches!(
            evaluator.scopes[env]["statement"],
            Value::Prefix(_, true, _)
        ));
        let frames = evaluator.scopes.len();
        evaluator.opaque_callbacks(&[root], 0);
        assert!(matches!(evaluator.scopes[env]["statement"], Value::Unknown));
        assert!(evaluator.events.is_empty());
        assert_eq!(evaluator.scopes.len(), frames);
        assert!(evaluator.active_callback_functions.is_none());
    });
}

#[test]
fn array_callbacks_reach_newly_installed_live_slots() {
    with_arrays(|evaluator, path, source, env| {
        let root = evaluator.scopes[env]["installer"].clone();
        evaluator.opaque_callbacks(&[root], 8);
        let start = source.find("database.query(\"SELECT installed\")").unwrap() as u32;
        let events = &evaluator.events[&(
            path.to_path_buf(),
            (
                start,
                start + "database.query(\"SELECT installed\")".len() as u32,
            ),
        )];
        assert!(
            !events.is_empty(),
            "slot installed by an opaque callback is reached"
        );
        assert!(
            events.len() <= 8,
            "revisits spend the existing callback budget"
        );
        assert!(evaluator.active_callback_functions.is_none());
    });
}
