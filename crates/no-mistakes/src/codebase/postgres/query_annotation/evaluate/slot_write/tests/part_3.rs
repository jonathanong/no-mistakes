#[test]
fn sloppy_fixture_slot_write_updates_its_actual_mapped_parameter() {
    let (path, facts) = fixture("sloppy.cjs");
    assert!(facts.mapped_arguments.iter().any(|start| {
        matches!(facts.globals.get("sloppyReplacement"),
            Some(query_annotation::Expr::Function(function)) if function.start == *start)
    }));
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
        callee: Box::new(query_annotation::Expr::Name("sloppyReplacement".into())),
        args: vec![query_annotation::Expr::Text("original".into())],
        start: 0,
        end: 0,
    };
    let result = evaluator.expr(&call, &path, &root, 16, false);

    assert!(matches!(result, Value::Prefix(text, true, None) if text == "replacement"));
    let Some((frame, _)) = evaluator.mapped_arguments.iter().find(|(_, mappings)| {
        mappings
            .iter()
            .any(|(_, params)| params.iter().any(|name| name == "statement"))
    }) else {
        panic!("missing actual sloppy mapped frame");
    };
    assert!(matches!(
        evaluator.scopes[*frame].get("statement"),
        Some(Value::Prefix(text, true, None)) if text == "replacement"
    ));

    let recreated = query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name("deleteAndRecreate".into())),
        args: vec![query_annotation::Expr::Text(
            "/* disconnected formal */ SELECT 1".into(),
        )],
        start: 1,
        end: 1,
    };
    let recreated_result = evaluator.expr(&recreated, &path, &root, 16, false);
    assert!(matches!(
        recreated_result,
        Value::Prefix(text, true, None) if text == "/* disconnected formal */ SELECT 1"
    ));
    let Some((id, _)) = evaluator
        .disconnected_argument_slots
        .iter()
        .find(|(_, index)| *index == 0)
    else {
        panic!("deleted sloppy parameter lost its permanent disconnection");
    };
    assert!(!evaluator
        .definite_deleted_argument_slots
        .contains(&(*id, 0)));
    assert!(!evaluator.deleted_argument_slots.contains(&(*id, Some(0))));
}

#[test]
fn appending_a_builder_updates_its_sparse_argument_slot_alias() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-argument-slot-write/src/query.mts",
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
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    evaluator.scopes[root].insert(
        "stableBuilder".into(),
        Value::Prefix("/* sparse alias */ SELECT 1".into(), true, Some(77)),
    );
    let call = query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name("extraBuilderAppend".into())),
        args: vec![query_annotation::Expr::Name("stableBuilder".into())],
        start: 0,
        end: 0,
    };
    let result = evaluator.expr(&call, &path, &root, 16, false);

    assert!(matches!(
        result,
        Value::Prefix(text, true, Some(77)) if text == "/* sparse alias */ SELECT 1 /* changed after extra-slot storage */"
    ));
    assert!(evaluator.argument_extra_slots.values().any(|slots| {
        matches!(slots.get(&9),
            Some(Value::Prefix(text, true, Some(77))) if text == "/* sparse alias */ SELECT 1 /* changed after extra-slot storage */")
    }));
}

