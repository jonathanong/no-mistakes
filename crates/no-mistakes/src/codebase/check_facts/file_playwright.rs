use super::{PlaywrightFactPlan, PlaywrightTestFacts};
use oxc_ast::ast::Program;
use std::path::Path;

pub(crate) fn collect_playwright_facts(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    plan: Option<&PlaywrightFactPlan>,
) -> Option<PlaywrightTestFacts> {
    let plan = plan?;
    let test_id_attributes = plan.test_id_attributes_by_path.get(path)?;
    Some(PlaywrightTestFacts {
        urls: crate::playwright::playwright_urls::extract_playwright_url_occurrences_from_program(
            program,
            source,
            &plan.navigation_helpers,
        ),
        selectors:
            crate::playwright::selectors::extract_playwright_selector_occurrences_from_program(
                program,
                source,
                &plan.selector_regexes,
                test_id_attributes,
            ),
        text_locators:
            crate::playwright::selectors::extract_playwright_text_locator_occurrences_from_program(
                program, source,
            ),
        helper_references:
            crate::playwright::selectors::extract_playwright_helper_reference_occurrences_from_program(
                program, source,
            ),
    })
}
