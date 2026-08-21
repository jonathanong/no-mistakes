mod path;

use crate::codebase::config::InferredRoots;
use crate::codebase::test_filter::TestFileFilter;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::{NoMistakesConfig, ProjectType};
use crate::config::v2::ConfigView;
use crate::server_routes::graph::{build_filter, PreparedServerAnalysis};
use crate::server_routes::model::{FileFacts, RouteSite};
use crate::server_routes::types::Framework;
use globset::GlobSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn merge_remix_route_facts(
    prepared: &PreparedServerAnalysis,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
) {
    let Some(config) = prepared.config.as_ref() else {
        return;
    };
    let roots = remix_project_roots(&prepared.root, config, &prepared.source_files);
    if roots.is_empty() {
        return;
    }
    let config_route_filter = build_filter(&ConfigView::new(config).server_route_globs())
        .ok()
        .flatten();
    let test_filter = TestFileFilter::new(&prepared.root, config);
    for path in prepared.source_files.iter() {
        let Some(site) = route_site(path, &roots) else {
            continue;
        };
        let rel = path.strip_prefix(&prepared.root).unwrap_or(path);
        let matches_config = config_route_filter
            .as_ref()
            .map(|filter| filter.is_match(rel))
            .unwrap_or(true);
        let matches_cli = cli_filter
            .map(|filter| filter.is_match(rel))
            .unwrap_or(true);
        let is_test = test_filter.is_match(&prepared.root, path);
        if matches_config && matches_cli && !is_test {
            facts.entry(path.clone()).or_default().routes.push(site);
        }
    }
}

fn remix_project_roots(
    workspace: &Path,
    config: &NoMistakesConfig,
    visible: &[PathBuf],
) -> Vec<PathBuf> {
    let mut inferred = InferredRoots::from_visible(workspace, visible);
    let mut roots = Vec::new();
    for project in config.projects.values() {
        if project.type_ != Some(ProjectType::Remix) {
            continue;
        }
        let root = match project.root.as_deref() {
            Some(root) => workspace.join(root),
            None => inferred
                .remix_root(workspace)
                .unwrap_or_else(|| workspace.to_path_buf()),
        };
        roots.push(crate::codebase::ts_resolver::normalize_path(&root));
    }
    roots.sort();
    roots.dedup();
    roots
}

fn route_site(path: &Path, remix_roots: &[PathBuf]) -> Option<RouteSite> {
    let remix_root = remix_roots.iter().find(|root| path.starts_with(root))?;
    let rel = relative_slash_path(remix_root, path);
    let route = if let Some(rest) = rel
        .strip_prefix("app/routes/")
        .or_else(|| rel.strip_prefix("routes/"))
    {
        path::route_from_routes_rel(rest)?
    } else {
        path::route_from_app_root(&rel)?
    };
    Some(RouteSite {
        file: path.to_path_buf(),
        line: 1,
        binding: String::new(),
        method: "*".to_string(),
        raw_path: route.clone(),
        path: route,
        query_params: Vec::new(),
        framework: Framework::Remix,
    })
}

#[cfg(test)]
mod tests;
