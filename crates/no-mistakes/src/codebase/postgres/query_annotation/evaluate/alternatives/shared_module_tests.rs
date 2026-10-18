use super::super::{Evaluator, File};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn lazy_module_argument_ids_use_shared_definite_intersections() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-module-sibling/src/shared-mask.cjs",
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
    let root = evaluator.module_environment(&path);
    let super::super::Value::Arguments(id) = evaluator.scopes[root]["shared"] else {
        panic!("saved shared argument identity");
    };
    let shared = crate::fx::FxHashSet::from_iter([id]);
    for _mask in ["property absence", "permanent mapping disconnection"] {
        let mut common = None;
        let mut private = crate::fx::FxHashSet::default();
        super::merge::tests::legacy::definite(
            &mut common,
            &mut private,
            &crate::fx::FxHashSet::from_iter([(id, 0)]),
            &shared,
        );
        super::merge::tests::legacy::definite(
            &mut common,
            &mut private,
            &Default::default(),
            &shared,
        );
        assert!(
            common.unwrap().is_empty(),
            "the pristine sibling retains its callback slot"
        );
        assert!(
            private.is_empty(),
            "lazy module containers are shared, not arm-private"
        );
    }
}
