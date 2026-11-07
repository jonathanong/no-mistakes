use super::*;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};

#[test]
fn imported_builders_copy_fragments_through_fresh_append_chains() {
    let (facts, graph, root) = prepared();
    let file = root.join("src/chain-copy.ts");
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    assert!(opts.embedded.trusted_sql_tags.is_empty());
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let binding = flow
        .bindings
        .iter()
        .find(|binding| binding.name == "statement")
        .unwrap();
    let execute = flow
        .calls
        .iter()
        .find(
            |call| matches!(&call.callee, ValueExpression::Named { name, .. } if name == "execute"),
        )
        .unwrap();
    let call = SqlCall {
        file,
        span: (execute.offset, execute.end),
        call: &embedded,
    };
    assert!(Proofs::new(&graph, &opts, &facts, &bindings)
        .proven(&call, &reference("statement", binding.scope)));
}

#[test]
fn imported_builders_reject_commonjs_mutation_of_an_otherwise_proven_const() {
    let (facts, graph, root) = prepared();
    let producer = root.join("src/cjs-producer.ts");
    let reader = root.join("src/cjs-reader.ts");
    let mutator = root.join("src/cjs-mutator.js");
    assert_eq!(
        graph.imported_value(&reader, "requiredStatement"),
        Some((producer.clone(), "requiredStatement".into()))
    );
    assert!(graph
        .dependents_of_node(&NodeId::file(&producer))
        .unwrap()
        .iter()
        .any(|(node, kind)| node == &NodeId::file(&mutator) && *kind == EdgeKind::Require));
    let readonly = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            imports: true,
            value_import_selectors: true,
            ..Default::default()
        },
        facts
            .files
            .iter()
            .filter(|file| *file != &mutator)
            .cloned()
            .collect(),
        &facts,
    )
    .unwrap();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: reader,
        span: (0, u32::MAX),
        call: &embedded,
    };
    for (graph, expected) in [(&readonly, true), (&graph, false)] {
        let bindings = Bindings::new(&facts).with_graph(graph);
        assert_eq!(
            Proofs::new(graph, &opts, &facts, &bindings)
                .proven(&call, &reference("requiredStatement", 0)),
            expected
        );
    }
}
