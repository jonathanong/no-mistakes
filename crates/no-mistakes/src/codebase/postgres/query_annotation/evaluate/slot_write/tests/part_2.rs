#[test]
fn static_slot_replacement_preserves_strict_formal_and_updates_sloppy_alias() {
    let (path, _) = fixture("query.mts");
    let facts = query_annotation::QueryAnnotationFileFacts::default();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: Default::default(),
        },
    )]);
    let original = Value::Prefix("/* annotated */ SELECT 1".into(), true, Some(9));
    let replacement = Value::Prefix("replacement".into(), true, None);
    let mut scope = fx_map();
    scope.insert("arguments".into(), Value::Arguments(1));
    scope.insert("statement".into(), original.clone());
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        transactions: Default::default(),
        scopes: vec![scope.into()].into(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 10,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: FxHashMap::from_iter([(1, vec![original.clone()])]),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let strict = evaluator.slot_write(
        &query_annotation::Expr::Name("arguments".into()),
        0,
        &query_annotation::Expr::Text("replacement".into()),
        &path,
        &0,
        (16, false),
    );

    assert!(matches!(strict, Value::Evaluated(_, true)));
    assert!(evaluator.scopes[0]["statement"] == original);
    assert!(evaluator.argument_objects[&1][0] == replacement);
    assert!(!evaluator.invalidated_builders.contains(&1));
    assert!(!evaluator.invalidated_builders.contains(&9));

    evaluator
        .mapped_arguments
        .insert(0, vec![(1, vec!["statement".into()])]);
    evaluator.rebuild_mapped_argument_owners();
    evaluator.scopes[0].insert(
        "statement".into(),
        Value::Prefix("/* annotated */ SELECT 1".into(), true, Some(9)),
    );
    evaluator.argument_objects.insert(
        1,
        vec![Value::Prefix(
            "/* annotated */ SELECT 1".into(),
            true,
            Some(9),
        )],
    );
    evaluator.invalidated_builders.clear();
    let sloppy = evaluator.slot_write(
        &query_annotation::Expr::Name("arguments".into()),
        0,
        &query_annotation::Expr::Text("replacement".into()),
        &path,
        &0,
        (16, false),
    );

    assert!(matches!(sloppy, Value::Evaluated(_, true)));
    assert!(evaluator.scopes[0]["statement"] == replacement);
    assert!(evaluator.mapped_parameter_value(0, "statement").is_none());
    assert!(!evaluator.invalidated_builders.contains(&9));

    let huge_index = 9_007_199_254_740_991usize;
    let stable = Value::Prefix("/* stored out of range */ SELECT 1".into(), true, Some(9));
    evaluator.scopes[0].insert("statement".into(), stable.clone());
    evaluator.argument_objects.insert(1, vec![stable.clone()]);
    evaluator.invalidated_builders.clear();
    let extra = evaluator.slot_write(
        &query_annotation::Expr::Name("arguments".into()),
        huge_index,
        &query_annotation::Expr::Name("statement".into()),
        &path,
        &0,
        (16, false),
    );
    assert!(matches!(extra, Value::Evaluated(_, true)));
    assert_eq!(evaluator.argument_objects[&1].len(), 1);
    assert!(matches!(
        evaluator.argument_extra_slots[&1].get(&huge_index),
        Some(Value::Prefix(text, true, Some(9))) if text == "/* stored out of range */ SELECT 1"
    ));
    assert!(!evaluator.invalidated_builders.contains(&9));
}

