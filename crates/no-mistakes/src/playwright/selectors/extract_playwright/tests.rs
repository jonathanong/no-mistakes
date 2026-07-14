use super::*;
use crate::playwright::ast;

#[test]
fn selector_occurrences_preserve_file_test_and_hook_scope() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "extract-playwright",
        "scope.ts",
    ]);
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-testid".to_string()],
        &Default::default(),
    );

    let occurrences = ast::with_program(
        std::path::Path::new("scope.ts"),
        &source,
        |program, source| {
            extract_playwright_selector_occurrences_from_program(
                program,
                source,
                &regexes,
                &["data-testid".to_string()],
                &[],
            )
        },
    )
    .expect("fixture should parse");

    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(file-scope)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::File
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(setup)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Hook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(inside-test)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.as_deref() == Some("active")
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(teardown)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::TeardownHook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(dynamic-test)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.is_none()
    }));
}
