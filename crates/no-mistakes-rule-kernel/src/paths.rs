use no_mistakes_ts_facts::path_norm::normalize_path;
use std::path::{Path, PathBuf};

pub fn normalize_discovery_path(path: &Path) -> PathBuf {
    let normalized = normalize_path(path);
    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

pub fn relative_slash_path(root: &Path, path: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(root) {
        return relative.to_string_lossy().replace('\\', "/");
    }
    // `Path::canonicalize` on Windows prefixes `\\?\`, so the check root and a
    // config path can name the same file and still fail `strip_prefix`.
    let root = without_verbatim_prefix(&root.to_string_lossy().replace('\\', "/"));
    let path = without_verbatim_prefix(&path.to_string_lossy().replace('\\', "/"));
    if root.is_empty() {
        return path;
    }
    if path == root {
        return String::new();
    }
    let prefix = format!("{root}/");
    path.strip_prefix(&prefix).unwrap_or(&path).to_string()
}

fn without_verbatim_prefix(path: &str) -> String {
    let Some(rest) = path.strip_prefix("//?/") else {
        return path.to_string();
    };
    if let Some(unc) = rest.strip_prefix("UNC/") {
        format!("//{unc}")
    } else {
        rest.to_string()
    }
}

/// Detect absolute paths from either native or Windows syntax. Finding paths
/// are external input at suppression boundaries, so host-native `Path`
/// parsing alone is not enough to recognize a foreign Windows absolute path.
pub fn is_portably_absolute_path(path: &Path) -> bool {
    let value = path.to_string_lossy().replace('\\', "/");
    value.starts_with('/')
        || value.starts_with("//")
        || (value.len() >= 3
            && value.as_bytes()[0].is_ascii_alphabetic()
            && value.as_bytes()[1] == b':'
            && value.as_bytes()[2] == b'/')
}
