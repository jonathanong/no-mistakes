#[test]
fn nested_scalar_joins_discard_sql_proof_but_keep_callback_references() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-live-binding/src/nested-scalars.cjs",
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
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let root = evaluator.module_environment(&path);
    // Restoring the four two-arm alternatives updates only arm-created
    // readers. Their compactions do not change capture ownership either.
    assert!(
        !observer
            .snapshot()
            .work
            .contains_key("postgres.annotation.captured_reader_rebuilds")
    );
    for (name, references) in [("scalar", false), ("callback", true)] {
        let original = vec![
            FxHashMap::from_iter([("value".into(), evaluator.scopes[root][name].clone())]).into(),
        ];
        let mut joined = original.clone();
        let current =
            vec![FxHashMap::from_iter([("value".into(), super::super::Value::Unknown)]).into()];
        super::bindings::tests::legacy::join_with_sql(&mut joined, &current, &original, false);
        if references {
            assert!(matches!(joined[0]["value"], super::super::Value::Joined(_)));
        } else {
            assert!(matches!(joined[0]["value"], super::super::Value::Unknown));
        }
    }
}

#[test]
fn nested_capture_restoration_keeps_reverse_readers_exact() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-live-binding/src/capture-restoration.cjs",
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
    evaluator.module_environment(&path);
    assert!(!evaluator.captured_bindings.is_empty());
    let mut expected = FxHashMap::default();
    for (frame, bindings) in &evaluator.captured_bindings {
        for (name, origin) in bindings {
            expected
                .entry(*origin)
                .or_insert_with(FxHashMap::default)
                .entry(name.clone())
                .or_insert_with(crate::fx::FxHashSet::default)
                .insert(*frame);
        }
    }
    assert_eq!(evaluator.captured_binding_readers, expected);
}
