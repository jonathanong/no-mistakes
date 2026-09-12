use crate::codebase::dependencies::extract::{CallableId, FunctionCall};

fn construct_base(caller: &str, caller_id: u32, base: &str, scope: usize) -> FunctionCall {
    FunctionCall {
        caller: Some(caller.to_string()),
        caller_id: Some(CallableId(caller_id)),
        syntactic_caller: None,
        callee: base.to_string(),
        line: 1,
        offset: 0,
        is_callback: true,
        invocation: InvocationKind::Construct,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: Some(scope),
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn inherited_static_members_walk_parent_lexical_scopes_and_cycles() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_scopes: vec![
            "Base".to_string(),
            "Child".to_string(),
            "A".to_string(),
            "B".to_string(),
        ],
        class_scopes: vec![
            "Base".to_string(),
            "Child".to_string(),
            "A".to_string(),
            "B".to_string(),
        ],
        callable_scope_ids: vec![
            (CallableId(1), "Base".to_string()),
            (CallableId(2), "Child".to_string()),
            (CallableId(3), "A".to_string()),
            (CallableId(4), "B".to_string()),
        ],
        callable_bindings: vec![
            (0, "Base".to_string(), CallableId(1)),
            (1, "Child".to_string(), CallableId(2)),
            (0, "A".to_string(), CallableId(3)),
            (0, "B".to_string(), CallableId(4)),
        ],
        class_member_callable_ids: vec![(CallableId(1), "run".to_string(), CallableId(11))],
        lexical_scope_parents: vec![(0, None), (1, Some(0))],
        function_calls: vec![
            construct_base("Child", 2, "Base", 0),
            construct_base("A", 3, "B", 0),
            construct_base("B", 4, "A", 0),
        ],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);

    assert_eq!(
        index
            .resolve_class_binding(Some(1), "Child.run", InvocationKind::Call)
            .and_then(|resolved| resolved.callable_id),
        Some(CallableId(11)),
        "Child in an inner scope must walk to Base in the parent lexical frame"
    );
    assert!(
        index
            .resolve_class_binding(Some(0), "A.missing", InvocationKind::Call)
            .is_none(),
        "cyclic local bases must stop instead of looping"
    );
}

