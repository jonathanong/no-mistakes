use super::super::*;
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::ResolvedCallTarget;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

#[test]
fn import_main_timer_uses_lexical_call_and_shadowed_timer_stays_uncertain() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("review-timer.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["target"], "shadowedTimer", "{report}");
    assert!(findings[0]["message"]
        .as_str()
        .unwrap()
        .contains("unanalyzable"));
}

#[test]
fn timer_without_an_exact_lexical_call_match_is_uncertain() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let path = crate::codebase::ts_resolver::normalize_path(&root.join("src/review-timer.mts"));
    let plan = TsFactPlan {
        function_calls: true,
        value_flow: true,
        call_eligibility: true,
        ..TsFactPlan::default()
    };
    let facts = collect_ts_facts(std::slice::from_ref(&path), plan);
    let call = facts[&path]
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "setTimeout")
        .unwrap();
    let site = ResolvedCallSite {
        file: path,
        caller: Some("<entry-point>".to_string()),
        caller_id: Some(CallableId(call.offset)),
        line: 1,
        offset: call.offset + 1,
        invocation: InvocationKind::Call,
        source_callee: "setTimeout".to_string(),
        target: ResolvedCallTarget::Global {
            name: "setTimeout".to_string(),
        },
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    let mut sites = FxHashMap::default();
    sites.insert(NodeId::file(&site.file), vec![&site]);
    let calls = builtins::CallIndex::new(&sites, &facts);
    let callbacks = builtins::classify(&site, &[], &calls, &facts).unwrap();
    assert!(callbacks.executed.is_empty());
    assert_eq!(callbacks.unresolved, ["timer callback"]);
}
