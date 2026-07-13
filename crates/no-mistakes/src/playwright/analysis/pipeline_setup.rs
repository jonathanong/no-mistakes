//! Early setup phase of `pipeline.rs`'s `analyze_with_policy_and_optional_facts`
//! (routes, test files, and the app-wide selector/text/reachability scans),
//! split out purely to stay under the 200-code-line-per-file cap — no
//! behavior change, this is the same sequence that used to live inline
//! there, with each cacheable scan already routed through the caller's
//! shared `TsFactLookup` when it has one (see `crates/CLAUDE.md`'s
//! "Duplicate full-repo work across independent call paths").

use crate::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, RouteReachableFiles, TsFactLookup,
};
use crate::playwright::analysis::app_collect::collect_app_selector_occurrences;
use crate::playwright::analysis::app_text::collect_app_text_targets;
use crate::playwright::analysis::context::DiscoveredTestFile;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::analysis::route_reachability::collect_route_reachable_files;
use crate::playwright::analysis::route_reachability::collect_route_source_files;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::config::Settings;
use crate::playwright::selectors::{self, AppSelector, SelectorRegexes};
use crate::playwright::{playwright_config, routes};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct PlaywrightSetup {
    pub(crate) routes: Arc<Vec<routes::Route>>,
    pub(crate) test_files: Vec<DiscoveredTestFile>,
    pub(crate) selector_regexes: SelectorRegexes,
    pub(crate) app_selectors: Vec<AppSelector>,
    pub(crate) app_selector_occurrences: Arc<Vec<AppSelector>>,
    pub(crate) app_text_targets: Arc<Vec<AppTextTarget>>,
    pub(crate) route_reachable_files: Arc<RouteReachableFiles>,
}

pub(crate) fn build_playwright_setup(
    root: &Path,
    settings: &Settings,
    unique_selector_policy: &UniqueSelectorPolicy,
    require_routes: bool,
    facts: Option<&dyn TsFactLookup>,
    route_import_graph: Option<&DepGraph>,
) -> Result<PlaywrightSetup> {
    let route_root = root.join(&settings.frontend_root);
    let compute_routes = || {
        let mut routes = routes::collect_routes(&route_root);
        let virtual_routes = crate::routes::rewrites::expand_rewrites(&settings.rewrites, &routes);
        routes.extend(virtual_routes);
        routes
    };
    let routes = crate::perf_trace::trace("playwright.routes", || match facts {
        Some(facts) => facts.get_or_compute_playwright_routes(&compute_routes),
        None => Arc::new(compute_routes()),
    });
    if require_routes && routes.is_empty() {
        let route_display = route_root.strip_prefix(root).unwrap_or(&route_root);
        anyhow::bail!(
            "no Next.js page routes found under {}",
            route_display.display()
        );
    }

    let playwright = playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    let test_files = crate::perf_trace::trace("playwright.discover_test_files", || {
        discover_test_files(root, settings, &playwright)
    })?;
    let selector_regexes = selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
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
            Some(facts) => facts.get_or_compute_app_selector_occurrences(scan_html_ids, &|| {
                collect_app_selector_occurrences(root, settings, &app_selector_regexes)
            }),
            None => collect_app_selector_occurrences(root, settings, &app_selector_regexes)
                .map(Arc::new),
        })?
    };
    let mut app_selectors: Vec<_> = app_selector_occurrences
        .iter()
        .filter(|selector| {
            settings.html_ids
                || unique_selector_policy.configured_html_id_selector
                || selector.attribute != selectors::HTML_ID_ATTRIBUTE
        })
        .cloned()
        .collect();
    app_selectors.sort();
    app_selectors.dedup();

    let app_text_targets =
        crate::perf_trace::trace("playwright.app_text_targets", || match facts {
            Some(facts) => {
                facts.get_or_compute_app_text_targets(&|| collect_app_text_targets(root, settings))
            }
            None => collect_app_text_targets(root, settings).map(Arc::new),
        })?;
    let route_reachable_files = if app_text_targets.is_empty() {
        Arc::new(Default::default())
    } else {
        let compute = || {
            let source_files = collect_route_source_files(root, settings)?;
            let supplied_graph = route_import_graph.filter(|graph| {
                source_files
                    .graph_files
                    .iter()
                    .all(|file| graph.contains_file(file))
            });
            let owned_route_import_graph = match supplied_graph {
                Some(_) => None,
                None => Some(build_route_import_graph(
                    root,
                    settings,
                    facts,
                    &source_files.graph_files,
                )?),
            };
            let route_import_graph = supplied_graph
                .or(owned_route_import_graph.as_ref())
                .expect("route-import graph is provided or built");
            collect_route_reachable_files(
                root,
                settings,
                routes.as_slice(),
                route_import_graph,
                &source_files,
            )
        };
        crate::perf_trace::trace("playwright.route_reachable_files", || match facts {
            Some(facts) => facts.get_or_compute_route_reachable_files(&compute),
            None => compute().map(Arc::new),
        })?
    };

    Ok(PlaywrightSetup {
        routes,
        test_files,
        selector_regexes,
        app_selectors,
        app_selector_occurrences,
        app_text_targets,
        route_reachable_files,
    })
}

pub(crate) fn build_route_import_graph(
    root: &Path,
    settings: &Settings,
    facts: Option<&dyn TsFactLookup>,
    route_source_files: &[PathBuf],
) -> Result<DepGraph> {
    let tsconfig = load_route_import_tsconfig(root, settings)?;
    let mut graph_file_paths = facts
        .and_then(TsFactLookup::graph_files)
        .map(<[PathBuf]>::to_vec)
        .unwrap_or_else(|| GraphFiles::discover(root).all().to_vec());
    graph_file_paths.extend_from_slice(route_source_files);
    graph_file_paths.sort();
    graph_file_paths.dedup();
    let graph_files = GraphFiles::from_files(graph_file_paths);
    Ok(DepGraph::build_with_plan_files_config_and_facts(
        root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        facts,
    ))
}

pub(crate) fn load_route_import_tsconfig(
    root: &Path,
    settings: &Settings,
) -> Result<crate::codebase::ts_resolver::TsConfig> {
    let frontend_root = root.join(&settings.frontend_root);
    Ok(crate::codebase::ts_resolver::find_tsconfig(&frontend_root)
        .or_else(|| crate::codebase::ts_resolver::find_tsconfig(root))
        .map(|path| crate::codebase::ts_resolver::load_tsconfig(&path))
        .transpose()?
        .unwrap_or_else(|| crate::codebase::ts_resolver::TsConfig {
            dir: root.to_path_buf(),
            paths: Vec::new(),
            paths_dir: root.to_path_buf(),
            base_url: None,
        }))
}
