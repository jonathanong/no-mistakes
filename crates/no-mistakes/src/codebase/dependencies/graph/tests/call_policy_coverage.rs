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
fn callable_alias_resolution_reaches_module_scope_from_nested_callers() {
    let index = CallableFileIndex {
        known_scopes: HashSet::from(["target".to_string()]),
        imported: HashMap::new(),
        exported: HashMap::new(),
        aliases: HashMap::from([((None, "alias".to_string()), "target".to_string())]),
        stars: Vec::new(),
    };

    assert_eq!(
        index.resolve_alias(Some("outer/inner"), "alias"),
        Some("target".to_string()),
    );
}
