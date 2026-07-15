use crate::codebase::dependencies::graph::TsFactLookup;
use crate::playwright::analysis::app_collect::collect_app_selector_occurrences_from_visible;
use crate::playwright::analysis::context::DiscoveredTestFile;
use crate::playwright::analysis::discover::discover_test_files_from_visible;
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::selectors::{self, AppSelector};
use crate::playwright::{playwright_config, routes};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub(crate) struct PlaywrightSetup {
    pub(crate) routes: Arc<Vec<routes::Route>>,
    pub(crate) app_selectors: Vec<AppSelector>,
    pub(crate) app_selector_occurrences: Arc<Vec<AppSelector>>,
}

pub(crate) struct AppSelectorSetup {
    pub(crate) app_selectors: Vec<AppSelector>,
    pub(crate) app_selector_occurrences: Arc<Vec<AppSelector>>,
}

pub(crate) fn discover_playwright_test_files(
    root: &Path,
    settings: &Settings,
    facts: Option<&dyn TsFactLookup>,
    snapshot: &VisiblePathSnapshot,
) -> Result<Vec<DiscoveredTestFile>> {
    if let Some(test_files) =
        facts.and_then(|facts| facts.get_playwright_test_files(settings.project.as_deref()))
    {
        return Ok(test_files.as_ref().clone());
    }
    let playwright = playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    crate::perf_trace::trace("playwright.discover_test_files", || {
        discover_test_files_from_visible(root, settings, &playwright, snapshot)
    })
}

pub(crate) fn collect_playwright_routes(
    root: &Path,
    settings: &Settings,
    require_routes: bool,
    route_demand: bool,
    facts: Option<&dyn TsFactLookup>,
    snapshot: &VisiblePathSnapshot,
) -> Result<Arc<Vec<routes::Route>>> {
    let route_root = root.join(&settings.frontend_root);
    let compute_routes = || {
        let mut routes = routes::collect_routes_from_snapshot(&route_root, snapshot);
        let virtual_routes = crate::routes::rewrites::expand_rewrites(&settings.rewrites, &routes);
        routes.extend(virtual_routes);
        routes
    };
    let routes = if require_routes || route_demand {
        crate::perf_trace::trace("playwright.routes", || match facts {
            Some(facts) => facts.get_or_compute_playwright_routes(settings, &compute_routes),
            None => Arc::new(compute_routes()),
        })
    } else {
        Arc::new(Vec::new())
    };
    if require_routes && routes.is_empty() {
        let route_display = route_root.strip_prefix(root).unwrap_or(&route_root);
        anyhow::bail!(
            "no Next.js page routes found under {}",
            route_display.display()
        );
    }
    Ok(routes)
}

pub(crate) fn collect_app_selectors(
    root: &Path,
    settings: &Settings,
    unique_selector_policy: &UniqueSelectorPolicy,
    facts: Option<&dyn TsFactLookup>,
    snapshot: &VisiblePathSnapshot,
) -> Result<AppSelectorSetup> {
    validate_prepared_selector_source_errors(root, settings, facts, snapshot)?;
    let unique_html_id_scan = unique_selector_policy.html_ids && !settings.html_ids;
    let app_selector_regexes = selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids || unique_html_id_scan,
    );
    let scan_html_ids = settings.html_ids || unique_html_id_scan;
    let app_selector_occurrences: Arc<Vec<AppSelector>> = if settings.selector_attributes.is_empty()
        && settings.component_selector_attributes.is_empty()
        && !settings.html_ids
        && !unique_html_id_scan
    {
        Arc::new(Vec::new())
    } else {
        crate::perf_trace::trace("playwright.app_selector_occurrences", || match facts {
            Some(facts) => {
                facts.get_or_compute_app_selector_occurrences(settings, scan_html_ids, &|| {
                    collect_app_selector_occurrences_from_visible(
                        root,
                        settings,
                        &app_selector_regexes,
                        snapshot,
                    )
                })
            }
            None => collect_app_selector_occurrences_from_visible(
                root,
                settings,
                &app_selector_regexes,
                snapshot,
            )
            .map(Arc::new),
        })?
    };
    let configured_attributes = settings
        .selector_attributes
        .iter()
        .chain(settings.component_selector_attributes.values())
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();
    let mut app_selectors: Vec<_> = app_selector_occurrences
        .iter()
        .filter(|selector| {
            if selector.attribute == selectors::HTML_ID_ATTRIBUTE {
                return settings.html_ids
                    || unique_selector_policy.configured_html_id_selector
                    || unique_html_id_scan;
            }
            configured_attributes.contains(selector.attribute.as_str())
        })
        .cloned()
        .collect();
    app_selectors.sort();
    app_selectors.dedup();
    Ok(AppSelectorSetup {
        app_selectors,
        app_selector_occurrences,
    })
}

pub(crate) fn validate_prepared_selector_source_errors(
    root: &Path,
    settings: &Settings,
    facts: Option<&dyn TsFactLookup>,
    snapshot: &VisiblePathSnapshot,
) -> Result<()> {
    let Some(facts) = facts else {
        return Ok(());
    };
    let include = crate::playwright::fsutil::build_globset(&settings.selector_include)?;
    let exclude = crate::playwright::fsutil::build_globset(&settings.selector_exclude)?;
    let source_files =
        crate::playwright::analysis::app_collect::collect_selector_source_files_from_visible(
            root,
            settings,
            &include,
            &exclude,
            settings.selector_include.is_empty(),
            snapshot,
        );
    for path in source_files {
        if let Some(error) = facts.get_playwright_parse_error(&path) {
            anyhow::bail!(
                "failed to parse selector source {}: {error}",
                path.display()
            );
        }
    }
    Ok(())
}
