use crate::codebase::ts_resolver::{
    find_tsconfig_from_visible, load_tsconfig, ImportResolver, TsConfig,
};
use crate::config::v2::{load_v2_config_from_visible, ConfigView};
use crate::server_routes::model::{FileFacts, ProjectReport, RouteSite};
use crate::server_routes::mounts::{prefixes_for, resolve_mounts_with_resolver};
use crate::server_routes::normalize::{join_paths, normalize_route};
use crate::server_routes::source::{discover_source_files_from_visible, relative_string};
use crate::server_routes::types::{Diagnostic, Edge, EdgeKind, ServerRoute, Severity, Summary};
use anyhow::Context;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelatedDirection {
    Deps,
    Dependents,
    Both,
}

/// Request-scoped server analysis inputs shared by route and contract reports.
#[doc(hidden)]
pub struct PreparedServerAnalysis {
    pub(crate) root: PathBuf,
    pub(crate) source_files: std::sync::Arc<Vec<PathBuf>>,
    pub(crate) tsconfig: TsConfig,
    pub(crate) config: Option<crate::config::v2::NoMistakesConfig>,
    pub(crate) facts: crate::codebase::ts_source::facts::TsFactMap,
}

include!("graph_prepare.rs");

pub fn analyze_project(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<ProjectReport> {
    let prepared = prepare_analysis(root, tsconfig_path)?;
    analyze_project_with_prepared(&prepared, filters)
}

#[doc(hidden)]
pub fn analyze_project_with_prepared(
    prepared: &PreparedServerAnalysis,
    filters: &[String],
) -> anyhow::Result<ProjectReport> {
    let root = &prepared.root;
    let config_route_filter = prepared
        .config
        .as_ref()
        .and_then(|config| build_filter(&ConfigView::new(config).server_route_globs()).ok())
        .flatten();
    let test_filter = prepared
        .config
        .as_ref()
        .map(|config| crate::codebase::test_filter::TestFileFilter::new(root, config));
    let filter = build_filter(filters)?;
    let mut facts = HashMap::new();
    for path in prepared.source_files.iter() {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let matches_config = config_route_filter
            .as_ref()
            .map(|filter| filter.is_match(rel))
            .unwrap_or(true);
        let matches_cli = filter
            .as_ref()
            .map(|filter| filter.is_match(rel))
            .unwrap_or(true);
        let is_test = test_filter
            .as_ref()
            .is_some_and(|filter| filter.is_match(root, path));
        if matches_config && matches_cli && !is_test {
            if let Some(routes) = prepared
                .facts
                .get(path)
                .and_then(|facts| facts.server_routes.clone())
            {
                facts.insert(path.clone(), routes);
            }
        }
    }
    Ok(build_report(root, &facts, &prepared.tsconfig))
}

pub(crate) fn route_defs_from_files(
    root: &Path,
    files: &[PathBuf],
    tsconfig: &TsConfig,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = collect_file_facts(files, &root);
    build_report(&root, &facts, tsconfig)
        .routes
        .into_iter()
        .map(|route| (root.join(route.file), route.route))
        .collect()
}

fn collect_file_facts(files: &[PathBuf], root: &Path) -> HashMap<PathBuf, FileFacts> {
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        files,
        crate::codebase::ts_source::facts::TsFactPlan {
            server_routes: true,
            ..Default::default()
        },
        &crate::codebase::ts_source::facts::TsFactContext::new(root),
    );
    facts
        .into_iter()
        .filter_map(|(path, facts)| facts.server_routes.map(|routes| (path, routes)))
        .collect()
}

pub(super) fn build_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> ProjectReport {
    let mut routes = Vec::new();
    let mut edges = Vec::new();
    let mut diagnostics = Vec::new();
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new(tsconfig).with_visible(&visible);
    let mounts = resolve_mounts_with_resolver(facts, &resolver);
    for (path, file_facts) in facts {
        diagnostics.extend(
            file_facts
                .diagnostics
                .iter()
                .map(|(line, message)| Diagnostic {
                    severity: Severity::Warning,
                    file: relative_string(root, path),
                    line: *line,
                    message: message.clone(),
                }),
        );
        for site in &file_facts.routes {
            for route in expand_site(root, site, facts, &mounts) {
                edges.push(Edge {
                    from: route.file.clone(),
                    to: route.route.clone(),
                    kind: EdgeKind::ServerRoute,
                });
                routes.push(route);
            }
        }
    }
    routes.sort();
    routes.dedup();
    edges.sort();
    edges.dedup();
    diagnostics.sort();
    diagnostics.dedup();
    let dynamic_routes = routes
        .iter()
        .filter(|route| route.route.contains('*'))
        .count();
    ProjectReport {
        summary: Summary {
            total_routes: routes.len(),
            total_files: facts.len(),
            dynamic_routes,
        },
        routes,
        edges,
        diagnostics,
    }
}

fn expand_site(
    root: &Path,
    site: &RouteSite,
    facts: &HashMap<PathBuf, FileFacts>,
    mounts: &[crate::server_routes::mounts::ResolvedMount],
) -> Vec<ServerRoute> {
    prefixes_for(site, facts, mounts)
        .into_iter()
        .map(|prefix| {
            let raw_path = join_paths(&prefix, &site.raw_path);
            ServerRoute {
                file: relative_string(root, &site.file),
                line: site.line,
                method: site.method.clone(),
                route: normalize_route(&raw_path),
                raw_path,
                query_params: site.query_params.clone(),
                framework: site.framework,
            }
        })
        .collect()
}

fn build_filter(filters: &[String]) -> anyhow::Result<Option<GlobSet>> {
    if filters.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for filter in filters {
        builder.add(GlobBuilder::new(filter).literal_separator(false).build()?);
    }
    Ok(Some(builder.build()?))
}
