pub fn load_workspace_globs(root: &Path) -> Result<Vec<String>> {
    let files = crate::codebase::ts_source::discover_visible_paths(root);
    load_workspace_globs_from_files(root, &files)
}

#[doc(hidden)]
pub fn load_workspace_globs_from_files(root: &Path, files: &[PathBuf]) -> Result<Vec<String>> {
    let visible = files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<std::collections::HashSet<_>>();
    let pnpm_path = normalize_path(&root.join("pnpm-workspace.yaml"));
    if visible.contains(&pnpm_path) {
        let content = std::fs::read_to_string(&pnpm_path)?;
        let pnpm_workspace: PnpmWorkspace = serde_yaml::from_str(&content)?;
        return Ok(pnpm_workspace
            .packages
            .unwrap_or_else(|| vec!["*".to_string()]));
    }

    let pkg_path = normalize_path(&root.join("package.json"));
    if visible.contains(&pkg_path) {
        let content = std::fs::read_to_string(&pkg_path)?;
        let root_pkg: PackageJson = serde_json::from_str(&content)?;

        let workspace_globs = match root_pkg.workspaces {
            Some(WorkspacesField::Array(globs)) => globs,
            Some(WorkspacesField::Object { packages }) => packages,
            None => Vec::new(),
        };
        return Ok(workspace_globs);
    }

    Ok(Vec::new())
}

fn build_glob_set(glob_strs: &[String], excluded: bool) -> globset::GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in glob_strs {
        let normalized = normalize_workspace_glob(pattern);
        let pattern = if excluded {
            let Some(stripped) = normalized.strip_prefix('!') else {
                continue;
            };
            stripped
        } else if normalized.starts_with('!') {
            continue;
        } else {
            normalized.as_str()
        };
        let Ok(glob) = GlobBuilder::new(pattern).literal_separator(true).build() else {
            continue;
        };
        builder.add(glob);
    }
    builder
        .build()
        .expect("globset with individually validated globs should build")
}

fn normalize_workspace_glob(pattern: &str) -> String {
    let (negated, pattern) = pattern
        .strip_prefix('!')
        .map_or((false, pattern), |stripped| (true, stripped));
    let normalized = glob_normalize::normalize(pattern);
    if negated {
        format!("!{normalized}")
    } else {
        normalized
    }
}

fn expand_workspace_globs_from_files(
    root: &Path,
    glob_strs: &[String],
    files: &[PathBuf],
) -> Vec<PathBuf> {
    let include = build_glob_set(glob_strs, false);
    let exclude = build_glob_set(glob_strs, true);

    let mut dirs: Vec<PathBuf> = files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| path.parent())
        .filter_map(|dir| {
            let rel = dir.strip_prefix(root).ok()?;
            if include.is_match(rel) && !exclude.is_match(rel) {
                Some(dir.to_path_buf())
            } else {
                None
            }
        })
        .collect();
    dirs.sort();
    dirs.dedup();
    dirs
}
