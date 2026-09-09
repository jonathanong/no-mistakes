#[test]
fn resource_diagnostics_follow_exported_resource_members_only() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        exported_resource_roots: vec!["loader".to_string()],
        ..Default::default()
    };
    let diagnostic = |scope: &str| crate::codebase::ts_resources::ResourceDiagnostic {
        kind: crate::codebase::ts_resources::ResourceDiagnosticKind::DynamicPath,
        line: 1,
        function_scope: Some(scope.to_string()),
        function_scope_id: Some(crate::codebase::dependencies::extract::CallableId(1)),
    };

    assert!(resource_diagnostic_is_reachable(
        &diagnostic("loader/member"),
        &facts,
        &HashSet::new(),
    ));
    assert!(!resource_diagnostic_is_reachable(
        &diagnostic("loader/nested/member"),
        &facts,
        &HashSet::new(),
    ));
    assert!(!resource_diagnostic_is_reachable(
        &diagnostic("loaderSuffix"),
        &facts,
        &HashSet::new(),
    ));
}

#[test]
fn call_scope_resolution_walks_multiple_lexical_parents() {
    let known_scopes = HashSet::from(["grand/target".to_string()]);

    assert_eq!(
        resolve_callee_scope(Some("grand/parent/child"), "target", &known_scopes),
        "grand/target",
    );
}

#[test]
fn callable_alias_resolution_uses_the_callee_binding_scope() {
    let index = CallableFileIndex {
        known_scopes: ["target".to_string()].into_iter().collect(),
        exported_scopes: fx_set(),
        class_scopes: fx_set(),
        callable_bindings: fx_map(),
        imported: fx_map(),
        exported: fx_map(),
        aliases: [(
            (0, "alias".to_string()),
            IndexedAlias {
                target: "target".to_string(),
                declared_at: 0,
                invalidated_at: None,
            },
        )]
        .into_iter()
        .collect(),
        binding_declared_at: fx_map(),
        invocation_offsets: fx_map(),
        class_bindings: fx_map(),
        lexical_scope_parents: [(0, None), (1, Some(0))].into_iter().collect(),
        scope_ids_by_display: fx_map(),
        stars: Vec::new(),
    };

    assert_eq!(
        index
            .resolve_alias(
                Some("outer/inner"),
                Some(0),
                "alias",
                0,
                None,
                InvocationKind::Call,
            )
            .map(|resolved| resolved.callee),
        Some("target".to_string()),
    );
    assert_eq!(
        index.resolve_alias(
            Some("outer/inner"),
            Some(1),
            "alias",
            0,
            None,
            InvocationKind::Call,
        ),
        None,
        "a block-local shadow must not resolve an outer alias",
    );
}

