#[test]
fn class_members_are_indexed_once_by_class_identity() {
    use crate::codebase::dependencies::extract::CallableId;

    let members = vec![
        (CallableId(1), "run".to_string(), CallableId(11)),
        (CallableId(2), "run".to_string(), CallableId(21)),
        (CallableId(1), "init".to_string(), CallableId(12)),
        (CallableId(2), "stop".to_string(), CallableId(22)),
    ];
    let indexed = index_class_members_by_id(&members);

    assert_eq!(indexed.len(), 2);
    assert_eq!(
        indexed.values().map(|members| members.len()).sum::<usize>(),
        members.len(),
        "one-pass grouping keeps one entry per member, not a cartesian product",
    );
    assert_eq!(indexed[&CallableId(1)]["run"], CallableId(11));
    assert_eq!(indexed[&CallableId(2)]["run"], CallableId(21));
    assert!(!indexed[&CallableId(1)].contains_key("stop"));
}

#[test]
fn callable_file_index_looks_up_preindexed_class_members() {
    use crate::codebase::dependencies::extract::{CallableId, FunctionCall};

    let construct = |caller_id: Option<CallableId>,
                     callee: &str,
                     is_callback: bool,
                     invocation: InvocationKind,
                     identity: CallTargetIdentity| {
        FunctionCall {
            caller: Some("Alpha".to_string()),
            caller_id,
            syntactic_caller: None,
            callee: callee.to_string(),
            line: 1,
            offset: 0,
            is_callback,
            invocation,
            target_identity: identity,
            callee_binding_scope: None,
            static_arg: None,
            static_cwd: None,
        }
    };
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_scopes: vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()],
        class_scopes: vec!["Alpha".to_string(), "Beta".to_string()],
        callable_scope_ids: vec![
            (CallableId(1), "Alpha".to_string()),
            (CallableId(2), "Beta".to_string()),
            (CallableId(3), "Gamma".to_string()),
        ],
        callable_bindings: vec![
            (0, "Alpha".to_string(), CallableId(1)),
            (0, "Beta".to_string(), CallableId(2)),
            (0, "Gamma".to_string(), CallableId(3)),
        ],
        class_member_callable_ids: vec![
            (CallableId(1), "run".to_string(), CallableId(11)),
            (CallableId(2), "run".to_string(), CallableId(21)),
            (CallableId(1), "init".to_string(), CallableId(12)),
        ],
        static_getter_callable_ids: vec![(CallableId(1), "value".to_string(), CallableId(13))],
        static_setter_callable_ids: vec![(CallableId(1), "value".to_string(), CallableId(14))],
        function_calls: vec![
            construct(
                Some(CallableId(1)),
                "Base",
                true,
                InvocationKind::Construct,
                CallTargetIdentity::RepositoryFunction,
            ),
            construct(
                Some(CallableId(1)),
                "IgnoredLaterBase",
                true,
                InvocationKind::Construct,
                CallTargetIdentity::RepositoryFunction,
            ),
            construct(
                None,
                "MissingCaller",
                true,
                InvocationKind::Construct,
                CallTargetIdentity::RepositoryFunction,
            ),
            construct(
                Some(CallableId(2)),
                "NotCallback",
                false,
                InvocationKind::Construct,
                CallTargetIdentity::RepositoryFunction,
            ),
            construct(
                Some(CallableId(2)),
                "NotConstruct",
                true,
                InvocationKind::Call,
                CallTargetIdentity::RepositoryFunction,
            ),
            construct(
                Some(CallableId(2)),
                "UnknownIdentity",
                true,
                InvocationKind::Construct,
                CallTargetIdentity::Unknown,
            ),
        ],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);
    assert_eq!(
        index.class_scopes,
        ["Alpha".to_string(), "Beta".to_string()]
            .into_iter()
            .collect(),
        "the class membership index must retain every class and exclude ordinary callables"
    );
    let alpha = index
        .class_bindings
        .get(&(0, "Alpha".to_string()))
        .expect("Alpha is a class binding");
    let beta = index
        .class_bindings
        .get(&(0, "Beta".to_string()))
        .expect("Beta is a class binding");

    assert_eq!(alpha.static_member_ids.get("run"), Some(&CallableId(11)));
    assert_eq!(alpha.static_member_ids.get("init"), Some(&CallableId(12)));
    assert_eq!(alpha.static_getter_ids.get("value"), Some(&CallableId(13)));
    assert_eq!(alpha.static_setter_ids.get("value"), Some(&CallableId(14)));
    assert_eq!(beta.static_member_ids.get("run"), Some(&CallableId(21)));
    assert!(!alpha.static_member_ids.contains_key("stop"));
    assert_eq!(alpha.local_base.as_deref(), Some("Base"));
    assert!(beta.local_base.is_none());
    assert!(
        !index
            .class_bindings
            .contains_key(&(0, "Gamma".to_string())),
        "non-class bindings must not enter the class member index"
    );
}

#[test]
fn callable_file_index_construction_preindexes_class_members() {
    let source = include_str!("../edge_calls.rs");
    let helpers = include_str!("../edge_calls/index_build.rs");

    // Output equality cannot detect a quadratic membership scan. Keep the
    // constructor using the same hash index retained for later resolution.
    assert!(
        source.contains(".filter(|(_, scope)| class_scopes.contains(scope))"),
        "callable identities must probe the prebuilt class membership index",
    );
    assert!(
        !source.contains("file.class_scopes.contains(")
            && !source.contains("class_scopes: file.class_scopes"),
        "construction must reuse its class membership index instead of scanning or rebuilding it",
    );
    assert!(
        helpers.contains("for (class_id, member, member_id) in members"),
        "class_member_callable_ids must be grouped in one pass by class identity",
    );
    assert!(
        source.contains("index_class_members_by_id(&file.class_member_callable_ids)"),
        "from_facts must assemble class bindings from the pre-indexed member map",
    );
    assert!(
        source.contains("index_class_members_by_id(&file.static_getter_callable_ids)")
            && source.contains("index_class_members_by_id(&file.static_setter_callable_ids)"),
        "from_facts must keep accessor identities off the method map",
    );
    assert!(
        !source.contains("candidate_class_id"),
        "from_facts must not rescan class_member_callable_ids per class binding",
    );
}
