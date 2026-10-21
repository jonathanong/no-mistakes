use super::*;

#[test]
fn only_canonical_static_arguments_indexes_are_collected_as_slot_writes() {
    let (_, strict) = fixture("query.mts");
    let (_, sloppy) = fixture("sloppy.cjs");
    let (strict_start, strict_index) = slot_write(&strict, "strictReplacement");
    let (sloppy_start, sloppy_index) = slot_write(&sloppy, "sloppyReplacement");

    assert_eq!(strict_index, 0);
    assert_eq!(sloppy_index, 0);
    assert!(!strict.mapped_arguments.contains(&strict_start));
    assert!(sloppy.mapped_arguments.contains(&sloppy_start));

    let Some(query_annotation::Expr::Function(function)) = strict.globals.get("dynamicReplacement")
    else {
        panic!("missing dynamic key case");
    };
    assert!(matches!(
        function.body.iter().find_map(|step| match step {
            query_annotation::Step::Bind(_, expr) => Some(expr),
            _ => None,
        }),
        Some(query_annotation::Expr::OpaqueWrite { .. })
    ));
}

#[test]
fn argument_slot_index_classification_is_canonical_and_receiver_specific() {
    let (_, facts) = fixture("index-cases.mts");
    let Some(query_annotation::Expr::Function(function)) = facts.globals.get("indexCases") else {
        panic!("missing index classification fixture function");
    };
    let mut indexes = Vec::new();
    let mut opaque_writes = 0;
    for step in &function.body {
        let query_annotation::Step::Bind(_, expression) = step else {
            continue;
        };
        match expression {
            query_annotation::Expr::SlotWrite { index, .. } => indexes.push(*index),
            query_annotation::Expr::OpaqueWrite { .. } => opaque_writes += 1,
            _ => {}
        }
    }

    assert_eq!(indexes, [0, 1, 2, 0]);
    // Fractional, unsafe/huge, noncanonical, nonnumeric, and dynamic indexes
    // stay opaque. Other receivers are classified at runtime conservatively.
    assert_eq!(opaque_writes, 6);
}

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
        scopes: vec![scope.into()],
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
        argument_objects: ArgumentObjects::from_iter([(1, vec![original.clone()])]),
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
