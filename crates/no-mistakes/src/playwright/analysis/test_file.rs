use crate::playwright::analysis::context::{DiscoveredTestFile, TestAnalysisContext};
use crate::playwright::analysis::routes_index::route_specificity;
use crate::playwright::analysis::text_edges::append_locator_text_edges;
use crate::playwright::analysis::types::{
    Edge, SelectorHelperReference, SelectorHelperReferenceWithValue, TestFileAnalysis,
};
use crate::playwright::fsutil::relative_string;
use crate::playwright::matcher;
use crate::playwright::selectors;
use crate::playwright::url::normalize_url;
use crate::playwright::{ast, playwright_urls};
use anyhow::{Context, Result};

pub(crate) fn analyze_test_file(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
) -> Result<TestFileAnalysis> {
    let source = std::fs::read_to_string(&test_file.path)
        .context(format!("reading test file {}", test_file.path.display()))?;
    let test_id_attributes = test_file.test_id_attributes();

    let parsed = ast::with_program(&test_file.path, &source, |program, source| {
        let raw_urls = playwright_urls::extract_playwright_url_occurrences_from_program(
            program,
            source,
            context.navigation_helpers,
        );
        let playwright_selectors = if context.app_selector_targets.is_empty() {
            Vec::new()
        } else {
            selectors::extract_playwright_selector_occurrences_from_program(
                program,
                source,
                context.selector_regexes,
                &test_id_attributes,
            )
        };
        let text_locators = if context.app_text_targets.is_empty() {
            Vec::new()
        } else {
            selectors::extract_playwright_text_locator_occurrences_from_program(program, source)
        };
        let helper_references =
            selectors::extract_playwright_helper_reference_occurrences_from_program(
                program, source,
            );
        (
            raw_urls,
            playwright_selectors,
            text_locators,
            helper_references,
        )
    });
    let (raw_urls, playwright_selectors, text_locators, helper_references) = parsed?;
    Ok(analyze_test_occurrences(
        test_file,
        context,
        raw_urls,
        playwright_selectors,
        text_locators,
        helper_references,
    ))
}

pub(crate) fn analyze_test_occurrences(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
    raw_urls: Vec<crate::playwright::playwright_tests::TestOccurrence<String>>,
    playwright_selectors: Vec<
        crate::playwright::playwright_tests::TestOccurrence<selectors::PlaywrightSelector>,
    >,
    text_locators: Vec<
        crate::playwright::playwright_tests::TestOccurrence<
            crate::playwright::analysis::text_types::PlaywrightTextLocator,
        >,
    >,
    helper_references: Vec<
        crate::playwright::playwright_tests::TestOccurrence<selectors::PlaywrightHelperReference>,
    >,
) -> TestFileAnalysis {
    let rel_test_file = std::sync::Arc::new(relative_string(context.root, &test_file.path));
    let mut edges = Vec::new();
    let base_urls = test_file.base_urls();
    for raw_url in raw_urls {
        if !context.test_policy.allows(raw_url.status) {
            continue;
        }
        if raw_url.scope == crate::playwright::playwright_tests::TestOccurrenceScope::TeardownHook {
            continue;
        }
        let Some(url) = normalize_url(&raw_url.value, &base_urls) else {
            continue;
        };
        let ref_segments = matcher::reference_segments(&url);
        let matching_routes: Vec<_> = context
            .route_index
            .candidates(&ref_segments)
            .into_iter()
            .filter(|route| matcher::matches_segments(&ref_segments, &route.segments))
            .collect();
        let Some(best_specificity) = matching_routes
            .iter()
            .map(|route| route_specificity(&route.segments))
            .max()
        else {
            continue;
        };

        let test_name_arc = raw_url.test_name.map(std::sync::Arc::new);
        let describe_path_arc = std::sync::Arc::new(raw_url.describe_path);
        let url_arc = std::sync::Arc::new(url);

        for route in matching_routes
            .into_iter()
            .filter(|route| route_specificity(&route.segments) == best_specificity)
        {
            edges.push(Edge::Route {
                test_file: rel_test_file.clone(),
                test_name: test_name_arc.clone(),
                describe_path: describe_path_arc.clone(),
                route_file: route.route_file.clone(),
                route: route.pattern.clone(),
                url: url_arc.clone(),
                hook: raw_url.scope
                    == crate::playwright::playwright_tests::TestOccurrenceScope::Hook,
                line: raw_url.line,
            });
        }
    }

    if !context.app_selector_targets.is_empty() {
        for playwright_selector in playwright_selectors {
            if !context.test_policy.allows(playwright_selector.status) {
                continue;
            }
            if playwright_selector.scope
                == crate::playwright::playwright_tests::TestOccurrenceScope::TeardownHook
            {
                continue;
            }

            let test_name_arc = playwright_selector.test_name.map(std::sync::Arc::new);
            let describe_path_arc = std::sync::Arc::new(playwright_selector.describe_path);

            for app_selector in context.selector_index.matches(&playwright_selector.value) {
                edges.push(Edge::Selector {
                    test_file: rel_test_file.clone(),
                    test_name: test_name_arc.clone(),
                    describe_path: describe_path_arc.clone(),
                    app_file: app_selector.app_file.clone(),
                    attribute: app_selector.selector.attribute.clone(),
                    value: app_selector.value.clone(),
                    selector: playwright_selector.value.selector.clone(),
                    line: playwright_selector.line,
                });
            }
        }
    }

    append_locator_text_edges(&mut edges, &rel_test_file, context, text_locators);

    let helper_references = helper_references
        .into_iter()
        .filter(|reference| context.test_policy.allows(reference.status))
        .filter(|reference| {
            reference.scope
                != crate::playwright::playwright_tests::TestOccurrenceScope::TeardownHook
        })
        .map(|reference| SelectorHelperReferenceWithValue {
            value: reference.value.value,
            reference: SelectorHelperReference {
                test_file: rel_test_file.clone(),
                line: reference.line,
                call: reference.value.call,
            },
        })
        .collect();

    TestFileAnalysis {
        edges,
        helper_references,
    }
}
