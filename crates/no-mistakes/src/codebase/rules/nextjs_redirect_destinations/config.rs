use super::options::{DEFAULT_APP_ROOT, DEFAULT_CONFIG_NAMES};
use std::path::{Path, PathBuf};

pub(super) fn resolve_config_path(
    target_root: &Path,
    files: &[PathBuf],
    config_path: Option<&str>,
) -> Option<PathBuf> {
    if let Some(rel) = config_path.map(str::trim).filter(|value| !value.is_empty()) {
        let rel = rel.trim_start_matches("./");
        let candidate = normalize_option_path(target_root, rel);
        return files
            .iter()
            .find(|path| crate::codebase::ts_resolver::normalize_path(path) == candidate)
            .cloned();
    }
    for name in DEFAULT_CONFIG_NAMES {
        let candidate = crate::codebase::ts_resolver::normalize_path(&target_root.join(name));
        if let Some(path) = files
            .iter()
            .find(|path| crate::codebase::ts_resolver::normalize_path(path) == candidate)
        {
            return Some(path.clone());
        }
    }
    None
}

fn normalize_option_path(target_root: &Path, rel: &str) -> PathBuf {
    let path = Path::new(rel);
    if path.is_absolute() {
        crate::codebase::ts_resolver::normalize_path(path)
    } else {
        crate::codebase::ts_resolver::normalize_path(&target_root.join(path))
    }
}

pub(super) fn resolved_app_root(target_root: &Path, app_root: Option<&str>) -> PathBuf {
    let app_root = normalize_app_root(app_root);
    crate::codebase::ts_resolver::normalize_path(&target_root.join(app_root))
}

pub(super) fn display_app_root(app_root: Option<&str>) -> String {
    let app_root = normalize_app_root(app_root);
    if app_root.is_empty() {
        DEFAULT_APP_ROOT.to_string()
    } else {
        app_root.replace('\\', "/")
    }
}

fn normalize_app_root(app_root: Option<&str>) -> &str {
    app_root
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_APP_ROOT)
        .trim_start_matches("./")
        .trim_end_matches('/')
}
