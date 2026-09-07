use std::path::Path;

pub(super) fn contained_in_root(root: &Path, path: &Path) -> bool {
    if path.strip_prefix(root).is_err() {
        return false;
    }
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(resolved), Ok(resolved_root)) => resolved.strip_prefix(resolved_root).is_ok(),
        _ => true,
    }
}
