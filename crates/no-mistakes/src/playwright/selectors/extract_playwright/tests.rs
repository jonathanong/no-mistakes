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

#[test]
fn helper_references_skip_counted_locator_calls() {
    let source = r#"
import { test } from '@playwright/test';

function getAsideLocator(page, dataPw) {
  return page.getByTestId(dataPw).first();
}

test('uses helper', async ({ page }) => {
  await page.getByTestId('direct-button').click();
  await getAsideLocator(page, 'helper-button').click();
});
"#;

    let references = ast::with_program(
        std::path::Path::new("helper.ts"),
        source,
        |program, source| {
            extract_playwright_helper_reference_occurrences_from_program(program, source)
        },
    )
    .expect("fixture should parse");

    assert!(references.iter().any(|reference| {
        reference.value.value == "helper-button"
            && reference.value.call == "getAsideLocator(...)"
            && reference.scope == playwright_tests::TestOccurrenceScope::Test
    }));
    assert!(
        references
            .iter()
            .all(|reference| reference.value.value != "direct-button"),
        "direct getByTestId coverage calls must not be helper-reference hints: {references:?}"
    );
}
