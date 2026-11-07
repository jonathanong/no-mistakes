use super::*;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};

#[test]
fn imported_builders_reject_dynamic_consumers_of_otherwise_proven_copies() {
    let fixtures = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("builder-runtime-consumers");
    for (scenario, expected_kind, target) in [
        ("dynamic", EdgeKind::DynamicImport, "producer.ts"),
        (
            "conditional",
            EdgeKind::ConditionalDynamicImport,
            "barrel.ts",
        ),
    ] {
        for include_consumer in [false, true] {
            let root = normalize_path(&fixtures.join(scenario));
            let mut files = vec![root.join("producer.ts"), root.join("query.ts")];
            if scenario == "conditional" {
                files.push(root.join("barrel.ts"));
            }
            if include_consumer {
                files.push(root.join("mutator.ts"));
            }
            let (facts, graph, root) = prepared_files(root, files);
            if include_consumer {
                assert!(graph
                    .dependents_of_node(&NodeId::file(root.join(target)))
                    .unwrap()
                    .iter()
                    .any(|(node, kind)| {
                        node == &NodeId::file(root.join("mutator.ts")) && *kind == expected_kind
                    }));
            }
            assert_eq!(
                copied_text_is_proven(&facts, &graph, root.join("query.ts")),
                !include_consumer,
                "{scenario}, include consumer: {include_consumer}"
            );
        }
    }
}

#[test]
fn imported_builders_keep_type_imports_and_resolution_only_consumers() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("builder-runtime-consumers/clean"),
    );
    let files = ["producer.ts", "query.ts", "types.ts", "resolve.js"]
        .map(|name| root.join(name))
        .to_vec();
    let (facts, graph, root) = prepared_files(root, files);
    let edges = graph
        .dependents_of_node(&NodeId::file(root.join("producer.ts")))
        .unwrap();
    for (name, expected_kind) in [
        ("types.ts", EdgeKind::TypeImport),
        ("resolve.js", EdgeKind::RequireResolve),
    ] {
        assert!(edges.iter().any(|(node, kind)| {
            node == &NodeId::file(root.join(name)) && *kind == expected_kind
        }));
    }
    assert!(copied_text_is_proven(&facts, &graph, root.join("query.ts")));
}

fn copied_text_is_proven(facts: &CheckFactMap, graph: &DepGraph, file: PathBuf) -> bool {
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(facts).with_graph(graph);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file,
        span: (0, u32::MAX),
        call: &embedded,
    };
    Proofs::new(graph, &opts, facts, &bindings).proven(&call, &reference("statement", 0))
}
