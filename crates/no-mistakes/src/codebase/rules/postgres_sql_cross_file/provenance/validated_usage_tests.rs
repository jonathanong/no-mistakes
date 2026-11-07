use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::ValueBinding;
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
    assert!(bindings.identifier_value_without_usage(&file, &call));
    assert!(!returns.source_used(0));
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
    ts.value_flow.bindings.push(ValueBinding {
        immutable: true,
        owner: None,
        name: "returned".into(),
        scope: 1,
        declared_at: 5,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: Default::default(),
        value: ValueExpression::Call(10, 20),
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
    let call = Arc::new(ValueExpression::Call(10, 20));
    let named = Arc::new(ValueExpression::Named {
        name: "returned".into(),
        scope: Some(1),
        offset: 30,
        context: None,
    });
    for base in [
        call.as_ref().clone(),
        ValueExpression::Awaited(call.clone()),
        ValueExpression::Either(call.clone(), call.clone()),
        ValueExpression::Nullish(call.clone(), Arc::new(ValueExpression::Unknown)),
        ValueExpression::Default(call.clone(), Arc::new(ValueExpression::Unknown)),
        ValueExpression::Awaited(named.clone()),
        ValueExpression::Either(named.clone(), named.clone()),
    ] {
        out.clear();
        super::super::classify(
            &graph,
            &facts,
            &file,
            &ValueExpression::Member(Arc::new(base), "table".into()),
            &mut out,
            &bindings,
        );
        assert_eq!(out.len(), 1);
        for provenance in &out {
            match provenance {
                Provenance::Reference { name, members, .. } if name == "returned" => {
                    assert_eq!(members.join("."), "table");
                }
                _ => {}
            }
        }
        assert!(!returns.source_used(0));
    }
    for value in [call, named] {
        out.clear();
        super::super::classify_value(&graph, &facts, &file, &value, &mut out, &bindings);
        assert!(out.is_empty());
        assert!(!returns.source_used(0));
    }
    super::super::classify_call(
        &graph,
        &facts,
        &file,
        &facts.get_ts_facts(&file).unwrap().value_flow.calls[0],
        &mut out,
        &bindings,
    );
    assert!(out.is_empty());
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
