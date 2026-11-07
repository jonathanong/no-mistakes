use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

fn prepared() -> (CheckFactMap, PathBuf) {
    let file = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-text-origins/src/callers.ts"),
    );
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let files = vec![file.clone(), file.with_file_name("external.ts")];
    let collected = collect_ts_facts(&files, plan);
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: files
            .into_iter()
            .map(|file| {
                let ts = collected.get(&file).unwrap();
                assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
                (
                    file,
                    CheckFileFacts {
                        ts: Arc::new(ts.clone()),
                        ..Default::default()
                    }
                    .into(),
                )
            })
            .collect(),
        ..Default::default()
    };
    (facts, file)
}

fn tags() -> [TrustedSqlTag; 1] {
    [TrustedSqlTag {
        module: "@example/sql".into(),
        name: "sql".into(),
    }]
}

#[test]
fn caller_origins_preserve_nullable_awaited_and_projected_scalar_values() {
    let (facts, file) = prepared();
    let root = file.parent().unwrap();
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        root,
        &TsConfig {
            dir: root.into(),
            paths_dir: root.into(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        facts.files.clone(),
        &facts,
    )
    .unwrap();
    assert!(super::super::caller_index(&graph, &file, &Default::default()).is_empty());
    let tags = tags();
    let bindings = Bindings::new(&facts)
        .with_graph(&graph)
        .with_trusted_tags(&tags);
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    for (name, expected) in [
        ("nullableCallerSlot", true),
        ("awaitedCallerSlot", true),
        ("nullishCallerSlot", true),
        ("memberCallerSlot", true),
        ("arrayMemberCallerSlot", true),
        ("containerCallerSlot", false),
        ("undefinedBindSlot", true),
        ("awaitedBindSlot", true),
        ("nullishBindSlot", true),
        ("defaultProjectionSlot", false),
    ] {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Call(start, end) = binding.value else {
            panic!("tag {name}");
        };
        let tag = flow
            .calls
            .iter()
            .find(|call| (call.offset, call.end) == (start, end))
            .unwrap();
        assert_eq!(
            is_text_tag(&bindings, &file, tag, &tags),
            expected,
            "{name}"
        );
    }
}

include!("text_tag_origins_boundary_tests.rs");
