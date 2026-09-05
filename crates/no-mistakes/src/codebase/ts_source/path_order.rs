use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// Byte-identity path order used by inventory and visible-membership probes.
///
/// Callers must normalize before probing; this does not collapse `.` / `..`
/// and must not canonicalize.
pub(crate) fn cmp_os_str_paths(left: &Path, right: &Path) -> Ordering {
    left.as_os_str().cmp(right.as_os_str())
}

pub(crate) fn sort_os_str_paths(paths: &mut [PathBuf]) {
    paths.sort_by(|left, right| cmp_os_str_paths(left, right));
}
