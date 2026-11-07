#[test]
fn alternative_origins_require_both_scalar_branches_and_their_literal_receipts() {
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts);
    let file = Path::new("missing.ts");
    let literal = TextOrigin {
        literal: true,
        ..Default::default()
    };
    let origin = TextOrigin {
        parts: vec![Some(literal.clone()), Some(literal)],
        ..Default::default()
    };
    let unknown = Arc::new(ValueExpression::Unknown);
    let null = Arc::new(ValueExpression::Null);
    let undefined = Arc::new(ValueExpression::Undefined);
    for value in [
        ValueExpression::Either(unknown.clone(), unknown.clone()),
        ValueExpression::Default(unknown.clone(), unknown),
        ValueExpression::Nullish(null.clone(), undefined.clone()),
    ] {
        assert!(ordinary(
            &bindings,
            file,
            &value,
            Some(&origin),
            &[],
            &[],
            &mut ProofState::default()
        ));
    }
    let unsafe_value = ValueExpression::Either(null, Arc::new(ValueExpression::Unknown));
    assert!(!ordinary(
        &bindings,
        file,
        &unsafe_value,
        None,
        &[],
        &[],
        &mut ProofState::default()
    ));
    for value in [
        ValueExpression::Null,
        ValueExpression::Undefined,
        ValueExpression::Awaited(undefined),
    ] {
        assert!(arguments::bound_argument(
            &bindings,
            file,
            &value,
            &[],
            &[],
            &mut ProofState::default()
        ));
        assert!(!arguments::bound_argument(
            &bindings,
            file,
            &value,
            &["field".into()],
            &[],
            &mut ProofState::default()
        ));
    }
    for value in [
        ValueExpression::Either(
            Arc::new(ValueExpression::Null),
            Arc::new(ValueExpression::Undefined),
        ),
        ValueExpression::Default(
            Arc::new(ValueExpression::Null),
            Arc::new(ValueExpression::Null),
        ),
        ValueExpression::Nullish(
            Arc::new(ValueExpression::Undefined),
            Arc::new(ValueExpression::Null),
        ),
    ] {
        assert!(arguments::bound_argument(
            &bindings,
            file,
            &value,
            &[],
            &[],
            &mut ProofState::default()
        ));
    }
}

#[test]
fn missing_prepared_facts_cannot_prove_named_arguments_or_tagged_text() {
    let (prepared, file) = prepared();
    let flow = &prepared.get_ts_facts(&file).unwrap().value_flow;
    let tag = flow
        .calls
        .iter()
        .find(|call| flow.tagged_templates.contains_key(&(call.offset, call.end)))
        .unwrap();
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts);
    let value = ValueExpression::Named {
        name: "value".into(),
        scope: Some(1),
        offset: 0,
        context: None,
    };
    assert!(!ordinary(
        &bindings,
        &file,
        &value,
        None,
        &[],
        &[],
        &mut ProofState::default()
    ));
    assert!(!arguments::bound_argument(
        &bindings,
        &file,
        &value,
        &[],
        &[],
        &mut ProofState::default()
    ));
    assert!(!is_text_tag(&bindings, &file, tag, &tags()));
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        file.parent().unwrap().into(),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let mut parameter = flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "value")
        .unwrap()
        .clone();
    parameter.default = ValueExpression::Unknown;
    parameter.projection.clear();
    assert!(!super::super::parameter_bound(
        &bindings,
        &file,
        &parameter,
        0,
        &[],
        &[],
        &mut ProofState::default()
    ));
}
