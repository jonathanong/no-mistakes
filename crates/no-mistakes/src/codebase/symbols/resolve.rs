fn resolve_root(arg: Option<&Path>, cwd: &Path) -> PathBuf {
    match arg {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => cwd.join(p),
        None => cwd.to_path_buf(),
    }
}

/// Load tsconfig from `--tsconfig` if given, else search upward from `root`,
/// else return an empty config.
#[inline(never)]
fn resolve_tsconfig(arg: Option<&Path>, root: &Path) -> Result<TsConfig> {
    if let Some(path) = arg {
        return load_tsconfig(path).context(format!("loading tsconfig {}", path.display()));
    }
    if let Some(path) = find_tsconfig(root) {
        return load_tsconfig(&path).context(format!("loading tsconfig {}", path.display()));
    }
    Ok(TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    })
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
