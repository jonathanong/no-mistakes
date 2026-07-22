use super::*;

#[test]
fn preserves_arrow_hoisting_assignment_invalidation_and_nested_object_scopes() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-review-regressions.ts"
    ));

    assert!(facts.diagnostics.is_empty());
    assert_eq!(facts.calls.len(), 9);
    assert_eq!(facts.calls[0].path.value, "nested.txt");
    assert_eq!(
        facts.calls[0].function_scope.as_deref(),
        Some("api/nested/load")
    );
    assert_eq!(facts.calls[1].path.value, "deep.txt");
    assert_eq!(
        facts.calls[1].function_scope.as_deref(),
        Some("api/nested/deeper/load")
    );
    for eager in [
        "eager-object.txt",
        "eager-nested-object.txt",
        "eager-static-field.txt",
    ] {
        assert_eq!(
            facts
                .calls
                .iter()
                .find(|call| call.path.value == eager)
                .and_then(|call| call.function_scope.as_deref()),
            None,
            "{eager} executes while the module initializes"
        );
    }
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "deferred-class-method.txt")
            .and_then(|call| call.function_scope.as_deref()),
        Some("PrivateCache/load")
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "deferred-static-arrow.txt")
            .and_then(|call| call.function_scope.as_deref()),
        Some("PrivateCache/loadLater")
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "deferred-static-function.txt")
            .and_then(|call| call.function_scope.as_deref()),
        Some("PrivateCache/loadFunction")
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "deferred-instance-field.txt")
            .and_then(|call| call.function_scope.as_deref()),
        Some("PrivateCache")
    );
}

#[test]
fn supports_nested_require_aliases_and_named_url_constructor_bindings() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/binding-regressions-consumer.ts"
    ));
    let paths = facts
        .calls
        .iter()
        .map(|call| (call.path.value.as_str(), call.path.base))
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            (
                "resources/require-promises-alias.txt",
                ResourcePathBase::AnalysisRoot
            ),
            ("./resources/named-url.txt", ResourcePathBase::SourceModule),
            (
                "./resources/named-file-url.txt",
                ResourcePathBase::SourceModule
            ),
        ]
    );
    assert!(!facts
        .calls
        .iter()
        .any(|call| call.path.value.contains("shadowed-url")));
}
