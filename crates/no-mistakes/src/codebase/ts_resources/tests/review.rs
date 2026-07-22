use super::*;

#[test]
fn preserves_arrow_hoisting_assignment_invalidation_and_nested_object_scopes() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-review-regressions.ts"
    ));

    assert!(facts.diagnostics.is_empty());
    assert_eq!(facts.calls.len(), 2);
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
}
