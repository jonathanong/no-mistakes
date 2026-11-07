use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

fn prepared() -> (CheckFactMap, DepGraph, PathBuf) {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID).join("issue-3073-builders"),
    );
    let files = [
        root.join("src/exact.ts"),
        root.join("src/negative.ts"),
        root.join("src/variants.ts"),
        root.join("src/proof-boundaries.ts"),
    ];
    let plan = TsFactPlan {
        value_flow: true,
        function_calls: true,
        ..TsFactPlan::imports()
    };
    let prepared = collect_ts_facts(&files, plan);
    let facts = CheckFactMap {
        files: files.to_vec(),
        graph_plan: plan,
        ts: prepared
            .into_iter()
            .map(|(file, ts)| {
                assert!(ts.parse_error.is_none(), "{file:?}: {:?}", ts.parse_error);
                (
                    file,
                    CheckFileFacts {
                        ts: Arc::new(ts),
                        ..Default::default()
                    }
                    .into(),
                )
            })
            .collect(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        files.to_vec(),
        &facts,
    )
    .unwrap();
    (facts, graph, root)
}

#[test]
fn builder_proof_boundaries_preserve_chains_and_reject_opaque_receivers() {
    let (facts, graph, root) = prepared();
    let file = root.join("src/proof-boundaries.ts");
    let opts = compile(RULE_ID, &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, u32::MAX),
        call: &embedded,
    };
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let mut proofs = Proofs::new(&graph, &opts, &facts, &bindings);
    for (name, expected) in [
        ("chainedStatement", true),
        ("namedStatement", true),
        ("returnedStatement", true),
        ("textReceiver", false),
        ("literalAppend", false),
        ("recursiveStatement", false),
        ("asyncStatement", false),
        ("boundStatement", false),
        ("unknownStatement", false),
    ] {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let source = Provenance::Reference {
            name: name.into(),
            scope: binding.scope,
            offset: u32::MAX,
            members: vec!["text".into()],
        };
        assert_eq!(proofs.proven(&call, &source), expected, "{name}");
    }
}

#[test]
fn exact_default_library_builder_shapes_have_parameterized_text() {
    let (facts, graph, root) = prepared();
    let opts = compile(RULE_ID, &Options::default()).unwrap();
    assert!(opts.embedded.trusted_sql_tags.is_empty());
    let bindings = Bindings::new(&facts).with_graph(&graph);
    for (file_name, names, expected) in [
        ("exact.ts", vec!["SELECT_COLUMNS", "statement", "s"], true),
        ("variants.ts", vec!["appendedStatement"], true),
        (
            "negative.ts",
            vec![
                "rawStatement",
                "mutatedStatement",
                "escapedStatement",
                "conditionalStatement",
                "rawChainStatement",
                "helperStatement",
                "unknownStatement",
                "shadowedStatement",
                "memberStatement",
                "fragmentStatement",
                "columns",
                "mutatedColumns",
            ],
            false,
        ),
    ] {
        let file = root.join("src").join(file_name);
        let embedded = EmbeddedSqlCall::default();
        let call = SqlCall {
            file: file.clone(),
            span: (0, u32::MAX),
            call: &embedded,
        };
        let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
        for name in names {
            let binding = flow
                .bindings
                .iter()
                .find(|binding| binding.name == name)
                .unwrap();
            let source = Provenance::Reference {
                name: name.into(),
                scope: binding.scope,
                offset: u32::MAX,
                members: vec!["text".into()],
            };
            assert_eq!(
                Proofs::new(&graph, &opts, &facts, &bindings).proven(&call, &source),
                expected,
                "{file_name}: {name}"
            );
        }
    }
}
