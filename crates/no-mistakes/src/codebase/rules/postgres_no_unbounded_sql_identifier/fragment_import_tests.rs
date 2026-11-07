use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn prepared() -> (CheckFactMap, DepGraph, PathBuf) {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-imported-builders"),
    );
    let files: Vec<_> = [
        "builders.ts",
        "clean.ts",
        "barrel.ts",
        "aliased.ts",
        "namespace.ts",
        "unsafe.ts",
        "dirty.ts",
        "shadowed.ts",
        "cycle-a.ts",
        "cycle-b.ts",
        "namespace-producer.ts",
        "namespace-barrel.ts",
        "namespace-writes.ts",
        "namespace-reader.ts",
        "namespace-readonly.ts",
        "namespace-escape-producer.ts",
        "namespace-bare.ts",
        "namespace-alias.ts",
        "namespace-quoted.ts",
        "namespace-opaque.ts",
        "namespace-unrelated.ts",
        "namespace-escape-barrel.ts",
        "namespace-escapes.ts",
        "namespace-escape-reader.ts",
    ]
    .iter()
    .map(|name| root.join("src").join(name))
    .collect();
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: collect_ts_facts(&files, plan)
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
            value_import_selectors: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    (facts, graph, root)
}

pub(super) fn reference(name: &str, scope: usize) -> Provenance {
    Provenance::Reference {
        name: name.into(),
        scope,
        offset: u32::MAX,
        members: vec!["text".into()],
    }
}

#[test]
fn imported_builders_keep_factory_returns_const_fragments_and_namespace_reads() {
    let (facts, graph, root) = prepared();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let calls: Vec<_> = ["clean.ts", "aliased.ts", "namespace.ts"]
        .iter()
        .map(|name| SqlCall {
            file: root.join("src").join(name),
            span: (0, u32::MAX),
            call: &embedded,
        })
        .collect();
    let mut proof = Proofs::new(&graph, &opts, &facts, &bindings);
    let flow = &facts.get_ts_facts(&calls[0].file).unwrap().value_flow;
    let statements: Vec<_> = flow
        .bindings
        .iter()
        .filter(|binding| binding.name == "statement")
        .collect();
    assert_eq!(statements.len(), 3);
    for binding in statements {
        assert!(
            proof.proven(&calls[0], &reference("statement", binding.scope)),
            "{}",
            binding.declared_at
        );
    }
    assert!(proof.proven(&calls[0], &reference("importedStatement", 0)));
    assert!(proof.proven(&calls[1], &reference("aliasedStatement", 0)));
    assert!(proof.proven(
        &calls[2],
        &Provenance::Namespace {
            local: "statements".into(),
            exported: "statement".into(),
            members: vec!["statement".into(), "text".into()],
        }
    ));
}

#[test]
fn imported_builders_reject_raw_returns_writes_escapes_and_shadowed_namespaces() {
    let (facts, graph, root) = prepared();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: root.join("src/dirty.ts"),
        span: (0, u32::MAX),
        call: &embedded,
    };
    let mut proof = Proofs::new(&graph, &opts, &facts, &bindings);
    for binding in facts
        .get_ts_facts(&call.file)
        .unwrap()
        .value_flow
        .bindings
        .iter()
        .filter(|binding| matches!(binding.name.as_str(), "raw" | "escaped" | "cycleStatement"))
    {
        assert!(!proof.proven(&call, &reference(&binding.name, binding.scope)));
    }
    for name in [
        "rawStatement",
        "mutatedStatement",
        "escapedStatement",
        "consumerMutated",
        "consumerEscaped",
        "consumerAliased",
    ] {
        assert!(!proof.proven(&call, &reference(name, 0)), "{name}");
    }
    let shadowed = SqlCall {
        file: root.join("src/shadowed.ts"),
        span: (0, u32::MAX),
        call: &embedded,
    };
    assert!(!proof.proven(
        &shadowed,
        &Provenance::Namespace {
            local: "statements".into(),
            exported: "statement".into(),
            members: vec!["statement".into(), "text".into()],
        }
    ));
}

#[path = "fragment_import_boundary_tests.rs"]
mod boundaries;

#[path = "fragment_import_prefix_tests.rs"]
mod prefixes;

#[path = "fragment_namespace_escape_tests.rs"]
mod namespace_escapes;
