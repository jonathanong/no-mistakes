use super::*;

#[test]
fn imported_builders_reject_only_namespaces_exposing_the_escaped_statement() {
    let (facts, graph, root) = prepared();
    let producer = root.join("src/namespace-escape-producer.ts");
    let writer = root.join("src/namespace-escapes.ts");
    for (selector, module, name) in [
        ("bare", "namespace-bare.ts", "bareStatement"),
        ("alias", "namespace-alias.ts", "aliasStatement"),
        ("quoted", "namespace-quoted.ts", "quotedStatement"),
        ("opaque", "namespace-opaque.ts", "opaqueStatement"),
    ] {
        let namespace = root.join("src").join(module);
        assert_eq!(
            graph.imported_namespace(&writer, selector),
            Some(namespace.clone())
        );
        assert!(graph.namespace_exports_value(&namespace, &producer, name));
        assert!(!graph.namespace_exports_value(&namespace, &producer, "relatedSafeStatement"));
    }
    let unrelated = graph.imported_namespace(&writer, "unrelated").unwrap();
    assert_eq!(unrelated, root.join("src/namespace-unrelated.ts"));
    assert!(!graph.namespace_exports_value(&unrelated, &producer, "relatedSafeStatement"));
    assert_eq!(
        graph.imported_value(&writer, "quotedAlias"),
        Some((producer, "quotedNamedStatement".into()))
    );

    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: root.join("src/namespace-escape-reader.ts"),
        span: (0, u32::MAX),
        call: &embedded,
    };
    let mut proof = Proofs::new(&graph, &opts, &facts, &bindings);
    for name in [
        "bareStatement",
        "aliasStatement",
        "quotedStatement",
        "opaqueStatement",
        "quotedNamedStatement",
    ] {
        assert!(!proof.proven(&call, &reference(name, 0)), "{name}");
    }
    assert!(proof.proven(&call, &reference("relatedSafeStatement", 0)));
}
