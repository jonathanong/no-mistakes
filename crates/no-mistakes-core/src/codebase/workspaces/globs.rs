pub fn load_workspace_globs(root: &Path) -> Result<Vec<String>> {
    let pnpm_path = root.join("pnpm-workspace.yaml");
    if pnpm_path.exists() {
        let content = std::fs::read_to_string(&pnpm_path)?;
        let pnpm_workspace: PnpmWorkspace = serde_yaml::from_str(&content)?;
        return Ok(pnpm_workspace
            .packages
            .unwrap_or_else(|| vec!["*".to_string()]));
    }

    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
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
        let pattern = if excluded {
            let Some(stripped) = pattern.strip_prefix('!') else {
                continue;
            };
            stripped
        } else if pattern.starts_with('!') {
            continue;
        } else {
            pattern.as_str()
        };
        let Ok(glob) = Glob::new(pattern) else {
            continue;
        };
        builder.add(glob);
    }
    builder
        .build()
        .expect("globset with individually validated globs should build")
}

fn expand_workspace_globs(root: &Path, glob_strs: &[String]) -> Vec<PathBuf> {
    let include = build_glob_set(glob_strs, false);
    let exclude = build_glob_set(glob_strs, true);

    let mut dirs = Vec::new();

    let glob_depth = glob_strs
        .iter()
        .filter(|pattern| !pattern.starts_with('!'))
        .map(|pattern| {
            if pattern.contains("**") {
                usize::MAX
            } else {
                pattern.split('/').count().max(1)
            }
        })
        .max()
        .unwrap_or(1);
    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(glob_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let rel = entry
            .path()
            .strip_prefix(root)
            .expect("walkdir entries are rooted under the walk root");
        if include.is_match(rel) && !exclude.is_match(rel) {
            dirs.push(entry.into_path());
        }
    }

    dirs
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
