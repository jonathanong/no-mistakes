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

pub fn relative_slash_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn line_number(source: &str, start: u32) -> usize {
    byte_offset_to_line(source, start as usize) as usize
}
