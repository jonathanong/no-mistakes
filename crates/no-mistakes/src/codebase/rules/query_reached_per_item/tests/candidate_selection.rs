use super::{candidates, CheckFactMap, NoMistakesConfig, ResolvedCallSite, RULE_ID};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{CallableId, InvocationKind, PerItemCall};
use crate::codebase::dependencies::graph::{test_support::from_raw_maps, ResolvedCallTarget};
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn shared_offsets_select_exact_callers_and_callback_kinds() {
    let root = crate::codebase::ts_resolver::normalize_path(&super::root());
    let file = root.join("src/entry.mts");
    let alpha = CallableId(1);
    let beta = CallableId(2);
    let gamma = CallableId(3);
    let site = |source_callee: &str, offset, caller_id, invocation| ResolvedCallSite {
        file: file.clone(),
        caller: Some("run".into()),
        caller_id: Some(caller_id),
        line: 10,
        offset,
        invocation,
        source_callee: source_callee.into(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    let graph = from_raw_maps(root.clone(), Default::default(), Default::default())
        .with_sorted_call_sites(vec![
            site("earlierCall", 41, alpha, InvocationKind::Call),
            site("alphaCall", 42, alpha, InvocationKind::Call),
            site("alphaCallback", 42, alpha, InvocationKind::Callback),
            site("betaCall", 42, beta, InvocationKind::Call),
            site("betaCallback", 42, beta, InvocationKind::Callback),
            site("gammaCall", 42, gamma, InvocationKind::Call),
            site("secondAlphaCall", 42, alpha, InvocationKind::Call),
            site("laterCall", 43, alpha, InvocationKind::Call),
        ]);
    let position = |callee: &str, caller_id, is_callback| PerItemCall {
        offset: 42,
        line: 10,
        callee: callee.into(),
        caller_id: Some(caller_id),
        construct: "map callback".into(),
        is_callback,
        capped_cursor_page_read: false,
        keyset_page_read: false,
    };
    let mut facts = CheckFactMap {
        graph_files: vec![file.clone()],
        graph_files_complete: true,
        ..Default::default()
    };
    facts.ts.insert(
        file,
        CheckFileFacts {
            ts: TsFileFacts {
                per_item_calls: vec![
                    position("alphaCall", alpha, false),
                    position("alphaCallback", alpha, true),
                    position("betaCall", beta, false),
                ],
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
    );
    let filter = RulePathFilter::new(
        &root,
        &NoMistakesConfig::default(),
        &RuleDef {
            rule: RULE_ID.into(),
            enabled: true,
            scope: Some(RuleScope::Repository),
            ..Default::default()
        },
    )
    .unwrap();

    let mut selected = candidates::select(&graph, &facts, &filter)
        .into_iter()
        .map(|(site, position)| (site.source_callee.as_str(), position.callee.as_str()))
        .collect::<Vec<_>>();
    selected.sort_unstable();
    assert_eq!(
        selected,
        [
            ("alphaCall", "alphaCall"),
            ("alphaCallback", "alphaCallback"),
            ("betaCall", "betaCall"),
            ("secondAlphaCall", "alphaCall"),
        ]
    );
}
