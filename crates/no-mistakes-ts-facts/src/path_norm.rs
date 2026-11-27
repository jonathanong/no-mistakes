use std::path::{Component, Path, PathBuf};

/// Resolve `.` and `..` components without touching the filesystem.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::with_capacity(path.as_os_str().len());
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                } else {
                    normalized.push(c.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
