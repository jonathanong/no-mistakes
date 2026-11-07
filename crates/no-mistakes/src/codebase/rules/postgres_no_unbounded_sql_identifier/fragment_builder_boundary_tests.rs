use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn builder_proof_boundaries_require_call_facts_and_a_canonical_helper_site() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-builders");
    let file = root.join("src/proof-boundaries.ts");
    let prepared = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            value_flow: true,
            ..TsFactPlan::imports()
        },
    );
    let ts = prepared.get(&file).unwrap();
    assert!(ts.parse_error.is_none());
    let mut facts = CheckFactMap {
        files: vec![file.clone()],
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
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root,
        Default::default(),
        Default::default(),
    );
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts);
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let mut proof = Proof::new(&graph, &opts, &facts, &bindings);
    assert!(!proof.expression(&file, &ValueExpression::Call(u32::MAX - 1, u32::MAX), false));
    assert!(!proof.expression(&file, &ValueExpression::Unknown, false));
    let helper = flow
        .bindings
        .iter()
        .find(|binding| binding.name == "returnedStatement")
        .unwrap();
    assert!(!proof.expression(&file, &helper.value, false));
}

#[test]
fn builder_proof_boundaries_reject_active_cycles_without_publishing_success() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier");
    let mut active = FxHashSet::default();
    let key = Key::Call(root.clone(), 1, 2, true);
    assert!(active.insert(key));
    let facts = CheckFactMap::default();
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts);
    let mut proof = Proof::new(&graph, &opts, &facts, &bindings);
    proof.active = active;
    assert!(!proof.memoized(Key::Call(root, 1, 2, true), |_| true));
    assert!(proof.done.is_empty());
}
