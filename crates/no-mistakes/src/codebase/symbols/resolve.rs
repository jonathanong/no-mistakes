fn resolve_root(arg: Option<&Path>, cwd: &Path) -> PathBuf {
    match arg {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => cwd.join(p),
        None => cwd.to_path_buf(),
    }
}

/// Resolve each input file path against `--root` first, falling back to cwd.
#[inline(never)]
fn resolve_input_files(files: &[PathBuf], root: &Path, cwd: &Path) -> Vec<PathBuf> {
    files
        .iter()
        .map(|f| {
            if f.is_absolute() {
                f.clone()
            } else {
                let from_root = root.join(f);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(f)
                }
            }
        })
        .collect()
}
