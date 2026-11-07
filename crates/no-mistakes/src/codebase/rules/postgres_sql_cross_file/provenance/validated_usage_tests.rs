use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

#[test]
fn identifier_source_usage_requires_consuming_the_exact_scalar_return() {
    let file = PathBuf::from("source.ts");
    let mut returns = IdentifierReturns::default();
    returns.insert(file.clone(), (10, 20), vec![0, 1]);
    returns.insert(file.clone(), (30, 40), Vec::new());
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts).with_identifier_returns(&returns);
    let call = ValueExpression::Call(10, 20);
    assert!(!returns.source_used(0));
    assert!(!returns.source_used(1));
    assert!(!bindings.identifier_value(&file, &call, &["table".into()]));
    assert!(!bindings.identifier_value(Path::new("other.ts"), &call, &[]));
    assert!(!bindings.identifier_value(&file, &ValueExpression::Call(10, 21), &[]));
    assert!(!bindings.identifier_value(
        &file,
        &ValueExpression::Member(Arc::new(call.clone()), "table".into()),
        &[],
    ));
    assert!(!returns.source_used(0));
    assert!(!returns.source_used(1));
    assert!(bindings.identifier_value(&file, &ValueExpression::Call(30, 40), &[]));
    assert!(!returns.source_used(0));
    assert!(bindings.identifier_value(&file, &ValueExpression::Awaited(Arc::new(call)), &[]));
    assert!(returns.source_used(0));
    assert!(returns.source_used(1));
    assert!(!returns.source_used(2));
}

#[test]
fn identifier_source_return_proof_does_not_apply_to_member_classification() {
    let file = PathBuf::from("source.ts");
    let mut returns = IdentifierReturns::default();
    returns.insert(file.clone(), (10, 20), vec![0]);
    let mut ts = TsFileFacts::default();
    ts.value_flow.calls.push(ValueCall {
        offset: 10,
        end: 20,
        source_callee: "trusted".into(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(vec![ValueExpression::Named {
            name: "input".into(),
            scope: Some(1),
            offset: 12,
            context: None,
        }]),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    });
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts).with_identifier_returns(&returns);
    let mut out = BTreeSet::new();
    super::super::classify(
        &graph,
        &facts,
        &file,
        &ValueExpression::Member(Arc::new(ValueExpression::Call(10, 20)), "table".into()),
        &mut out,
        &bindings,
    );
    assert_eq!(out.len(), 1);
    assert!(!returns.source_used(0));
    out.clear();
    super::super::classify(
        &graph,
        &facts,
        &file,
        &ValueExpression::Call(10, 20),
        &mut out,
        &bindings,
    );
    assert!(out.is_empty());
    assert!(returns.source_used(0));
}
