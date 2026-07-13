use crate::playwright::analysis::text_types::{AppTextTarget, PlaywrightTextLocator};
use crate::playwright::analysis::types::Edge;
use crate::playwright::playwright_tests::{TestOccurrence, TestOccurrenceScope};
use std::sync::Arc;

mod context;
pub(crate) use context::TextEdgeContext;
mod index;
pub(crate) use index::AppTextIndex;
mod matching;
use matching::{text_target_matches, TextMatch};
mod route_scope;
pub(crate) use route_scope::route_signal_matches_locator;
#[cfg(test)]
mod tests;

struct LocatorTestScope<'a> {
    test_name: Option<&'a str>,
    describe_path: &'a [String],
    scope: TestOccurrenceScope,
}

pub(crate) fn locator_has_app_text_candidate(
    app_text_targets: &[AppTextTarget],
    app_text_index: &AppTextIndex,
    locator: &PlaywrightTextLocator,
) -> bool {
    app_text_index
        .candidates(locator)
        .iter()
        .any(|position| locator_matches_target(locator, &app_text_targets[*position]))
}

pub(crate) fn append_locator_text_edges(
    edges: &mut Vec<Edge>,
    rel_test_file: &Arc<String>,
    test_id_attributes: &[String],
    context: &TextEdgeContext<'_>,
    text_locators: &[TestOccurrence<PlaywrightTextLocator>],
) {
    let existing_edges = edges.as_slice();
    let mut locator_edges = Vec::new();
    for text_locator in text_locators {
        if !context.test_policy.allows(text_locator.status)
            || text_locator.scope == TestOccurrenceScope::TeardownHook
        {
            continue;
        }
        let locator_scope = LocatorTestScope {
            test_name: text_locator.test_name.as_deref(),
            describe_path: &text_locator.describe_path,
            scope: text_locator.scope,
        };
        for position in context.app_text_index.candidates(&text_locator.value) {
            let app_text = &context.app_text_targets[*position];
            if !locator_matches_target(&text_locator.value, app_text) {
                continue;
            }
            let reasons = locator_reasons(
                existing_edges,
                rel_test_file,
                &locator_scope,
                text_locator.line,
                app_text,
                context,
            );
            if reasons.is_empty() {
                continue;
            }
            locator_edges.push(Edge::LocatorText {
                test_file: rel_test_file.clone(),
                test_name: text_locator.test_name.clone().map(Arc::new),
                describe_path: Arc::new(text_locator.describe_path.clone()),
                app_file: app_text.app_file.clone(),
                locator_kind: text_locator.value.kind.as_str().to_string(),
                role: text_locator.value.role.clone(),
                text: text_locator.value.text.clone(),
                locator: text_locator.value.locator.clone(),
                test_id_attributes: test_id_attributes.to_vec(),
                selector_refs: app_text.selector_refs.clone(),
                reasons,
                line: text_locator.line,
            });
        }
    }
    locator_edges.sort();
    edges.extend(locator_edges);
}

fn locator_matches_target(locator: &PlaywrightTextLocator, target: &AppTextTarget) -> bool {
    text_target_matches(
        target,
        &locator.kind,
        locator.role.as_deref(),
        &TextMatch::new(&locator.text, locator.exact),
        locator.include_hidden,
    )
}

fn locator_reasons(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    locator_scope: &LocatorTestScope<'_>,
    line: u32,
    app_text: &AppTextTarget,
    context: &TextEdgeContext<'_>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if has_reachable_route_signal(edges, rel_test_file, locator_scope, line, app_text, context) {
        reasons.push("route-signal".to_string());
    }
    if has_adjacent_selector_signal(edges, rel_test_file, locator_scope, line, app_text) {
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
    context: &TextEdgeContext<'_>,
) -> bool {
    edges.iter().any(|edge| {
        let Edge::Route { route_file, .. } = edge else {
            return false;
        };
        if !route_signal_matches_locator(
            edge,
            rel_test_file,
            locator_scope.test_name,
            locator_scope.describe_path,
            locator_scope.scope,
            line,
        ) {
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
    locator_scope: &LocatorTestScope<'_>,
    line: u32,
    app_text: &AppTextTarget,
) -> bool {
    if locator_scope.test_name.is_none() && locator_scope.describe_path.is_empty() {
        return false;
    }
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
            && selector_test_name.as_deref().map(String::as_str) == locator_scope.test_name
            && selector_describe_path.as_slice() == locator_scope.describe_path
            && *selector_line <= line
            && line - *selector_line <= 5
    })
}
