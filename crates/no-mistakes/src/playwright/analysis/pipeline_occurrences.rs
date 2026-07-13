use super::context::DiscoveredTestFile;
use crate::codebase::check_facts::PlaywrightOccurrenceKey;
use crate::playwright::playwright_tests::{TestOccurrence, TestOccurrenceScope, TestPolicy};
use crate::playwright::selectors::SelectorRegexes;
use crate::playwright::test_file_occurrences::{
    CommonOccurrences, TestFileOccurrences, VariantOccurrences,
};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::sync::Arc;

pub(crate) struct PreparedTestFile {
    pub(crate) test_file: DiscoveredTestFile,
    pub(crate) occurrences: TestFileOccurrences,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct TestOccurrenceDemand {
    pub(crate) routes: bool,
    pub(crate) text_locators: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum CachedOccurrenceSelection {
    Exact,
}

pub(crate) fn prepare_test_files(
    test_files: Vec<DiscoveredTestFile>,
    settings: &crate::playwright::config::Settings,
    selector_regexes: &SelectorRegexes,
    test_policy: TestPolicy,
    skip_test_file_errors: bool,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
    selection: CachedOccurrenceSelection,
) -> Result<(Vec<PreparedTestFile>, TestOccurrenceDemand)> {
    let prepared: Vec<_> = test_files
        .into_par_iter()
        .map(|test_file| {
            let occurrences =
                match facts.and_then(|facts| facts.get_playwright_facts(&test_file.path)) {
                    Some(playwright) => match selection {
                        CachedOccurrenceSelection::Exact => {
                            let attributes = test_file.test_id_attributes();
                            let key = PlaywrightOccurrenceKey::new(
                                &settings.navigation_helpers,
                                &settings.selector_attributes,
                                &settings.component_selector_attributes,
                                settings.html_ids,
                                &attributes,
                            );
                            vec![playwright.select(&key).ok_or_else(|| {
                                anyhow::anyhow!(
                                    "cached Playwright facts lack the requested variant for {}",
                                    test_file.path.display()
                                )
                            })?]
                        }
                    },
                    None => {
                        if let Some(error) = facts
                            .and_then(|facts| facts.get_playwright_parse_error(&test_file.path))
                        {
                            if skip_test_file_errors {
                                return Ok(None);
                            }
                            return Err(anyhow::Error::msg(error.to_string()));
                        }
                        match extract_test_file_occurrences(
                            &test_file,
                            &settings.navigation_helpers,
                            selector_regexes,
                        ) {
                            Ok(occurrences) => vec![occurrences],
                            Err(_) if skip_test_file_errors => return Ok(None),
                            Err(error) => return Err(error),
                        }
                    }
                };
            Ok(Some(
                occurrences
                    .into_iter()
                    .map(|occurrences| PreparedTestFile {
                        test_file: test_file.clone(),
                        occurrences,
                    })
                    .collect::<Vec<_>>(),
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    let demand = prepared
        .iter()
        .fold(TestOccurrenceDemand::default(), |mut demand, file| {
            demand.routes |= file
                .occurrences
                .urls()
                .iter()
                .any(|occurrence| is_eligible(occurrence, test_policy));
            demand.text_locators |= file
                .occurrences
                .text_locators()
                .iter()
                .any(|occurrence| is_eligible(occurrence, test_policy));
            demand
        });
    Ok((prepared, demand))
}

pub(crate) fn extract_test_file_occurrences(
    test_file: &DiscoveredTestFile,
    navigation_helpers: &[String],
    selector_regexes: &SelectorRegexes,
) -> Result<TestFileOccurrences> {
    let source = std::fs::read_to_string(&test_file.path)
        .context(format!("reading test file {}", test_file.path.display()))?;
    let test_id_attributes = test_file.test_id_attributes();
    crate::playwright::ast::with_program(&test_file.path, &source, |program, source| {
        TestFileOccurrences {
            variant: Arc::new(VariantOccurrences {
                urls: crate::playwright::playwright_urls::extract_playwright_url_occurrences_from_program(
                program,
                source,
                navigation_helpers,
            ),
                selectors: crate::playwright::selectors::extract_playwright_selector_occurrences_from_program(
                program,
                source,
                selector_regexes,
                &test_id_attributes,
            ),
            }),
            common: Arc::new(CommonOccurrences {
                text_locators: crate::playwright::selectors::extract_playwright_text_locator_occurrences_from_program(
                program, source,
            ),
                helper_references: crate::playwright::selectors::extract_playwright_helper_reference_occurrences_from_program(
                program, source,
            ),
            }),
        }
    })
}

fn is_eligible<T>(occurrence: &TestOccurrence<T>, test_policy: TestPolicy) -> bool {
    test_policy.allows(occurrence.status) && occurrence.scope != TestOccurrenceScope::TeardownHook
}
