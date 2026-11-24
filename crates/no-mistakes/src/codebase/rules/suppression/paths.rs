use std::path::{Path, PathBuf};

pub(super) fn finding_source_candidate(
    root: &Path,
    file: &str,
    allow_absolute: bool,
) -> Option<(PathBuf, bool)> {
    let path = Path::new(file);
    if crate::codebase::ts_source::is_portably_absolute_path(path) {
        return allow_absolute.then(|| (path.to_path_buf(), true));
    }
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Prefix(_) | std::path::Component::RootDir
        )
    }) || (!allow_absolute
        && path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir)))
    {
        return None;
    }
    Some((root.join(path), false))
}

pub(super) fn source_path_for_candidate(
    canonical_root: &Path,
    candidate: PathBuf,
) -> Option<PathBuf> {
    let canonical_candidate = std::fs::canonicalize(&candidate).ok()?;
    let metadata = std::fs::metadata(&canonical_candidate).ok()?;
    (canonical_candidate.starts_with(canonical_root) && metadata.is_file()).then_some(candidate)
}

pub(super) fn markdown_prose(file: &str) -> bool {
    // `.md` and `.markdown` are prose. `.mdx` keeps JavaScript quote tracking.
    Path::new(file)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}
