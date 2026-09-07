use std::io;
use std::path::Path;

/// Canonical containment is fail-closed so missing paths and symlink failures
/// cannot turn an out-of-repository reference into an allowed one.
pub(super) fn verify(root: &Path, candidate: &Path) -> io::Result<bool> {
    let root = root.canonicalize()?;
    let candidate = candidate.canonicalize()?;
    Ok(candidate.strip_prefix(root).is_ok())
}
