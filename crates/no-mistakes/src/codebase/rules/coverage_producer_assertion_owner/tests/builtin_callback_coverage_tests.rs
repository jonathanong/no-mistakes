use super::super::*;
use crate::codebase::dependencies::extract::{InvocationKind, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::ResolvedCallTarget;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

fn unknown_site(path: &Path, call: &ValueCall) -> ResolvedCallSite {
    ResolvedCallSite {
        file: path.to_path_buf(),
        caller: None,
        caller_id: call.context,
        line: 1,
        offset: call.offset,
        invocation: InvocationKind::Call,
        source_callee: call.source_callee.clone(),
        target: ResolvedCallTarget::Unknown,
        target_node: None,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
    }
}

fn classify_saved_call(
    path: &Path,
    select: impl Fn(&ValueCall, &crate::codebase::ts_source::facts::TsFileFacts) -> bool,
) -> Option<Vec<String>> {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let plan = TsFactPlan {
        function_calls: true,
        value_flow: true,
        call_eligibility: true,
        ..TsFactPlan::default()
    };
    let facts = collect_ts_facts(std::slice::from_ref(&path), plan);
    let file = facts.get(&path).unwrap();
    let call = file
        .value_flow
        .calls
        .iter()
        .find(|call| select(call, file))
        .unwrap();
    let site = unknown_site(&path, call);
    let mut sites = FxHashMap::default();
    sites.insert(NodeId::file(&path), vec![&site]);
    let index = builtins::CallIndex::new(&sites, &facts);
    builtins::classify(&site, &[], &index, &facts).map(|callbacks| callbacks.unresolved)
}

#[test]
fn unknown_outer_call_stays_unclassified_with_inner_native_callback_proof() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let path = root.join("src/projection-branches.mts");
    let result = classify_saved_call(&path, |call, file| {
        call.source_callee == "<unknown>"
            && file
                .builtin_callback_calls
                .keys()
                .any(|(start, _)| *start == call.offset)
    });
    assert!(result.is_none());
}

#[test]
fn missing_inline_callback_site_is_uncertain() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let path = root.join("src/continuations.mts");
    let unresolved = classify_saved_call(&path, |call, _| {
        call.source_callee.ends_with(".finally")
            && matches!(call.arguments.first(), Some(ValueExpression::Callable(_)))
    });
    assert_eq!(unresolved, Some(vec!["argument 1".to_string()]));
}

#[test]
fn repeated_method_group_requires_every_exact_span_to_be_proven() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let result = classify_saved_call(&root.join("src/review-parent.mts"), |call, file| {
        call.source_callee.ends_with(".find")
            && !file
                .builtin_callback_calls
                .contains_key(&(call.offset, call.end))
    });
    assert!(result.is_none());
}

#[test]
fn ignored_uncertainty_keeps_proven_numeric_missing_owner_findings() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("review-parent-ignore.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["target"], "numberMethods", "{report}");
    assert!(findings[0].get("causes").is_none(), "{report}");
}
