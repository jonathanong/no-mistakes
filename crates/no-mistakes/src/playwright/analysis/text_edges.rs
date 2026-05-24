use crate::playwright::analysis::context::TestAnalysisContext;
use crate::playwright::analysis::text_types::{
    AppTextKind, AppTextTarget, LocatorKind, PlaywrightTextLocator,
};
use crate::playwright::analysis::types::Edge;
use crate::playwright::playwright_tests::TestOccurrence;
use std::sync::Arc;

pub(crate) fn append_locator_text_edges(
    edges: &mut Vec<Edge>,
    rel_test_file: &Arc<String>,
    context: &TestAnalysisContext<'_>,
    text_locators: Vec<TestOccurrence<PlaywrightTextLocator>>,
) {
    for text_locator in text_locators {
        if !context.test_policy.allows(text_locator.status) {
            continue;
        }
        let test_name = text_locator.test_name.map(Arc::new);
        let describe_path = Arc::new(text_locator.describe_path);

        for app_text in context.app_text_targets.iter().filter(|target| {
            text_target_matches(
                target,
                &text_locator.value.kind,
                text_locator.value.role.as_deref(),
                &text_locator.value.text,
                text_locator.value.exact,
            )
        }) {
            let reasons = locator_reasons(
                edges,
                rel_test_file,
                &test_name,
                &describe_path,
                text_locator.line,
                app_text,
                context,
            );
            if reasons.is_empty() {
                continue;
            }
            edges.push(Edge::LocatorText {
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
            });
        }
    }
}

fn locator_reasons(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    test_name: &Option<Arc<String>>,
    describe_path: &Arc<Vec<String>>,
    line: u32,
    app_text: &AppTextTarget,
    context: &TestAnalysisContext<'_>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if has_reachable_route_signal(
        edges,
        rel_test_file,
        test_name,
        describe_path,
        line,
        app_text,
        context,
    ) {
        reasons.push("route-signal".to_string());
    }
    if has_adjacent_selector_signal(
        edges,
        rel_test_file,
        test_name,
        describe_path,
        line,
        app_text,
    ) {
        reasons.push("adjacent-selector".to_string());
    }
    reasons
}

fn text_target_matches(
    target: &AppTextTarget,
    kind: &LocatorKind,
    role: Option<&str>,
    text: &str,
    exact: bool,
) -> bool {
    locator_text_matches(&target.text, text, exact)
        && match kind {
            LocatorKind::Text => target.kind == AppTextKind::VisibleText,
            LocatorKind::Label => target.kind == AppTextKind::Label,
            LocatorKind::Placeholder => target.kind == AppTextKind::Placeholder,
            LocatorKind::Role => {
                target.role.as_deref() == role
                    && (target.kind == AppTextKind::VisibleText
                        || target.kind == AppTextKind::AccessibleName)
            }
        }
}

fn locator_text_matches(target: &str, locator: &str, exact: bool) -> bool {
    if exact {
        return target == locator;
    }
    target.to_lowercase().contains(&locator.to_lowercase())
}

fn has_reachable_route_signal(
    edges: &[Edge],
    rel_test_file: &Arc<String>,
    test_name: &Option<Arc<String>>,
    describe_path: &Arc<Vec<String>>,
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
                test_name,
                describe_path,
            )
            || (route_test_name.is_some() && *route_line > line)
        {
            return false;
        }
        context
            .route_reachable_files
            .get(route_file)
            .is_some_and(|files| files.contains(&app_text.app_file))
    })
}

fn route_signal_matches_test(
    route_test_name: &Option<Arc<String>>,
    route_describe_path: &Arc<Vec<String>>,
    test_name: &Option<Arc<String>>,
    describe_path: &Arc<Vec<String>>,
) -> bool {
    if route_test_name == test_name && route_describe_path == describe_path {
        return true;
    }
    route_test_name.is_none()
        && test_name.is_some()
        && describe_path_starts_with(describe_path, route_describe_path)
}

fn describe_path_starts_with(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
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
            && selector_line.abs_diff(line) <= 5
    })
}
