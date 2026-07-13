use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::playwright::config::Settings;
use crate::playwright::fsutil::VisiblePathSnapshot;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn selector_rel_by_file(
    root: &Path,
    selector_files: &[PathBuf],
) -> HashMap<PathBuf, Arc<String>> {
    let mut by_file = HashMap::new();
    for file in selector_files {
        let relative = Arc::new(crate::playwright::fsutil::relative_string(root, file));
        by_file.insert(normalize_path(file), Arc::clone(&relative));
        if let Ok(canonical) = file.canonicalize() {
            by_file.insert(normalize_path(&canonical), relative);
        }
    }
    by_file
}

pub(super) fn collect(
    root: &Path,
    settings: &Settings,
    snapshot: &VisiblePathSnapshot,
) -> HashSet<PathBuf> {
    let mut visible = HashSet::new();
    extend(&mut visible, &snapshot.paths_for(root));
    extend(
        &mut visible,
        &snapshot.paths_for(&root.join(&settings.frontend_root)),
    );
    for selector_root in &settings.selector_roots {
        extend(&mut visible, &snapshot.paths_for(&root.join(selector_root)));
    }
    visible
}

fn extend(visible: &mut HashSet<PathBuf>, paths: &[PathBuf]) {
    for path in paths {
        visible.insert(normalize_path(path));
        if let Ok(canonical) = path.canonicalize() {
            visible.insert(normalize_path(&canonical));
        }
    }
}

pub(super) fn resolve_tsconfig(
    root: &Path,
    frontend_root: &Path,
    visible: &HashSet<PathBuf>,
) -> Result<TsConfig> {
    find_tsconfig(frontend_root, visible)
        .or_else(|| find_tsconfig(root, visible))
        .map(|path| crate::codebase::ts_resolver::load_tsconfig(&path))
        .transpose()
        .map(|config| config.unwrap_or_else(|| empty_tsconfig(root)))
}

fn find_tsconfig(start: &Path, visible: &HashSet<PathBuf>) -> Option<PathBuf> {
    let mut current = normalize_path(start);
    loop {
        let candidate = current.join("tsconfig.json");
        if visible.contains(&candidate) {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn empty_tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    }
}

pub(super) fn route_entry_files(
    root: &Path,
    settings: &Settings,
    route_file: &Path,
    visible: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let frontend_root = normalize_path(&root.join(&settings.frontend_root));
    let route_file = normalize_path(route_file);
    let mut files = visible
        .contains(&route_file)
        .then_some(route_file.clone())
        .into_iter()
        .collect::<Vec<_>>();
    let mut current = route_file.parent();
    while let Some(parent) = current.filter(|parent| parent.starts_with(&frontend_root)) {
        for stem in ["layout", "template"] {
            for extension in ["tsx", "ts", "jsx", "js"] {
                let wrapper = parent.join(format!("{stem}.{extension}"));
                if visible.contains(&wrapper) {
                    files.push(wrapper);
                }
            }
        }
        current = parent.parent();
    }
    files
}
