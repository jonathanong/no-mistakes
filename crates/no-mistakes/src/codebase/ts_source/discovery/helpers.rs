pub(crate) fn is_under_skipped_dir(root: &Path, path: &Path, extra_skip: &HashSet<&str>) -> bool {
    path.strip_prefix(root).ok().is_some_and(|rel| {
        rel.components().any(|component| {
            let name = component.as_os_str().to_str();
            name.is_some_and(|name| SKIP_DIRS.contains(&name) || extra_skip.contains(name))
        })
    })
}

pub fn discover_with_extensions(
    root: &Path,
    extra_skip: &[String],
    extensions: &[&str],
) -> Vec<PathBuf> {
    discover_files(root, extra_skip)
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| extensions.contains(&ext))
        })
        .collect()
}

pub fn discover_with_basenames(
    root: &Path,
    extra_skip: &[String],
    basenames: &[&str],
) -> Vec<PathBuf> {
    discover_files(root, extra_skip)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| basenames.contains(&n))
        })
        .collect()
}

pub fn line_number(source: &str, start: u32) -> usize {
    byte_offset_to_line(source, start as usize) as usize
}
