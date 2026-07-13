use crate::fetch::cache::Cache;
use crate::fetch::file_analysis::{
    analyze_file, analyze_file_from_visible, analyze_file_from_visible_with_facts,
    VisibleFileAnalysis,
};
use crate::fetch::file_facts::ParsedFileCache;
use crate::fetch::import_routes::is_route_handler_file;
use crate::fetch::types::FetchOccurrence;
use crate::routes::Route;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn collect_route_fetches(
    route: &Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
) -> Result<Vec<FetchOccurrence>> {
    collect_route_fetches_inner(route, frontend_root, root, cache, None, None)
}

pub fn collect_route_fetches_from_visible(
    route: &Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
    visible_files: &HashSet<PathBuf>,
) -> Result<Vec<FetchOccurrence>> {
    collect_route_fetches_inner(route, frontend_root, root, cache, Some(visible_files), None)
}

#[doc(hidden)]
pub fn collect_route_fetches_from_visible_with_facts(
    route: &Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
    parsed_files: &mut ParsedFileCache,
    visible_files: &HashSet<PathBuf>,
) -> Result<Vec<FetchOccurrence>> {
    collect_route_fetches_inner(
        route,
        frontend_root,
        root,
        cache,
        Some(visible_files),
        Some(parsed_files),
    )
}

fn collect_route_fetches_inner(
    route: &Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
    visible_files: Option<&HashSet<PathBuf>>,
    parsed_files: Option<&mut ParsedFileCache>,
) -> Result<Vec<FetchOccurrence>> {
    let route_is_page = route.file.file_stem().and_then(|s| s.to_str()) == Some("page");
    let route_is_route_handler = is_route_handler_file(&route.file);

    let mut visited = HashSet::new();
    let mut fetches = Vec::new();

    let mut traversal = FetchTraversal {
        root,
        visited: &mut visited,
        fetches: &mut fetches,
        cache,
        visible_files,
        parsed_files,
    };
    let _route_is_client = traversal.analyze(&route.file, (false, route_is_route_handler))?;

    if route_is_page {
        collect_page_layout_fetches(route, frontend_root, &mut traversal)?;
    }

    fetches.sort();
    Ok(fetches)
}

fn collect_page_layout_fetches(
    route: &Route,
    frontend_root: &Path,
    traversal: &mut FetchTraversal<'_>,
) -> Result<()> {
    let route_is_route_handler = is_route_handler_file(&route.file);
    let mut current = route.file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }

        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if traversal.visible_files.map_or_else(
                    || layout_file.is_file(),
                    |visible| visible.contains(&layout_file),
                ) {
                    traversal.analyze(&layout_file, (false, route_is_route_handler))?;
                }
            }
        }
        current = parent.parent();
    }
    Ok(())
}

struct FetchTraversal<'a> {
    root: &'a Path,
    visited: &'a mut HashSet<(PathBuf, bool, bool)>,
    fetches: &'a mut Vec<FetchOccurrence>,
    cache: &'a mut Cache,
    visible_files: Option<&'a HashSet<PathBuf>>,
    parsed_files: Option<&'a mut ParsedFileCache>,
}

impl FetchTraversal<'_> {
    fn analyze(&mut self, path: &Path, inherited: (bool, bool)) -> Result<bool> {
        match (self.visible_files, &mut self.parsed_files) {
            (Some(visible), Some(parsed_files)) => analyze_file_from_visible_with_facts(
                path,
                inherited,
                &mut VisibleFileAnalysis {
                    root: self.root,
                    visited: self.visited,
                    fetches: self.fetches,
                    cache: self.cache,
                    parsed_files,
                    visible_files: visible,
                },
            ),
            (Some(visible), None) => analyze_file_from_visible(
                path,
                self.root,
                self.visited,
                self.fetches,
                self.cache,
                inherited,
                visible,
            ),
            (None, _) => {
                let (inherited_is_client, inherited_is_route_handler) = inherited;
                analyze_file(
                    path,
                    self.root,
                    self.visited,
                    self.fetches,
                    self.cache,
                    inherited_is_client,
                    inherited_is_route_handler,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests;
