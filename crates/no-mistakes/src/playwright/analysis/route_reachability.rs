use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::config;
use crate::playwright::routes;
use anyhow::Result;
use globset::GlobSet;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn collect_route_reachable_files(
    root: &Path,
    settings: &config::Settings,
    routes: &[routes::Route],
) -> Result<HashMap<Arc<String>, BTreeSet<Arc<String>>>> {
    let include = GlobSet::empty();
    let exclude = crate::playwright::fsutil::build_globset(&settings.selector_exclude)?;
    let selector_files = collect_selector_source_files(root, settings, &include, &exclude, true);
    let selector_rel_by_file: HashMap<_, _> = selector_files
        .iter()
        .map(|file| {
            (
                crate::codebase::ts_resolver::normalize_path(file),
                Arc::new(crate::playwright::fsutil::relative_string(root, file)),
            )
        })
        .collect();
    let mut import_cache = HashMap::new();
    let mut output = HashMap::new();
    for route in routes {
        output.insert(
            route_key(root, &route.file),
            reachable_files(&route.file, &selector_rel_by_file, &mut import_cache),
        );
    }
    Ok(output)
}

fn reachable_files(
    route_file: &Path,
    selector_rel_by_file: &HashMap<std::path::PathBuf, Arc<String>>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> BTreeSet<Arc<String>> {
    let mut reachable = BTreeSet::new();
    let mut stack = vec![crate::codebase::ts_resolver::normalize_path(route_file)];
    let mut seen = HashSet::new();
    while let Some(file) = stack.pop() {
        if !seen.insert(file.clone()) {
            continue;
        }
        if let Some(rel) = selector_rel_by_file.get(&file) {
            reachable.insert(rel.clone());
        }
        if let Ok(imports) = crate::imports::collect_imports(&file, import_cache) {
            stack.extend(
                imports
                    .into_iter()
                    .map(|file| crate::codebase::ts_resolver::normalize_path(&file)),
            );
        }
    }
    reachable
}

fn route_key(root: &Path, file: &Path) -> Arc<String> {
    Arc::new(crate::playwright::fsutil::relative_string(root, file))
}
