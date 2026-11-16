use super::*;
use crate::codebase::dependencies::extract::ValueBinding;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use std::sync::Arc;

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: Some(0),
        offset: 40,
        context: None,
    }
}

fn call(arguments: Vec<ValueExpression>) -> ValueCall {
    ValueCall {
        offset: 30,
        end: 50,
        source_callee: "TABLES.map".into(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(arguments),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

fn configured_result(flow: &ValueFlowFacts, call: &ValueCall) -> bool {
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        "/repo".into(),
        Default::default(),
        Default::default(),
    );
    let options: Options = serde_json::from_value(serde_json::json!({
        "identifierValidators": [{ "name": "safe", "argument": 0 }]
    }))
    .unwrap();
    let opts = compile("postgres-no-unbounded-sql-identifier", &options).unwrap();
    configured(
        &graph,
        &opts,
        &CheckFactMap::default(),
        Path::new("query.ts"),
        flow,
        call,
    )
}

#[test]
fn configured_named_contract_requires_one_matching_callback() {
    let flow = ValueFlowFacts::default();
    assert!(configured_result(&flow, &call(vec![named("safe")])));
    for arguments in [
        Vec::new(),
        vec![named("safe"), named("other")],
        vec![named("other")],
        vec![ValueExpression::Unknown],
    ] {
        assert!(!configured_result(&flow, &call(arguments)));
    }
}

#[test]
fn configured_named_contract_rejects_mutable_and_invalidated_bindings() {
    let callback = call(vec![named("safe")]);
    for (immutable, invalidated_at, expected) in [
        (true, None, true),
        (false, None, false),
        (true, Some(35), false),
    ] {
        let mut flow = ValueFlowFacts::default();
        flow.bindings.push(ValueBinding {
            immutable,
            owner: None,
            name: "safe".into(),
            scope: 0,
            declared_at: 10,
            repeated: false,
            invalidated_at,
            mutation_contexts: Default::default(),
            value: ValueExpression::Unknown,
        });
        assert_eq!(configured_result(&flow, &callback), expected);
    }
}

#[test]
fn literal_callback_requires_direct_unspread_callable_argument() {
    let flow = ValueFlowFacts::default();
    let callback = call(vec![ValueExpression::Callable(CallableId(40))]);
    let mut bound = callback.clone();
    bound.bound = true;
    assert!(!literal(&flow, &bound));
    let mut spread = callback;
    spread.spread_at = Some(0);
    assert!(!literal(&flow, &spread));
    for arguments in [
        Vec::new(),
        vec![named("safe")],
        vec![ValueExpression::Callable(CallableId(40)), named("other")],
    ] {
        assert!(!literal(&flow, &call(arguments)));
    }
}
