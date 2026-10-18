use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;

#[test]
fn pristine_module_snapshots_keep_live_frame_and_argument_roots() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-module-snapshot/initial.cjs",
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
            module: std::sync::Arc::new(Module::new(ts.into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        transactions: Default::default(),
        scopes: Default::default(),
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
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    };

    let base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(base);
    let transaction = evaluator.begin_transaction();
    let root = evaluator.module_environment(&path);
    let initial = evaluator.active_module_initials.last().unwrap();
    let roots = initial.roots().collect::<FxHashSet<_>>();
    let arguments = initial.arguments().collect::<FxHashSet<_>>();

    assert!(
        roots.contains(&root),
        "the module frame is promoted as a root"
    );
    assert!(
        !arguments.is_empty(),
        "callbacks retain the module's argument objects"
    );
    assert!(arguments.iter().any(|id| {
        evaluator.argument_objects.get(id).is_some_and(|slots| {
            slots.iter().any(|value| matches!(value, Value::Prefix(text, _, _) if text.contains("dense-only callback")))
        })
    }));

    evaluator.finish_transaction(transaction);
    assert_eq!(evaluator.modules.get(&path), Some(&root));
    assert!(roots.iter().all(|env| evaluator.scopes.get(*env).is_some()));
    assert!(arguments
        .iter()
        .all(|id| evaluator.argument_objects.contains_key(id)));
    evaluator.active_module_initials.pop();
    assert!(evaluator.active_module_initials.is_empty());
}
