use super::*;

#[test]
fn imported_builders_require_complete_producer_and_consumer_facts() {
    for missing in ["builders.ts", "aliased.ts"] {
        let (mut facts, graph, root) = prepared();
        facts.ts.remove(&root.join("src").join(missing));
        let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
        let bindings = Bindings::new(&facts).with_graph(&graph);
        let embedded = EmbeddedSqlCall::default();
        let call = SqlCall {
            file: root.join("src/clean.ts"),
            span: (0, u32::MAX),
            call: &embedded,
        };
        assert!(
            !Proofs::new(&graph, &opts, &facts, &bindings)
                .proven(&call, &reference("importedStatement", 0)),
            "{missing}"
        );
    }
}

#[test]
fn imported_builders_refuse_incomplete_source_facts() {
    let (mut facts, graph, root) = prepared();
    let file = root.join("src/builders.ts");
    let mut source = facts.get_ts_facts(&file).unwrap().clone();
    source.value_flow.incomplete_reason = Some("budget".into());
    facts.ts.insert(
        file,
        CheckFileFacts {
            ts: Arc::new(source),
            ..Default::default()
        }
        .into(),
    );
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: root.join("src/clean.ts"),
        span: (0, u32::MAX),
        call: &embedded,
    };
    assert!(!Proofs::new(&graph, &opts, &facts, &bindings)
        .proven(&call, &reference("importedStatement", 0)));
}
