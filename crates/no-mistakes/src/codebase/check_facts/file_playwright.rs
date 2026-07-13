use super::{PlaywrightFactPlan, PlaywrightTestFacts};
use crate::playwright::test_file_occurrences::{CommonOccurrences, VariantOccurrences};
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn collect_playwright_facts(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    plan: Option<&PlaywrightFactPlan>,
) -> Option<PlaywrightTestFacts> {
    let plan = plan?.file(path)?;
    let common = Arc::new(CommonOccurrences {
        text_locators:
            crate::playwright::selectors::extract_playwright_text_locator_occurrences_from_program(
                program, source,
            ),
        helper_references:
            crate::playwright::selectors::extract_playwright_helper_reference_occurrences_from_program(
                program, source,
            ),
    });
    let variants = plan
        .variants()
        .map(|(key, variant)| {
            let occurrences = VariantOccurrences {
                urls: crate::playwright::playwright_urls::extract_playwright_url_occurrences_from_program(
                    program,
                    source,
                    &key.navigation_helpers,
                ),
                selectors:
                    crate::playwright::selectors::extract_playwright_selector_occurrences_from_program(
                        program,
                        source,
                        &variant.selector_regexes,
                        &key.test_id_attributes,
                    ),
            };
            (key.clone(), Arc::new(occurrences))
        })
        .collect::<BTreeMap<_, _>>();
    Some(PlaywrightTestFacts::new(common, variants))
}
