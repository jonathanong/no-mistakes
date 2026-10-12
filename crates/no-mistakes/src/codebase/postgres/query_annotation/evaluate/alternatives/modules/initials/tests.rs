use super::*;
use crate::codebase::postgres::query_annotation::evaluate::File;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
#[test]
fn pristine_module_snapshots_restore_and_remap_sloppy_argument_callback_state() {
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
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
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
    evaluator
        .active_module_initials
        .push(super::Initials::default());
    let root = evaluator.module_environment(&path);
    evaluator.record_alternative_module_initial(&path, root);
    let mut initial = evaluator.active_module_initials.pop().unwrap();
    assert!(!initial.objects.is_empty());
    assert!(
        initial
            .objects
            .keys()
            .any(|id| !initial.extras.contains_key(id)),
        "dense-only argument objects remain live"
    );
    assert!(
        !initial.updates.is_empty(),
        "reachable builder append updates are recorded"
    );
    assert!(initial
        .extras
        .values()
        .any(|slots| slots.contains_key(&1000)));
    assert!(!initial.mapped.is_empty());
    assert!(!initial.captured.is_empty());
    assert!(initial
        .fresh
        .values()
        .any(|names| names.contains("parameter")));
    assert!(!initial.invalidated.is_empty());
    assert!(!initial.deleted.is_empty());
    assert!(!initial.definite.is_empty());
    assert!(!initial.disconnected.is_empty());
    let base = evaluator.scopes.len();
    let original_readers = evaluator.captured_binding_readers.clone();
    assert!(!original_readers.is_empty());
    let offset = base + 5;
    let indices = initial
        .roots()
        .map(|env| (env, env + offset))
        .collect::<FxHashMap<_, _>>();
    initial.remap(&indices);
    assert!(initial.roots().all(|env| env >= offset));
    assert!(initial
        .captured
        .values()
        .flat_map(|names| names.values())
        .all(|env| *env >= offset));
    assert!(initial
        .extras
        .values()
        .flat_map(|slots| slots.values())
        .any(|value| matches!(value, Value::Function(_,_,env) if *env >= offset)));
    evaluator
        .scopes
        .resize(evaluator.scopes.len() + offset, Default::default());
    evaluator.argument_objects.clear();
    evaluator.argument_extra_slots.clear();
    evaluator.mapped_arguments.clear();
    evaluator.fresh_mapped_parameters.clear();
    evaluator.invalidated_builders.clear();
    evaluator.builder_updates.clear();
    evaluator.deleted_argument_slots.clear();
    evaluator.definite_deleted_argument_slots.clear();
    evaluator.disconnected_argument_slots.clear();
    evaluator.active_module_initials.push(initial);
    evaluator.restore_alternative_modules();
    evaluator.add_alternative_captured_readers(base);
    let readers = evaluator.captured_binding_readers.clone();
    assert!(readers.keys().any(|origin| *origin >= offset));
    // Removing the restored arm empties its private origins without losing
    // readers on the original frames. Reinstalling must recover the exact index.
    evaluator.remove_alternative_captured_readers(base);
    assert_eq!(evaluator.captured_binding_readers, original_readers);
    evaluator.add_alternative_captured_readers(base);
    assert_eq!(evaluator.captured_binding_readers, readers);
    evaluator.rebuild_captured_readers();
    assert_eq!(evaluator.captured_binding_readers, readers);
    // If every captured frame belongs to the arm, restoring it can clear
    // the whole reverse index, including the remapped module-initial readers.
    evaluator.remove_alternative_captured_readers(0);
    assert!(evaluator.captured_binding_readers.is_empty());
    evaluator.add_alternative_captured_readers(0);
    assert_eq!(evaluator.captured_binding_readers, readers);
    assert!(evaluator
        .fresh_mapped_parameters
        .values()
        .any(|names| names.contains("parameter")));
    assert!(evaluator
        .argument_extra_slots
        .values()
        .any(|slots| slots.contains_key(&1000)));
    assert!(!evaluator.disconnected_argument_slots.is_empty());
    assert!(
        !evaluator.builder_updates.is_empty(),
        "pristine append state is restored"
    );
    assert!(evaluator
        .argument_objects
        .keys()
        .any(|id| !evaluator.argument_extra_slots.contains_key(id)));
    evaluator.active_module_initials.pop();
    assert!(evaluator.active_module_initials.is_empty());
}
