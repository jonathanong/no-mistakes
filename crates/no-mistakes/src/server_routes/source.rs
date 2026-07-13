use std::path::{Path, PathBuf};

pub(crate) fn discover_source_files_from_visible(
    root: &Path,
    extra_skip: &[String],
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    crate::codebase::ts_source::discover_files_preserving_roots_from_visible(
        root,
        extra_skip,
        &[],
        visible_paths,
    )
    .into_iter()
    .filter(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
            })
    })
    .collect()
}

pub(crate) fn relative_string(root: &Path, path: &Path) -> String {
    crate::codebase::ts_source::relative_slash_path(root, path)
}

pub(crate) fn line_number(source: &str, start: u32) -> usize {
    crate::codebase::ts_source::line_number(source, start)
}
