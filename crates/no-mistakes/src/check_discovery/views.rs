use super::{
    explicit_reopened_roots, preserved_project_roots_with_inferred,
    preserved_roots::{
        include_patterns_by_base_with_inferred, include_preserved_roots_from_files_with_inferred,
        repository_include_patterns,
    },
    unique_exports_project_roots_with_inferred, unresolved_typed_reopen_suffixes,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

mod fallback_walk;
mod filter;
mod inferred_roots;

pub(super) use fallback_walk::walk_ignore_aware_universe;
use filter::filter_filesystem_view;
pub(super) use inferred_roots::infer_project_roots_from_files;

pub(crate) struct CheckFileViews {
    pub(crate) filesystem: Vec<PathBuf>,
    pub(crate) graph: Vec<PathBuf>,
}

pub(crate) fn discover_check_file_views(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
) -> CheckFileViews {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let root_files = no_mistakes::codebase::ts_source::git_visible_files(&root);
    discover_check_file_views_from_git_files(
        &root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
    )
}

/// Builds both views from one universe per configured base. `root_files: None`
/// means Git is known unavailable; this function must not retry Git discovery.
pub(super) fn discover_check_file_views_from_git_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    root_files: Option<Vec<String>>,
) -> CheckFileViews {
    discover_check_file_views_with_external_lookup(
        root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
        no_mistakes::codebase::ts_source::git_visible_files,
    )
}

pub(super) fn discover_check_file_views_with_external_lookup(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    root_files: Option<Vec<String>>,
    mut external_git_files: impl FnMut(&Path) -> Option<Vec<String>>,
) -> CheckFileViews {
    // Git provides the complete repository universe once. When Git is known
    // unavailable, walk each base exactly once with pattern-aware skip pruning.
    let fallback = root_files.is_none();
    let reopened_roots = if fallback {
        explicit_reopened_roots(root, config, unique_exports_enabled)
    } else {
        Vec::new()
    };
    let reopened_suffixes = if fallback {
        unresolved_typed_reopen_suffixes(config)
    } else {
        Vec::new()
    };
    let mut universe = match root_files {
        Some(root_files) => existing_git_paths(root, root_files),
        None => walk_ignore_aware_universe(
            root,
            &repository_include_patterns(config),
            &reopened_roots,
            &reopened_suffixes,
        ),
    };
    let mut inferred_roots = infer_project_roots_from_files(root, &universe);
    let unique_exports_roots = if unique_exports_enabled {
        unique_exports_project_roots_with_inferred(root, config, &mut inferred_roots)
    } else {
        Vec::new()
    };
    let mut project_roots =
        preserved_project_roots_with_inferred(root, config, &mut inferred_roots);
    project_roots.extend(unique_exports_roots.iter().cloned());
    project_roots.sort();
    project_roots.dedup();
    let patterns = include_patterns_by_base_with_inferred(root, config, &mut inferred_roots);
    for project_root in project_roots
        .iter()
        .filter(|project_root| !project_root.starts_with(root))
    {
        let includes = patterns
            .get(project_root)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if fallback {
            universe.extend(walk_ignore_aware_universe(project_root, includes, &[], &[]));
        } else {
            match external_git_files(project_root) {
                Some(files) => universe.extend(existing_git_paths(project_root, files)),
                None => {
                    universe.extend(walk_ignore_aware_universe(project_root, includes, &[], &[]));
                }
            }
        }
    }
    universe.sort();
    universe.dedup();

    let graph_preserved_roots = include_preserved_roots_from_files_with_inferred(
        root,
        config,
        &[],
        &universe,
        &mut inferred_roots,
    );
    let graph = filter_filesystem_view(
        root,
        universe.into_iter(),
        &[],
        &graph_preserved_roots,
        &unique_exports_roots,
    );

    let preserved_roots = include_preserved_roots_from_files_with_inferred(
        root,
        config,
        skip_directories,
        &graph,
        &mut inferred_roots,
    );
    let filesystem = filter_filesystem_view(
        root,
        graph.iter().cloned(),
        skip_directories,
        &preserved_roots,
        &unique_exports_roots,
    );

    CheckFileViews { filesystem, graph }
}

fn existing_git_paths(root: &Path, files: Vec<String>) -> Vec<PathBuf> {
    files
        .into_iter()
        .map(|relative| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(relative)))
        .filter(|path| path.exists())
        .collect()
}
