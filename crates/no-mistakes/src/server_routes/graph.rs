use crate::codebase::ts_resolver::{
    find_tsconfig_from_visible, load_tsconfig, ImportResolver, TsConfig,
};
use crate::config::v2::{load_v2_config_from_visible, ConfigView};
use crate::edge_index::{CanonicalEdge, EdgeIndex, NodeAliases};
use crate::server_routes::model::{FileFacts, PreparedProjectReport, ProjectReport, RouteSite};
use crate::server_routes::mounts::{prefixes_for, resolve_mounts_with_resolver};
use crate::server_routes::normalize::{join_paths, normalize_route};
use crate::server_routes::source::{discover_source_files_from_visible, relative_string};
use crate::server_routes::types::{
    Diagnostic, Edge, EdgeKind, RelationshipEdge, RelationshipNode, ServerRoute, Severity, Summary,
};
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
pub fn analyze_project_indexed(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<PreparedProjectReport> {
    let prepared = prepare_analysis(root, tsconfig_path)?;
    analyze_project_with_prepared_indexed(&prepared, filters)
}

#[doc(hidden)]
pub fn analyze_project_with_prepared(
    prepared: &PreparedServerAnalysis,
    filters: &[String],
) -> anyhow::Result<ProjectReport> {
    analyze_project_with_prepared_inner(prepared, filters, build_report)
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_indexed(
    prepared: &PreparedServerAnalysis,
    filters: &[String],
) -> anyhow::Result<PreparedProjectReport> {
    analyze_project_with_prepared_inner(prepared, filters, build_prepared_report)
}

fn analyze_project_with_prepared_inner<T>(
    prepared: &PreparedServerAnalysis,
    filters: &[String],
    builder: impl FnOnce(&Path, &HashMap<PathBuf, FileFacts>, &TsConfig) -> T,
) -> anyhow::Result<T> {
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
    Ok(builder(root, &facts, &prepared.tsconfig))
}

pub(crate) fn route_defs_from_files(
    root: &Path,
    files: &[PathBuf],
    tsconfig: &TsConfig,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = collect_file_facts(files, &root);
    build_route_defs(&root, &facts, tsconfig)
}

pub(crate) fn route_defs_from_prepared_facts(
    root: &Path,
    tsconfig: &TsConfig,
    prepared: impl IntoIterator<Item = (PathBuf, FileFacts)>,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = prepared.into_iter().collect();
    build_route_defs(&root, &facts, tsconfig)
}

fn build_route_defs(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> Vec<(PathBuf, String)> {
    build_report(root, facts, tsconfig)
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
        .filter_map(|(path, facts)| facts.server_routes.clone().map(|routes| (path, routes)))
        .collect()
}

include!("graph_report.rs");

pub(crate) fn configure_fact_context(
    context: &mut crate::codebase::ts_source::facts::TsFactContext,
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) {
    if let Some(glob) = build_filter(&ConfigView::new(config).server_route_globs())
        .ok()
        .flatten()
    {
        context.set_server_route_filter(
            glob,
            Some(crate::codebase::test_filter::TestFileFilter::new(
                root, config,
            )),
        );
    }
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