#[test]
fn callable_alias_resolution_reaches_module_scope_from_outermost_function() {
    use crate::codebase::dependencies::extract::{CallableAlias, FunctionCall};

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_aliases: vec![CallableAlias {
            scope: None,
            scope_id: None,
            local: "moduleAlias".to_string(),
            target: "target".to_string(),
            binding_scope: 0,
            declared_at: 0,
            invalidated_at: None,
        }],
        function_calls: vec![
            FunctionCall {
                caller: None,
                caller_id: None,
                syntactic_caller: None,
                callee: "run".to_string(),
                line: 1,
                offset: 0,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: CallTargetIdentity::RepositoryFunction,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
            FunctionCall {
                caller: Some("run".to_string()),
                caller_id: Some(crate::codebase::dependencies::extract::CallableId(1)),
                syntactic_caller: Some("run".to_string()),
                callee: "moduleAlias".to_string(),
                line: 2,
                offset: 1,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: CallTargetIdentity::Unknown,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
        ],
        callable_scopes: vec!["run".to_string(), "target".to_string()],
        callable_scope_ids: vec![
            (
                crate::codebase::dependencies::extract::CallableId(1),
                "run".to_string(),
            ),
            (
                crate::codebase::dependencies::extract::CallableId(2),
                "target".to_string(),
            ),
        ],
        callable_bindings: vec![],
        ..Default::default()
    };

    let index = CallableFileIndex::from_facts(&facts);
    assert_eq!(
        index
            .resolve_alias(
                facts.function_calls[1].caller.as_deref(),
                facts.function_calls[1].callee_binding_scope,
                &facts.function_calls[1].callee,
                facts.function_calls[1].offset,
                facts.function_calls[1].caller_id,
                facts.function_calls[1].invocation,
            )
            .map(|resolved| resolved.callee),
        Some("target".to_string()),
    );
    assert!(
        reachable_function_scopes(&facts)
            .contains(&crate::codebase::dependencies::extract::CallableId(2))
    );
}

#[test]
fn callable_alias_lookup_does_not_cross_a_shadowed_parameter() {
    use crate::codebase::dependencies::extract::FunctionCall;

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        function_calls: vec![
            FunctionCall {
                caller: None,
                caller_id: None,
                syntactic_caller: None,
                callee: "run".to_string(),
                line: 1,
                offset: 0,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: CallTargetIdentity::RepositoryFunction,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
            FunctionCall {
                caller: Some("run".to_string()),
                caller_id: Some(crate::codebase::dependencies::extract::CallableId(1)),
                syntactic_caller: Some("run".to_string()),
                callee: "target".to_string(),
                line: 2,
                offset: 1,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: CallTargetIdentity::Unknown,
                callee_binding_scope: Some(1),
                static_arg: None,
                static_cwd: None,
            },
        ],
        callable_scopes: vec!["run".to_string(), "target".to_string()],
        callable_scope_ids: vec![
            (
                crate::codebase::dependencies::extract::CallableId(1),
                "run".to_string(),
            ),
            (
                crate::codebase::dependencies::extract::CallableId(2),
                "target".to_string(),
            ),
        ],
        callable_bindings: vec![],
        ..Default::default()
    };

    let reachable = reachable_function_scopes(&facts);
    assert!(reachable.contains(&crate::codebase::dependencies::extract::CallableId(1)));
    assert!(!reachable.contains(&crate::codebase::dependencies::extract::CallableId(2)));
}

#[test]
fn constructor_membership_is_reachable_but_other_member_membership_is_not() {
    use crate::codebase::dependencies::extract::FunctionCall;

    let membership = |callee: &str| FunctionCall {
        caller: Some("Service".to_string()),
        caller_id: Some(crate::codebase::dependencies::extract::CallableId(1)),
        syntactic_caller: None,
        callee: callee.to_string(),
        line: 0,
        offset: 0,
        is_callback: true,
        invocation: InvocationKind::Membership,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    };
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        function_calls: vec![
            FunctionCall {
                caller: None,
                caller_id: None,
                syntactic_caller: None,
                callee: "Service".to_string(),
                line: 1,
                offset: 0,
                is_callback: false,
                invocation: InvocationKind::Construct,
                target_identity: CallTargetIdentity::RepositoryFunction,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
            membership("constructor"),
            membership("unused"),
        ],
        callable_scopes: vec![
            "Service".to_string(),
            "Service/constructor".to_string(),
            "Service/unused".to_string(),
        ],
        callable_scope_ids: vec![
            (
                crate::codebase::dependencies::extract::CallableId(1),
                "Service".to_string(),
            ),
            (
                crate::codebase::dependencies::extract::CallableId(2),
                "Service/constructor".to_string(),
            ),
            (
                crate::codebase::dependencies::extract::CallableId(3),
                "Service/unused".to_string(),
            ),
        ],
        callable_bindings: vec![],
        ..Default::default()
    };

    let reachable = reachable_function_scopes(&facts);
    assert!(reachable.contains(&crate::codebase::dependencies::extract::CallableId(2)));
    assert!(!reachable.contains(&crate::codebase::dependencies::extract::CallableId(3)));
}

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

#[test]
fn callable_alias_resolution_is_indexed_once_per_file() {
    let source = include_str!("../edge_calls.rs");
    let helpers = include_str!("../edge_calls/index_build.rs");
    let reachability = include_str!("../edge_import_reachability_traversal.rs");
    let deferred = include_str!("../../extract_collector_deferred_aliases.rs");

    assert!(
        helpers.contains("fn index_callable_aliases("),
        "aliases must be grouped once by lexical binding identity",
    );
    assert!(
        source.contains("aliases: index_callable_aliases(&file.callable_aliases)"),
        "from_facts must assemble the alias map in one pass",
    );
    assert!(
        reachability.contains("CallableFileIndex::from_facts(facts)"),
        "import reachability must reuse the per-file alias index",
    );
    assert!(
        reachability.contains("index.resolve_alias("),
        "import reachability must share resolve_alias with call edges",
    );
    assert!(
        !reachability.contains("callable_aliases.iter()"),
        "import reachability must not linear-scan callable_aliases",
    );
    assert!(
        !deferred.contains("callable_aliases.iter()"),
        "deferred alias finalization must look up the binding index",
    );
}

#[test]
fn callable_file_index_uses_fx_hash_for_interned_keys() {
    let sources = [
        include_str!("../edge_calls.rs"),
        include_str!("../edge_calls/index_build.rs"),
        include_str!("../edge_calls/liveness.rs"),
        include_str!("../edge_calls/collection.rs"),
        include_str!("../edge_calls/local_resolution.rs"),
        include_str!("../edge_calls/class_resolution.rs"),
        include_str!("../edge_calls/this_resolution.rs"),
        include_str!("../edge_calls/alias_resolution.rs"),
        include_str!("../edge_calls/export_resolution_population.rs"),
        include_str!("../edge_calls/roots.rs"),
    ];
    for source in sources {
        assert!(
            !source.contains("std::collections::HashMap"),
            "callable indexes must use crate::fx maps, not SipHash HashMap"
        );
        assert!(
            !source.contains("std::collections::HashSet"),
            "callable indexes must use crate::fx sets, not SipHash HashSet"
        );
        assert!(
            !source.contains("HashMap::new()"),
            "rustc-hash 2 FxHashMap has no new(); use fx_map()"
        );
        assert!(
            !source.contains("HashSet::new()"),
            "rustc-hash 2 FxHashSet has no new(); use fx_set()"
        );
    }
}
