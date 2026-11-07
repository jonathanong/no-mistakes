use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn configured_text_slots_reject_known_raw_callers_and_preserve_ordinary_data() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-fragments"),
    );
    let file = root.join("src/text-callers.ts");
    let plan = TsFactPlan {
        imports: true,
        function_calls: true,
        value_flow: true,
        ..Default::default()
    };
    let prepared = collect_ts_facts(std::slice::from_ref(&file), plan);
    let ts = prepared.get(&file).unwrap();
    assert!(ts.parse_error.is_none());
    let mut facts = CheckFactMap {
        files: vec![file.clone()],
        graph_plan: plan,
        ..Default::default()
    };
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts.clone()),
            ..Default::default()
        }
        .into(),
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &config,
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
        .with_graph(&graph)
        .with_trusted_tags(&tags);
    let flow = &ts.value_flow;
    let tagged = |name| {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Call(start, end) = binding.value else {
            panic!("tagged slot");
        };
        flow.calls
            .iter()
            .find(|call| (call.offset, call.end) == (start, end))
            .unwrap()
    };
    for (name, expected) in [
        ("directCallerSlot", false),
        ("aliasCallerSlot", false),
        ("chainCallerSlot", false),
        ("objectCallerSlot", false),
        ("defaultCallerSlot", false),
        ("optionalCallerSlot", false),
        ("mutableCallerSlot", false),
        ("writtenParameterSlot", false),
        ("recursiveCallerSlot", false),
        ("ordinaryCallerSlot", true),
        ("ordinaryMemberSlot", true),
        ("ordinaryDiamondSlot", true),
    ] {
        assert_eq!(
            is_text_tag(&bindings, &file, tagged(name), &tags),
            expected,
            "{name}"
        );
    }
    let mut state = ProofState::default();
    assert!(text_tag(
        &bindings,
        &file,
        tagged("ordinaryDiamondSlot"),
        &tags,
        &mut state
    ));
    assert_eq!(state.callers.done.len(), 4);
}
