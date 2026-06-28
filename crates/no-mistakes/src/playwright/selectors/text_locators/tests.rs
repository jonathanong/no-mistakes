use super::*;
use crate::playwright::analysis::text_types::LocatorKind;
use crate::playwright::ast;
use std::path::Path;

#[test]
fn text_locators_preserve_hook_scope() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "playwright-text-locator-scope.ts",
    ]);

    let occurrences = ast::with_program(Path::new("fixture.ts"), &source, |program, source| {
        extract_playwright_text_locator_occurrences_from_program(program, source)
    })
    .expect("fixture should parse");

    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Setup text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Hook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Suite setup text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Hook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Test text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.as_deref() == Some("uses setup")
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Teardown text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::TeardownHook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.text == "Dynamic test text"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.is_none()
    }));
}

#[test]
fn text_locators_extract_alt_text_and_title() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "playwright-text-locators-branches.ts",
    ]);

    let occurrences = ast::with_program(Path::new("fixture.ts"), &source, |program, source| {
        extract_playwright_text_locator_occurrences_from_program(program, source)
    })
    .expect("fixture should parse");

    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.kind == LocatorKind::Alt && occurrence.value.text == "Company logo"
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.kind == LocatorKind::Title && occurrence.value.text == "Only title"
    }));
}
