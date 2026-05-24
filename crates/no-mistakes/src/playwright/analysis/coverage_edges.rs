use crate::playwright::analysis::coverage::{RouteCoverageEntry, SelectorCoverageEntry};
use crate::playwright::analysis::fetch::FetchCoverageEntry;
use crate::playwright::analysis::types::{Edge, TestRef};

pub(super) fn seed_coverage_from_edges<'a>(
    edges: &'a [Edge],
    by_route: &mut RouteCoverageEntry<'a>,
    by_selector: &mut SelectorCoverageEntry,
    by_fetch: &mut FetchCoverageEntry,
) {
    for edge in edges {
        match edge {
            Edge::Route {
                test_file,
                test_name,
                describe_path,
                route,
                url,
                ..
            } => seed_route(by_route, route, url, test_file, test_name, describe_path),
            Edge::Selector {
                test_file,
                test_name,
                describe_path,
                app_file,
                attribute,
                value,
                selector,
                ..
            } => {
                let key = (app_file.clone(), attribute.clone(), value.clone());
                seed_selector(
                    by_selector,
                    key,
                    selector,
                    test_file,
                    test_name,
                    describe_path,
                );
            }
            Edge::Fetch {
                test_file,
                test_name,
                describe_path,
                route_file,
                method,
                path,
                ..
            } => {
                let key = (method.clone(), path.clone());
                let entry = by_fetch.entry(key).or_insert_with(|| {
                    (Default::default(), Default::default(), Default::default())
                });
                entry.0.insert(test_file.clone());
                entry
                    .1
                    .insert(test_ref(test_file, test_name, describe_path));
                entry.2.insert(route_file.clone());
            }
            Edge::LocatorText {
                test_file,
                test_name,
                describe_path,
                app_file,
                locator,
                selector_refs,
                ..
            } => {
                for selector_ref in selector_refs {
                    let key = (
                        app_file.clone(),
                        selector_ref.attribute.clone(),
                        selector_ref.value.clone(),
                    );
                    seed_selector(
                        by_selector,
                        key,
                        locator,
                        test_file,
                        test_name,
                        describe_path,
                    );
                }
            }
        }
    }
}

fn seed_route<'a>(
    by_route: &mut RouteCoverageEntry<'a>,
    route: &'a str,
    url: &std::sync::Arc<String>,
    test_file: &std::sync::Arc<String>,
    test_name: &Option<std::sync::Arc<String>>,
    describe_path: &std::sync::Arc<Vec<String>>,
) {
    let entry = by_route
        .entry(route)
        .or_insert_with(|| (Default::default(), Default::default(), Default::default()));
    entry.0.insert(test_file.clone());
    entry.1.insert(url.clone());
    entry
        .2
        .insert(test_ref(test_file, test_name, describe_path));
}

fn seed_selector(
    by_selector: &mut SelectorCoverageEntry,
    key: crate::playwright::analysis::types::SelectorCoverageKey,
    selector: &str,
    test_file: &std::sync::Arc<String>,
    test_name: &Option<std::sync::Arc<String>>,
    describe_path: &std::sync::Arc<Vec<String>>,
) {
    let entry = by_selector
        .entry(key)
        .or_insert_with(|| ((Default::default(), Default::default()), Default::default()));
    entry.0 .0.insert(test_file.clone());
    entry.0 .1.insert(selector.to_string());
    entry
        .1
        .insert(test_ref(test_file, test_name, describe_path));
}

fn test_ref(
    test_file: &std::sync::Arc<String>,
    test_name: &Option<std::sync::Arc<String>>,
    describe_path: &std::sync::Arc<Vec<String>>,
) -> TestRef {
    TestRef {
        file: test_file.clone(),
        name: test_name.clone(),
        describe_path: describe_path.clone(),
    }
}
