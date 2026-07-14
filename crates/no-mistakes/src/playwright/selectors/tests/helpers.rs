use crate::playwright::ast;
use crate::playwright::playwright_tests::TestStatus;
use crate::playwright::selectors::{
    compile_selector_regexes, extract_playwright_selector_occurrences_from_program,
    extract_playwright_text_locator_occurrences_from_program, PlaywrightSelector, SelectorRegexes,
};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) type TextLocatorOccurrence = (
    String,
    String,
    Option<String>,
    TestStatus,
    Option<String>,
    Vec<String>,
);

pub(super) fn extract_playwright_selectors(
    source: &str,
    selector_attributes: &[String],
    test_id_attributes: &[String],
) -> Vec<PlaywrightSelector> {
    let regexes = compile_selector_regexes(selector_attributes, &BTreeMap::new());
    extract_playwright_selectors_with_regexes(
        Path::new("fixture.ts"),
        source,
        &regexes,
        test_id_attributes,
    )
    .expect("fixture should parse")
}

pub(super) fn extract_playwright_selectors_with_regexes(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
    test_id_attributes: &[String],
) -> anyhow::Result<Vec<PlaywrightSelector>> {
    ast::with_program(path, source, |program, source| {
        extract_playwright_selector_occurrences_from_program(
            program,
            source,
            regexes,
            test_id_attributes,
            &[],
        )
        .into_iter()
        .map(|o| o.value)
        .collect()
    })
}

pub(super) fn extract_playwright_selector_occurrences(
    source: &str,
    selector_attributes: &[String],
    test_id_attributes: &[String],
) -> Vec<(String, TestStatus)> {
    let regexes = compile_selector_regexes(selector_attributes, &BTreeMap::new());
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        let mut selectors = Vec::new();
        for occurrence in extract_playwright_selector_occurrences_from_program(
            program,
            source,
            &regexes,
            test_id_attributes,
            &[],
        ) {
            let selector = (occurrence.value.selector, occurrence.status);
            if !selectors.contains(&selector) {
                selectors.push(selector);
            }
        }
        selectors
    })
    .expect("fixture should parse")
}

pub(super) fn extract_playwright_text_locators(
    source: &str,
) -> Vec<(String, String, Option<String>)> {
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        let mut locators = Vec::new();
        for occurrence in extract_playwright_text_locator_occurrences_from_program(program, source)
        {
            let locator = (
                occurrence.value.kind.as_str().to_string(),
                occurrence.value.text,
                occurrence.value.role,
            );
            if !locators.contains(&locator) {
                locators.push(locator);
            }
        }
        locators
    })
    .expect("fixture should parse")
}

pub(super) fn extract_playwright_text_locator_occurrences(
    source: &str,
) -> Vec<TextLocatorOccurrence> {
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        extract_playwright_text_locator_occurrences_from_program(program, source)
            .into_iter()
            .map(|o| {
                (
                    o.value.kind.as_str().to_string(),
                    o.value.text,
                    o.value.role,
                    o.status,
                    o.test_name,
                    o.describe_path,
                )
            })
            .collect()
    })
    .expect("fixture should parse")
}
