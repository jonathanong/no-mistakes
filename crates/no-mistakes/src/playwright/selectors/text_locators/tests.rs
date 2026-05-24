use super::*;
use crate::playwright::ast;
use std::path::Path;

#[test]
fn text_locators_preserve_hook_scope() {
    let source = r#"
        test.beforeEach(async ({ page }) => {
            await page.getByText("Setup text").click();
        });
        test("uses setup", async ({ page }) => {
            await page.getByText("Test text").click();
        });
    "#;

    let occurrences = ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        extract_playwright_text_locator_occurrences_from_program(program, source)
    })
    .expect("fixture should parse");

    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Setup text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Hook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Test text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.as_deref() == Some("uses setup")
    }));
}
