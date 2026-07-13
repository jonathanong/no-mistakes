use super::context::{DiscoveredTestFile, TestAnalysisContext};
use super::routes_index::route_specificity;
use super::types::{
    Edge, SelectorHelperReference, SelectorHelperReferenceWithValue, TestFileAnalysis,
};
use crate::playwright::fsutil::relative_string;
use crate::playwright::matcher;
use crate::playwright::playwright_tests::{TestOccurrence, TestOccurrenceScope};
use crate::playwright::test_file_occurrences::TestFileOccurrences;
use crate::playwright::url::normalize_url;
use std::sync::Arc;

pub(crate) fn analyze_prepared_test_occurrences(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
    occurrences: &TestFileOccurrences,
) -> TestFileAnalysis {
    // Legacy route-graph callers only consume direct edges. The full pipeline
    // appends text edges later with its one shared app-text index.
    analyze_direct_test_occurrences(test_file, context, occurrences)
}

pub(crate) fn analyze_direct_test_occurrences(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
    occurrences: &TestFileOccurrences,
) -> TestFileAnalysis {
    let rel_test_file = Arc::new(relative_string(context.root, &test_file.path));
    let mut edges = route_edges(test_file, context, &rel_test_file, occurrences.urls());
    edges.extend(selector_edges(
        context,
        &rel_test_file,
        occurrences.selectors(),
    ));
    let test_id_attributes = test_file.test_id_attributes();
    let helper_references = occurrences
        .helper_references()
        .iter()
        .filter(|reference| context.test_policy.allows(reference.status))
        .filter(|reference| reference.scope != TestOccurrenceScope::TeardownHook)
        .flat_map(|reference| {
            let value = reference.value.value.clone();
            let helper_reference = SelectorHelperReference {
                test_file: rel_test_file.clone(),
                line: reference.line,
                call: reference.value.call.clone(),
            };
            test_id_attributes.iter().cloned().map(move |attribute| {
                SelectorHelperReferenceWithValue {
                    attribute,
                    value: value.clone(),
                    reference: helper_reference.clone(),
                }
            })
        })
        .collect();
    TestFileAnalysis {
        edges,
        helper_references,
    }
}

fn route_edges(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
    rel_test_file: &Arc<String>,
    urls: &[TestOccurrence<String>],
) -> Vec<Edge> {
    let base_urls = test_file.base_urls();
    let mut edges = Vec::new();
    for raw_url in urls {
        if !context.test_policy.allows(raw_url.status)
            || raw_url.scope == TestOccurrenceScope::TeardownHook
        {
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
        let test_name = raw_url.test_name.clone().map(Arc::new);
        let describe_path = Arc::new(raw_url.describe_path.clone());
        let url = Arc::new(url);
        edges.extend(
            matching_routes
                .into_iter()
                .filter(|route| route_specificity(&route.segments) == best_specificity)
                .map(|route| Edge::Route {
                    test_file: rel_test_file.clone(),
                    test_name: test_name.clone(),
                    describe_path: describe_path.clone(),
                    route_file: route.route_file.clone(),
                    route: route.pattern.clone(),
                    url: url.clone(),
                    hook: raw_url.scope == TestOccurrenceScope::Hook,
                    line: raw_url.line,
                }),
        );
    }
    edges
}

fn selector_edges(
    context: &TestAnalysisContext<'_>,
    rel_test_file: &Arc<String>,
    selectors: &[TestOccurrence<crate::playwright::selectors::PlaywrightSelector>],
) -> Vec<Edge> {
    let mut edges = Vec::new();
    for selector in selectors {
        if !context.test_policy.allows(selector.status)
            || selector.scope == TestOccurrenceScope::TeardownHook
        {
            continue;
        }
        let test_name = selector.test_name.clone().map(Arc::new);
        let describe_path = Arc::new(selector.describe_path.clone());
        edges.extend(
            context
                .selector_index
                .matches(&selector.value)
                .into_iter()
                .map(|app_selector| Edge::Selector {
                    test_file: rel_test_file.clone(),
                    test_name: test_name.clone(),
                    describe_path: describe_path.clone(),
                    app_file: app_selector.app_file.clone(),
                    attribute: app_selector.selector.attribute.clone(),
                    value: app_selector.value.clone(),
                    selector: selector.value.selector.clone(),
                    line: selector.line,
                }),
        );
    }
    edges
}
