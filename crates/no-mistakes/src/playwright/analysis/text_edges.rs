use crate::playwright::analysis::context::TestAnalysisContext;
use crate::playwright::analysis::text_types::{AppTextTarget, PlaywrightTextLocator};
use crate::playwright::analysis::types::Edge;
use crate::playwright::playwright_tests::TestOccurrence;
use rayon::prelude::*;
use std::sync::Arc;

mod matching;
use matching::{text_target_matches, TextMatch};
mod route_scope;
use route_scope::route_signal_matches_test;
#[cfg(test)]
mod tests;

struct LocatorTestScope<'a> {
    test_name: &'a Option<Arc<String>>,
    describe_path: &'a Arc<Vec<String>>,
    is_hook: bool,
}

pub(crate) fn append_locator_text_edges(
    edges: &mut Vec<Edge>,
    rel_test_file: &Arc<String>,
    context: &TestAnalysisContext<'_>,
    text_locators: Vec<TestOccurrence<PlaywrightTextLocator>>,
) {
    let existing_edges = edges.as_slice();
    let mut locator_edges = text_locators
        .into_par_iter()
        .flat_map_iter(|text_locator| {
            if !context.test_policy.allows(text_locator.status) {
                return Vec::new();
            }
            let test_name = text_locator.test_name.map(Arc::new);
            let describe_path = Arc::new(text_locator.describe_path);
            let locator_scope = LocatorTestScope {
                test_name: &test_name,
                describe_path: &describe_path,
                is_hook: text_locator.scope
                    == crate::playwright::playwright_tests::TestOccurrenceScope::Hook,
            };
            let text_match = TextMatch::new(&text_locator.value.text, text_locator.value.exact);
            context
                .app_text_targets
                .iter()
                .filter(|target| {
                    text_target_matches(
                        target,
                        &text_locator.value.kind,
                        text_locator.value.role.as_deref(),
                        &text_match,
                        text_locator.value.include_hidden,
                    )
                })
                .filter_map(|app_text| {
                    let reasons = locator_reasons(
                        existing_edges,
                        rel_test_file,
                        &locator_scope,
                        text_locator.line,
                        app_text,
                        context,
                    );
                    if reasons.is_empty() {
                        return None;
                    }
                    Some(Edge::LocatorText {
                        test_file: rel_test_file.clone(),
                        test_name: test_name.clone(),
                        describe_path: describe_path.clone(),
                        app_file: app_text.app_file.clone(),
                        locator_kind: text_locator.value.kind.as_str().to_string(),
                        role: text_locator.value.role.clone(),
                        text: text_locator.value.text.clone(),
                        locator: text_locator.value.locator.clone(),
                        selector_refs: app_text.selector_refs.clone(),
                        reasons,
                        line: text_locator.line,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    locator_edges.sort();
    edges.extend(locator_edges);
}

fn locator_reasons(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    locator_scope: &LocatorTestScope<'_>,
    line: u32,
    app_text: &AppTextTarget,
    context: &TestAnalysisContext<'_>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if has_reachable_route_signal(edges, rel_test_file, locator_scope, line, app_text, context) {
        reasons.push("route-signal".to_string());
    }
    if has_adjacent_selector_signal(
        edges,
        rel_test_file,
        locator_scope.test_name,
        locator_scope.describe_path,
        line,
        app_text,
    ) {
        reasons.push("adjacent-selector".to_string());
    }
    reasons
}

fn has_reachable_route_signal(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    locator_scope: &LocatorTestScope<'_>,
    line: u32,
    app_text: &AppTextTarget,
    context: &TestAnalysisContext<'_>,
) -> bool {
    edges.iter().any(|edge| {
        let Edge::Route {
            test_file,
            test_name: route_test_name,
            describe_path: route_describe_path,
            route_file,
            hook: route_is_hook,
            line: route_line,
            ..
        } = edge
        else {
            return false;
        };
        if test_file != rel_test_file
            || !route_signal_matches_test(
                route_test_name,
                route_describe_path,
                *route_is_hook,
                locator_scope.test_name,
                locator_scope.describe_path,
                locator_scope.is_hook,
            )
            || *route_line > line
        {
            return false;
        }
        context
            .route_reachable_files
            .get(route_file)
            .is_some_and(|files| files.contains(&app_text.app_file))
    })
}

fn has_adjacent_selector_signal(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    test_name: &Option<Arc<String>>,
    describe_path: &Arc<Vec<String>>,
    line: u32,
    app_text: &AppTextTarget,
) -> bool {
    edges.iter().any(|edge| {
        let Edge::Selector {
            test_file,
            test_name: selector_test_name,
            describe_path: selector_describe_path,
            app_file,
            line: selector_line,
            ..
        } = edge
        else {
            return false;
        };
        test_file == rel_test_file
            && app_file == &app_text.app_file
            && selector_test_name == test_name
            && selector_describe_path == describe_path
            && *selector_line <= line
            && line - *selector_line <= 5
    })
}
