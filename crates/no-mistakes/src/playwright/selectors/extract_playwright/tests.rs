use super::*;
use crate::playwright::ast;
use std::path::Path;

#[test]
fn selector_occurrences_preserve_file_test_and_hook_scope() {
    let source = r#"
        await page.getByTestId("file-scope");
        test.beforeEach(async ({ page }) => {
            await page.getByTestId("setup");
        });
        test("active", async ({ page }) => {
            await page.getByTestId("inside-test");
        });
    "#;
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-testid".to_string()],
        &Default::default(),
    );

    let occurrences = ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        extract_playwright_selector_occurrences_from_program(
            program,
            source,
            &regexes,
            &["data-testid".to_string()],
        )
    })
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
}
