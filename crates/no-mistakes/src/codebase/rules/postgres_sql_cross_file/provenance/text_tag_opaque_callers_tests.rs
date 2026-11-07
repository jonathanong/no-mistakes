use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn caller_origins_reject_opaque_unknowns_and_retain_literal_scalar_paths() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-fragments"),
    );
    let file = root.join("src/text-opaque-callers.ts");
    let plan = TsFactPlan {
        value_flow: true,
        function_calls: true,
        ..TsFactPlan::imports()
    };
    let prepared = collect_ts_facts(std::slice::from_ref(&file), plan);
    let ts = prepared.get(&file).unwrap();
    assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
    let facts = CheckFactMap {
        files: vec![file.clone()],
        graph_plan: plan,
        ts: [(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts.clone()),
                ..Default::default()
            }
            .into(),
        )]
        .into_iter()
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
        vec![file.clone()],
        &facts,
    )
    .unwrap();
    let tags = [TrustedSqlTag {
        module: "@example/sql".into(),
        name: "sql".into(),
    }];
    let bindings = Bindings::new(&facts)
        .with_trusted_tags(&tags)
        .with_graph(&graph);
    let flow = &ts.value_flow;
    for (name, expected) in [
        ("opaqueOptionalSlot", false),
        ("opaqueFactorySlot", false),
        ("opaqueAliasSlot", false),
        ("opaqueObjectSlot", false),
        ("opaqueMixedSlot", false),
        ("opaqueDuplicateSlot", false),
        ("literalActualSlot", true),
        ("literalAliasActualSlot", true),
        ("literalObjectActualSlot", true),
        ("literalArrayActualSlot", true),
        ("literalConditionalActualSlot", true),
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
    let origins = flow
        .tagged_templates
        .values()
        .next()
        .unwrap()
        .origins
        .as_ref()
        .unwrap();
    assert!(flow
        .tagged_templates
        .values()
        .all(|info| Arc::ptr_eq(origins, info.origins.as_ref().unwrap())));
}
