#[test]
fn sloppy_formal_callback_writes_are_seen_after_escape_and_by_a_later_consumer() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-mapped-formal-callback/src/helper.cjs",
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
    let mut evaluator = setup::evaluator(&files);
    evaluator.module_environment(&path);

    for (marker, expected) in [
        ("formal-write-after-escape", "SELECT 1"),
        ("second-consumer", "SELECT 3"),
    ] {
        let start = embedded
            .calls
            .iter()
            .zip(&embedded.call_spans)
            .find_map(|(call, start)| {
                source
                    .lines()
                    .nth(call.line as usize - 1)
                    .is_some_and(|line| line.contains(marker))
                    .then_some(*start)
            })
            .expect("marked nested executor call");
        assert!(
            matches!(
                evaluator.events.get(&(path.clone(), start)).and_then(|events| events.first()),
                Some((_, Value::Possible(values)))
                    if values.iter().any(|value| matches!(value, Value::Prefix(text, _, _) if text == expected))
            ),
            "missing modeled callback write for {marker}"
        );
    }

    let literal_start = embedded
        .calls
        .iter()
        .zip(&embedded.call_spans)
        .find_map(|(call, start)| {
            source
                .lines()
                .nth(call.line as usize - 1)
                .is_some_and(|line| line.contains("literal-formal-write-after-escape"))
                .then_some(*start)
        })
        .expect("marked literal callback executor call");
    assert!(matches!(
        evaluator.events.get(&(path.clone(), literal_start)).and_then(|events| events.first()),
        Some((_, Value::Prefix(text, true, None))) if text == "SELECT 4"
    ));

    let control_start = embedded
        .calls
        .iter()
        .zip(&embedded.call_spans)
        .find_map(|(call, start)| {
            source
                .lines()
                .nth(call.line as usize - 1)
                .is_some_and(|line| line.contains("known:noescape-control"))
                .then_some(*start)
        })
        .expect("marked no-escape write");
    assert!(matches!(
        evaluator.events.get(&(path, control_start)).and_then(|events| events.first()),
            Some((_, Value::Prefix(text, true, _))) if text == "/* no escape control */ SELECT 2"
    ));
}
