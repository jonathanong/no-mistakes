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
        known_scopes: HashSet::from(["target".to_string()]),
        imported: HashMap::new(),
        exported: HashMap::new(),
        aliases: HashMap::from([((0, "alias".to_string()), "target".to_string())]),
        stars: Vec::new(),
    };

    assert_eq!(
        index.resolve_alias(Some("outer/inner"), Some(0), "alias"),
        Some("target".to_string()),
    );
    assert_eq!(
        index.resolve_alias(Some("outer/inner"), Some(1), "alias"),
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
            local: "moduleAlias".to_string(),
            target: "target".to_string(),
            binding_scope: 0,
        }],
        function_calls: vec![
            FunctionCall {
                caller: None,
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
        ..Default::default()
    };

    assert_eq!(
        resolve_callable_alias(&facts, Some("run"), "moduleAlias"),
        Some("target".to_string()),
    );
    assert!(reachable_function_scopes(&facts).contains("target"));
}

#[test]
fn callable_alias_lookup_does_not_cross_a_shadowed_parameter() {
    use crate::codebase::dependencies::extract::FunctionCall;

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        function_calls: vec![
            FunctionCall {
                caller: None,
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
        ..Default::default()
    };

    let reachable = reachable_function_scopes(&facts);
    assert!(reachable.contains("run"));
    assert!(!reachable.contains("target"));
}
