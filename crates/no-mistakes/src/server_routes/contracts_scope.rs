fn contract_source_files(
    root: &Path,
    extra_skip: &[String],
    filter: Option<&GlobSet>,
) -> Vec<PathBuf> {
    discover_source_files(root, extra_skip)
        .into_iter()
        .filter(|path| !is_test_file(&relative_string(root, path)))
        .filter(|path| source_file_matches_filter(root, path, filter))
        .collect()
}

fn source_file_matches_filter(root: &Path, path: &Path, filter: Option<&GlobSet>) -> bool {
    filter
        .map(|filter| filter.is_match(path.strip_prefix(root).unwrap_or(path)))
        .unwrap_or(true)
}

fn resolve_tsconfig(root: &Path, explicit: Option<&Path>) -> anyhow::Result<TsConfig> {
    let explicit_path = explicit.is_some();
    let path = match explicit {
        Some(path) if path.is_absolute() => Some(path.to_path_buf()),
        Some(path) => Some(root.join(path)),
        None => find_tsconfig(root),
    };
    match path {
        Some(path) if explicit_path => {
            load_tsconfig(&path).context(format!("loading tsconfig {}", path.display()))
        }
        Some(path) => Ok(load_tsconfig(&path).unwrap_or_else(|_| empty_tsconfig(root))),
        None => Ok(empty_tsconfig(root)),
    }
}

fn empty_tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths_dir: root.to_path_buf(),
        paths: Vec::new(),
        base_url: None,
    }
}

fn build_filter(filters: &[String]) -> anyhow::Result<Option<GlobSet>> {
    if filters.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for filter in filters {
        builder.add(GlobBuilder::new(filter).literal_separator(false).build()?);
    }
    Ok(Some(builder.build()?))
}
