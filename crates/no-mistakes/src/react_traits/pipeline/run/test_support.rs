use super::*;

#[inline(never)]
pub(super) fn run_analyze_inner(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    depth: Option<usize>,
) -> Result<Vec<ComponentFacts>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    run_analyze_inner_from_visible(&root, file_config, targets, depth, &visible_paths, None)
}

pub(super) fn aggregate_children(
    facts: &ComponentFacts,
    file_cache: &mut HashMap<PathBuf, Vec<ComponentFacts>>,
    root: &Path,
    visited: &mut HashSet<String>,
) -> AggregatedFacts {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let entries = std::mem::take(file_cache);
    file_cache.extend(entries.into_iter().map(|(path, components)| {
        (
            crate::codebase::ts_resolver::normalize_path(&path),
            components,
        )
    }));
    aggregate_children_inner(facts, file_cache, &root, None, None, visited)
}

pub(super) fn aggregate_children_from_visible(
    facts: &ComponentFacts,
    file_cache: &mut HashMap<PathBuf, Vec<ComponentFacts>>,
    root: &Path,
    visible_files: &crate::fx::PathSet,
    visited: &mut HashSet<String>,
) -> AggregatedFacts {
    aggregate_children_inner(facts, file_cache, root, Some(visible_files), None, visited)
}
