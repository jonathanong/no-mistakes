use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::playwright::config;
use crate::playwright::fsutil::{build_globset, walk_files_from_snapshot, VisiblePathSnapshot};
use crate::playwright::routes;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct RouteSourceFiles {
    pub(crate) graph_files: Vec<PathBuf>,
    visible_files: HashSet<PathBuf>,
    selector_rel_by_file: HashMap<PathBuf, Arc<String>>,
}

pub(crate) fn collect_route_source_files_from_visible(
    root: &Path,
    settings: &config::Settings,
    snapshot: &VisiblePathSnapshot,
) -> Result<RouteSourceFiles> {
    let include = build_globset(&settings.selector_include)?;
    let exclude = build_globset(&settings.selector_exclude)?;
    let include_all = settings.selector_include.is_empty();
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let visible_files = snapshot
        .paths_for(root)
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    let mut graph_files = BTreeSet::new();
    let mut selector_rel_by_file = HashMap::new();

    for selector_root in &settings.selector_roots {
        let source_root = root.join(selector_root);
        if !source_root.exists() {
            continue;
        }
        for file in walk_files_from_snapshot(&source_root, snapshot) {
            if !crate::playwright::selectors::is_source_file(&file) {
                continue;
            }
            let file = crate::codebase::ts_resolver::normalize_path(&file);
            graph_files.insert(file.clone());
            let relative = crate::playwright::fsutil::relative_string(&normalized_root, &file);
            if (include_all || include.is_match(&relative)) && !exclude.is_match(&relative) {
                selector_rel_by_file.insert(file, Arc::new(relative));
            }
        }
    }

    Ok(RouteSourceFiles {
        graph_files: graph_files.into_iter().collect(),
        visible_files,
        selector_rel_by_file,
    })
}

pub(crate) fn collect_route_reachable_files(
    root: &Path,
    settings: &config::Settings,
    routes: &[routes::Route],
    graph: &DepGraph,
    source_files: &RouteSourceFiles,
) -> Result<BTreeMap<Arc<String>, BTreeSet<Arc<String>>>> {
    routes
        .par_iter()
        .map(|route| {
            Ok((
                route_key(root, &route.file),
                reachable_files(root, settings, &route.file, source_files, graph)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()
}

fn reachable_files(
    root: &Path,
    settings: &config::Settings,
    route_file: &Path,
    source_files: &RouteSourceFiles,
    graph: &DepGraph,
) -> Result<BTreeSet<Arc<String>>> {
    let entry_files = route_entry_files(root, settings, route_file, &source_files.visible_files)
        .into_iter()
        .map(|file| crate::codebase::ts_resolver::normalize_path(&file))
        .collect::<Vec<_>>();
    let roots = entry_files
        .iter()
        .cloned()
        .map(NodeId::File)
        .collect::<Vec<_>>();
    let allowed: HashSet<_> = [EdgeKind::RouteImport].into();
    let dependencies = graph.deps_of(&roots, None, Some(&allowed));
    let all_files = entry_files
        .iter()
        .map(PathBuf::as_path)
        .chain(dependencies.iter().filter_map(|entry| entry.node.as_file()))
        .collect::<BTreeSet<_>>();

    // BTreeSet order makes the surfaced parse error deterministic.
    for file in &all_files {
        if let Some(error) = graph.parse_error(file) {
            anyhow::bail!(
                "failed to parse route-reachable {}: {error}",
                file.display()
            );
        }
    }

    Ok(all_files
        .into_iter()
        .filter_map(|file| source_files.selector_rel_by_file.get(file).cloned())
        .collect())
}

fn route_entry_files(
    root: &Path,
    settings: &config::Settings,
    route_file: &Path,
    visible_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let frontend_root = root.join(&settings.frontend_root);
    let mut files = vec![route_file.to_path_buf()];
    files.extend(
        crate::fetch::import_routes::collect_layout_chain_files(route_file, &frontend_root)
            .into_iter()
            .filter(|file| {
                matches!(
                    file.file_stem().and_then(|stem| stem.to_str()),
                    Some("layout" | "template")
                ) && visible_files.contains(&crate::codebase::ts_resolver::normalize_path(file))
            }),
    );
    files
}

fn route_key(root: &Path, file: &Path) -> Arc<String> {
    Arc::new(crate::playwright::fsutil::relative_string(root, file))
}

#[cfg(test)]
mod tests;
