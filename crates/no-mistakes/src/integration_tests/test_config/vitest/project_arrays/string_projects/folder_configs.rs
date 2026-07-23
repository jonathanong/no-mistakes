use super::{is_vitest_project_config, slash_path};
use globset::{GlobBuilder, GlobSetBuilder};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// A folder project creates one Vitest project, even when its root contains
/// several valid config files. Explicit file globs use the separate path.
pub(super) fn folder_config_paths(
    specifier: &str,
    declaration_path: &Path,
    visible: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let base = declaration_path.parent().unwrap_or(Path::new("."));
    let pattern = crate::codebase::ts_resolver::normalize_path(
        &base.join(specifier.trim_start_matches("./")),
    );
    let glob = specifier
        .contains(['*', '?', '[', '{'])
        .then(|| visible_folder_config_glob(&slash_path(&pattern)));
    let mut candidates = BTreeMap::<PathBuf, Vec<PathBuf>>::new();
    for path in visible.iter().filter(|path| is_vitest_project_config(path)) {
        let Some(root) = path.parent() else {
            continue;
        };
        let matches = match &glob {
            Some(Ok(glob)) => glob.is_match(slash_path(root)),
            Some(Err(_)) => false,
            None => root == pattern,
        };
        if matches {
            candidates
                .entry(root.to_path_buf())
                .or_default()
                .push(path.clone());
        }
    }
    candidates
        .into_values()
        .filter_map(|paths| paths.into_iter().min_by_key(folder_config_rank))
        .collect()
}

pub(super) fn visible_folder_config_glob(
    specifier: &str,
) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    builder.add(
        GlobBuilder::new(specifier.trim_end_matches('/'))
            .literal_separator(true)
            .build()?,
    );
    builder.build()
}

fn folder_config_rank(path: &PathBuf) -> (u8, String) {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let rank = if name.starts_with("vitest.config") {
        0
    } else if name.starts_with("vite.config") {
        1
    } else if name.starts_with("vitest.") {
        2
    } else {
        3
    };
    (rank, slash_path(path))
}
