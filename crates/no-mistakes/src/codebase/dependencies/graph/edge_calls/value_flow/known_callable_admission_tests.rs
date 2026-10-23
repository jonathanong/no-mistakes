use super::*;

fn site() -> ResolvedCallSite {
    let file = std::path::PathBuf::from("/saved/entry.mts");
    ResolvedCallSite {
        alternatives: Vec::new(),
        target_node: Some(NodeId::callable(&file, "direct", CallableId(10))),
        target: ResolvedCallTarget::RepositoryFunction {
            file: file.clone(),
            scope: "direct".into(),
        },
        file,
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "direct".into(),
        returned_from: vec![],
        typed_from: vec![],
        returned_member_from: vec![],
    }
}
fn prepared(facts: &crate::codebase::ts_source::facts::TsFileFacts) -> (CallableFileIndex<'_>, ValueCall) {
    let mut file =
        CallableFileIndex::from_facts(facts);
    file.callable_bindings
        .insert(0, ScopedNames::One("direct".into(), CallableId(10)));
    file.values.scopes.insert(CallableId(10), "direct".into());
    let call = ValueCall {
        offset: 0,
        end: 8,
        source_callee: "direct".into(),
        context: None,
        callee: ValueExpression::Named {
            name: "direct".into(),
            scope: Some(0),
            offset: 0,
            context: None,
        },
        arguments: std::sync::Arc::new(vec![]),
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    };
    (file, call)
}
#[test]
fn exact_node_requires_same_file_symbol_and_callable_identity() {
    let base = site();
    assert_eq!(exact_identity(&base), Some((CallableId(10), "direct")));
    let mut changed = base.clone();
    changed.target = ResolvedCallTarget::Unknown;
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target_node = None;
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target_node = Some(NodeId::file(&base.file));
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target_node = Some(NodeId::symbol(&base.file, "direct"));
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target_node = Some(NodeId::callable("/other.mts", "direct", CallableId(10)));
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target_node = Some(NodeId::callable(&base.file, "other", CallableId(10)));
    assert_eq!(exact_identity(&changed), None);
    changed = base.clone();
    changed.target = ResolvedCallTarget::RepositoryFunction {
        file: "/other.mts".into(),
        scope: "direct".into(),
    };
    assert_eq!(exact_identity(&changed), None);
}
#[test]
fn lexical_proof_requires_exact_raw_scope_id_and_simple_unbound_name() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts::default();
    let (mut file, mut call) = prepared(&facts);
    let site = site();
    assert!(exact_name(&file, &call, &site, CallableId(10), "direct"));
    assert!(!exact_name(&file, &call, &site, CallableId(11), "direct"));
    assert!(!exact_name(&file, &call, &site, CallableId(10), "other"));
    call.bound = true;
    assert!(!exact_name(&file, &call, &site, CallableId(10), "direct"));
    call.bound = false;
    for name in ["receiver.direct", "path/direct", "other"] {
        call.callee = ValueExpression::Named {
            name: name.into(),
            scope: Some(0),
            offset: 0,
            context: None,
        };
        assert!(!exact_name(&file, &call, &site, CallableId(10), "direct"));
    }
    call.callee = ValueExpression::Named {
        name: "direct".into(),
        scope: None,
        offset: 0,
        context: None,
    };
    assert!(!exact_name(&file, &call, &site, CallableId(10), "direct"));
    call.callee = ValueExpression::Unknown;
    assert!(!exact_name(&file, &call, &site, CallableId(10), "direct"));
    call.callee = ValueExpression::Named {
        name: "direct".into(),
        scope: Some(1),
        offset: 0,
        context: None,
    };
    // A matching parent declaration is insufficient: require the direct map branch.
    file.lexical_scope_parents.insert(1, Some(0));
    assert!(!exact_name(&file, &call, &site, CallableId(10), "direct"));
}

#[test]
fn missing_prepared_proof_and_non_call_sites_remain_refinable() {
    struct Prepared(Option<crate::codebase::ts_source::facts::TsFileFacts>);
    impl TsFactLookup for Prepared {
        fn get_ts_facts(
            &self,
            _: &std::path::Path,
        ) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
            self.0.as_ref()
        }
    }
    let mut site = site();
    // Missing facts and an empty prepared call table cannot establish redundancy.
    assert!(!exact_direct_callable(
        &Prepared(None),
        &CallableResolutionIndexes::default(),
        &site
    ));
    assert!(!exact_direct_callable(
        &Prepared(Some(Default::default())),
        &CallableResolutionIndexes::default(),
        &site
    ));
    site.invocation = InvocationKind::Construct;
    assert_eq!(exact_identity(&site), None);
}
