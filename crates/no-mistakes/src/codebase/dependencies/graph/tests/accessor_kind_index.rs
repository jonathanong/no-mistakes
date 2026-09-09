#[test]
fn own_static_accessor_descriptor_blocks_inherited_kind() {
    use crate::codebase::dependencies::extract::{CallableId, FunctionCall};

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_scopes: vec![
            "Base".to_string(),
            "Derived".to_string(),
            "GetterOnly".to_string(),
        ],
        class_scopes: vec![
            "Base".to_string(),
            "Derived".to_string(),
            "GetterOnly".to_string(),
        ],
        callable_scope_ids: vec![
            (CallableId(1), "Base".to_string()),
            (CallableId(2), "Derived".to_string()),
            (CallableId(3), "GetterOnly".to_string()),
        ],
        callable_bindings: vec![
            (0, "Base".to_string(), CallableId(1)),
            (0, "Derived".to_string(), CallableId(2)),
            (0, "GetterOnly".to_string(), CallableId(3)),
        ],
        static_getter_callable_ids: vec![
            (CallableId(1), "value".to_string(), CallableId(11)),
            (CallableId(3), "value".to_string(), CallableId(31)),
        ],
        static_setter_callable_ids: vec![
            (CallableId(1), "value".to_string(), CallableId(12)),
            (CallableId(2), "value".to_string(), CallableId(22)),
        ],
        function_calls: vec![
            FunctionCall {
                caller: Some("Derived".to_string()),
                caller_id: Some(CallableId(2)),
                syntactic_caller: None,
                callee: "Base".to_string(),
                line: 1,
                offset: 0,
                is_callback: true,
                invocation: InvocationKind::Construct,
                target_identity: CallTargetIdentity::RepositoryFunction,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
            FunctionCall {
                caller: Some("GetterOnly".to_string()),
                caller_id: Some(CallableId(3)),
                syntactic_caller: None,
                callee: "Base".to_string(),
                line: 1,
                offset: 0,
                is_callback: true,
                invocation: InvocationKind::Construct,
                target_identity: CallTargetIdentity::RepositoryFunction,
                callee_binding_scope: Some(0),
                static_arg: None,
                static_cwd: None,
            },
        ],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);

    assert!(
        index
            .resolve_class_binding(Some(0), "Derived.value", InvocationKind::Get)
            .is_none(),
        "an own setter descriptor must not inherit the base getter"
    );
    assert_eq!(
        index
            .resolve_class_binding(Some(0), "Derived.value", InvocationKind::Set)
            .and_then(|resolved| resolved.callable_id),
        Some(CallableId(22))
    );
    assert_eq!(
        index
            .resolve_class_binding(Some(0), "Base.value", InvocationKind::Get)
            .and_then(|resolved| resolved.callable_id),
        Some(CallableId(11))
    );
    assert!(
        index
            .resolve_class_binding(Some(0), "GetterOnly.value", InvocationKind::Set)
            .is_none(),
        "an own getter descriptor must not inherit the base setter"
    );
}
