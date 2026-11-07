use super::*;

#[test]
fn imported_builders_invalidate_only_the_producer_written_through_namespace_reexports() {
    let (facts, graph, root) = prepared();
    let producer = root.join("src/namespace-producer.ts");
    let writer = root.join("src/namespace-writes.ts");
    for (selector, name) in [
        ("grouped.namedWrite", "namedWrite"),
        ("all.grouped.namespaceWrite", "namespaceWrite"),
    ] {
        assert_eq!(
            graph.imported_value(&writer, selector),
            Some((producer.clone(), name.into()))
        );
    }
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let reader = SqlCall {
        file: root.join("src/namespace-reader.ts"),
        span: (0, u32::MAX),
        call: &embedded,
    };
    let mut proof = Proofs::new(&graph, &opts, &facts, &bindings);
    assert!(!proof.proven(&reader, &reference("namedWrite", 0)));
    assert!(!proof.proven(&reader, &reference("namespaceWrite", 0)));
    assert!(proof.proven(&reader, &reference("untouched", 0)));
}
