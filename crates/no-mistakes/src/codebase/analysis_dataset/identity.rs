use std::path::{Path, PathBuf};

pub(super) fn manifest_key(root: &Path, path: Option<&Path>) -> Option<PathBuf> {
    path.map(|path| {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        super::super::ts_resolver::normalize_path(&path)
    })
}
