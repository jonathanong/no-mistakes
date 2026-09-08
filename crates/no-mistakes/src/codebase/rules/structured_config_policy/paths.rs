use std::path::Path;

pub(super) fn canonical_path_in_root(root: &Path, path: &Path) -> Option<std::path::PathBuf> {
    let resolved_root = root.canonicalize().ok()?;
    let resolved_path = path.canonicalize().ok()?;
    resolved_path.strip_prefix(resolved_root).ok()?;
    Some(resolved_path)
}

pub(super) fn contained_in_root(root: &Path, path: &Path) -> bool {
    if path.strip_prefix(root).is_err() {
        return false;
    }
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(resolved), Ok(resolved_root)) => resolved.strip_prefix(resolved_root).is_ok(),
        _ => true,
    }
}
