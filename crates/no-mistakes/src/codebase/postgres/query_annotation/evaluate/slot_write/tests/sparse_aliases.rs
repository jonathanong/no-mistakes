use super::*;

#[test]
fn replacing_arguments_slot_does_not_invalidate_the_detached_strict_formal() {
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
        parallel_arms: None,
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
    evaluator.module_environment(&path);

    let events = embedded
        .calls
        .iter()
        .zip(&embedded.call_spans)
        .filter_map(|(call, start)| {
            let line = source.lines().nth(call.line as usize - 1)?;
            let value = evaluator
                .events
                .get(&(path.clone(), *start))?
                .first()?
                .1
                .clone();
            Some((line.to_string(), value))
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        events.iter().find(|(sql, _)| sql.contains("preserved formal")),
        Some((_, Value::Prefix(sql, true, _))) if sql.contains("preserved formal")
    ));
    assert!(matches!(
        events.iter().find(|(sql, _)| sql.contains("dynamic slot")),
        Some((_, Value::Unknown))
    ));
    assert!(matches!(
        events
            .iter()
            .find(|(sql, _)| sql.contains("selected slot write")),
        Some((_, Value::Prefix(sql, true, _))) if sql.contains("selected slot write")
    ));
    let Some((_, prior_escape)) = events
        .iter()
        .find(|(sql, _)| sql.contains("prior escape taint"))
    else {
        panic!("missing prior-escape executor event");
    };
    assert!(matches!(
        prior_escape,
        Value::Possible(values)
            if values.iter().any(|value| matches!(value, Value::Unknown))
    ));
    assert!(matches!(
        events
            .iter()
            .find(|(sql, _)| sql.contains("awaited strict delete")),
        Some((_, Value::Prefix(sql, true, _))) if sql.contains("awaited strict delete")
    ));
    assert!(matches!(
        events.iter().find(|(sql, _)| sql.contains("extra slot read")),
        Some((_, Value::Prefix(sql, true, _))) if sql.contains("extra slot read")
    ));
    let Some((_, sloppy)) = events
        .iter()
        .find(|(line, _)| line.contains("sloppy mapped slot"))
    else {
        panic!("missing imported sloppy helper executor event");
    };
    // This single-file harness has no prepared imported helper facts; the
    // separate saved CJS test checks the actual mapped replacement.
    assert!(matches!(sloppy, Value::Unknown));
}