#[test]
fn alias_resolution_walks_parent_scopes_and_stops_on_cycles() {
    let index = CallableFileIndex {
        known_scopes: ["target".to_string()].into_iter().collect(),
        exported_scopes: fx_set(),
        class_scopes: fx_set(),
        callable_bindings: fx_map(),
        imported: fx_map(),
        exported: fx_map(),
        aliases: [
            (
                (1, "run".to_string()),
                IndexedAlias {
                    target: "alias".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
            (
                (0, "alias".to_string()),
                IndexedAlias {
                    target: "target".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
            (
                (0, "loop.member".to_string()),
                IndexedAlias {
                    target: "loop.member".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
        ]
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
                Some("inner"),
                Some(1),
                "run",
                10,
                None,
                InvocationKind::Call,
            )
            .map(|resolved| resolved.callee),
        Some("target".to_string()),
        "a nested alias must hop through the parent lexical frame"
    );
    assert!(
        index
            .resolve_alias(
                Some("inner"),
                Some(0),
                "loop.member",
                10,
                None,
                InvocationKind::Call,
            )
            .is_none(),
        "a self-alias must not recurse forever"
    );
}

#[test]
fn non_dotted_alias_resolution_exhausts_parents_and_stops_on_cycles() {
    let index = CallableFileIndex {
        known_scopes: fx_set(),
        exported_scopes: fx_set(),
        class_scopes: fx_set(),
        callable_bindings: fx_map(),
        imported: fx_map(),
        exported: fx_map(),
        aliases: [
            (
                (1, "run".to_string()),
                IndexedAlias {
                    target: "ghost".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
            (
                (0, "cycleA".to_string()),
                IndexedAlias {
                    target: "cycleB".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
            (
                (0, "cycleB".to_string()),
                IndexedAlias {
                    target: "cycleA".to_string(),
                    declared_at: 0,
                    invalidated_at: None,
                },
            ),
        ]
        .into_iter()
        .collect(),
        binding_declared_at: fx_map(),
        invocation_offsets: fx_map(),
        class_bindings: fx_map(),
        lexical_scope_parents: [(0, None), (1, Some(0))].into_iter().collect(),
        scope_ids_by_display: fx_map(),
        stars: Vec::new(),
    };

    assert!(
        index
            .resolve_alias(
                Some("inner"),
                Some(1),
                "run",
                10,
                None,
                InvocationKind::Call,
            )
            .is_none(),
        "an alias to an unknown binding must walk parents until the chain ends"
    );
    assert!(
        index
            .resolve_alias(
                Some("inner"),
                Some(0),
                "cycleA",
                10,
                None,
                InvocationKind::Call,
            )
            .is_none(),
        "a non-dotted alias cycle must stop instead of looping"
    );
}

#[test]
fn this_member_resolution_stops_on_duplicate_class_ids_and_instance_ids() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_scopes: vec![
            "Service".to_string(),
            "Service/method".to_string(),
            "Cycle".to_string(),
            "Cycle/method".to_string(),
            "Twin".to_string(),
        ],
        class_scopes: vec![
            "Service".to_string(),
            "Cycle".to_string(),
            "Twin".to_string(),
        ],
        callable_scope_ids: vec![
            (CallableId(1), "Service".to_string()),
            (CallableId(11), "Service/method".to_string()),
            (CallableId(12), "Service/method".to_string()),
            (CallableId(2), "Cycle".to_string()),
            (CallableId(21), "Cycle/method".to_string()),
            (CallableId(3), "Twin".to_string()),
            (CallableId(4), "Twin".to_string()),
        ],
        callable_bindings: vec![
            (0, "Service".to_string(), CallableId(1)),
            (0, "Cycle".to_string(), CallableId(2)),
            (0, "Twin".to_string(), CallableId(3)),
            (1, "Twin".to_string(), CallableId(4)),
        ],
        class_member_callable_ids: vec![(CallableId(1), "method".to_string(), CallableId(11))],
        lexical_scope_parents: vec![(0, None), (1, Some(0))],
        function_calls: vec![construct_base("Cycle", 2, "Cycle", 0)],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);

    assert!(
        index
            .resolve_this_member(
                Some("Cycle/method"),
                Some(CallableId(21)),
                "this.missing",
                InvocationKind::Call,
            )
            .is_none(),
        "a cyclic local base must not walk forever"
    );
    assert!(
        index
            .resolve_this_member(
                Some("Service/method"),
                Some(CallableId(11)),
                "this.method",
                InvocationKind::Call,
            )
            .is_some()
            || index
                .resolve_this_member(
                    Some("Service/nested/method"),
                    Some(CallableId(12)),
                    "this.method",
                    InvocationKind::Call,
                )
                .is_none()
            || index.unique_class_binding_named("Twin").is_none(),
        "duplicate Twin bindings and duplicate instance ids must stay defined"
    );
}

#[test]
fn binding_liveness_walks_nested_parents_and_skips_synthetic_callbacks() {
    let parents: crate::fx::FxHashMap<usize, Option<usize>> =
        [(0, None), (1, Some(0)), (2, Some(1))]
            .into_iter()
            .collect();
    assert!(lexical_scope_is_nested_in(&parents, Some(2), 0));
    assert!(!lexical_scope_is_nested_in(&parents, Some(0), 1));
    assert!(!lexical_scope_is_nested_in(&parents, None, 0));

    let bindings = fx_map();
    let calls = [
        FunctionCall {
            caller: None,
            caller_id: None,
            syntactic_caller: None,
            callee: "fn".to_string(),
            line: 1,
            offset: 4,
            is_callback: true,
            invocation: InvocationKind::Construct,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: Some(0),
            static_arg: None,
            static_cwd: None,
        },
        FunctionCall {
            caller: None,
            caller_id: None,
            syntactic_caller: None,
            callee: "fn".to_string(),
            line: 1,
            offset: 8,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: None,
            static_arg: None,
            static_cwd: None,
        },
    ];
    assert!(invocation_offsets_from_bindings(&bindings, &calls).is_empty());

    let live = binding_live_at(BindingLivenessQuery {
        declared_at: 20,
        invalidated_at: Some(5),
        call_offset: 8,
        binding_scope: 0,
        call_binding_scope: Some(2),
        caller_id: Some(CallableId(1)),
        invocation_offsets: &fx_map(),
        lexical_parents: &parents,
    });
    assert!(!live);
}
