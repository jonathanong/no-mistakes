use super::super::{Evaluator, File};
use super::binding_scan_count::SCANS as REFERENCE_BINDING_SCANS;
use super::*;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;

#[test]
fn precise_record_arms_never_construct_reference_binding_indexes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/performance/annotation-shared-record-dag/reference-binding-index/root.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@data-stores/psql", &[]);
    for precise_records in [false, true] {
        let mut facts = crate::ast::with_program(&path, &source, |program, _| {
            query_annotation::collect(program, &source, &options)
        })
        .unwrap();
        assert!(facts.globals.contains_key("joined"));
        facts.precise_records = precise_records;
        let files = FxHashMap::from_iter([(
            path.clone(),
            File {
                facts: &facts,
                module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
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

        // Execute directly on this thread: this counter deliberately does not
        // follow Rayon workers, and later scope writes may invalidate indexes.
        REFERENCE_BINDING_SCANS.with(|count| count.set(0));
        evaluator.module_environment(&path);
        let scans = REFERENCE_BINDING_SCANS.with(|count| count.get());
        if precise_records {
            assert_eq!(scans, 0, "precise arms do not perform legacy comparisons");
        } else {
            assert!(
                scans > 0,
                "the saved reference-bearing branch exercises indexing"
            );
        }
    }
}
