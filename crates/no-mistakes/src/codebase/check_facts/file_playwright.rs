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
    semantic: &Option<oxc_semantic::SemanticBuilderReturn<'_>>,
) -> Option<PlaywrightTestFacts> {
    let fact_plan = plan?;
    let plan = fact_plan.file(path)?;
    let static_templates = crate::playwright::playwright_urls::static_templates_from_semantic(
        &semantic
            .as_ref()
            .expect("prepared Playwright binding projection")
            .semantic,
    );
    let common = Arc::new(CommonOccurrences {
        text_locators:
            crate::playwright::selectors::extract_playwright_text_locator_occurrences_from_program(
                program, source,
            ),
    });
    let variants = plan
        .variants()
        .map(|(key, variant)| {
            let (selectors, wrapper_calls) = crate::playwright::selectors::extract_playwright_selector_occurrences_and_wrapper_calls_from_program(
                program,
                source,
                &variant.selector_regexes,
                &key.test_id_attributes,
                &key.selector_wrappers,
                Some(path),
                fact_plan.module_resolution(),
            );
            let occurrences = VariantOccurrences {
                urls: crate::playwright::playwright_urls::extract_with_static_templates(
                    program,
                    source,
                    &key.navigation_helpers,
                    &static_templates,
                ),
                selectors,
                helper_references: crate::playwright::selectors::extract_playwright_helper_reference_occurrences_from_program(
                    program,
                    source,
                    &wrapper_calls,
                ),
            };
            (key.clone(), Arc::new(occurrences))
        })
        .collect::<BTreeMap<_, _>>();
    Some(PlaywrightTestFacts::new(common, variants))
}
