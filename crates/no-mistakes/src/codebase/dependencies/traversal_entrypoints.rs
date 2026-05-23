fn raw_looks_like_source_file(
    raw: &str,
    path: &Path,
    root_dependencies: &std::collections::HashSet<String>,
) -> bool {
    let has_source_extension = Path::new(raw)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
        });
    if !has_source_extension {
        return false;
    }
    if !raw.contains('/') && !raw.contains('\\') {
        return true;
    }
    if raw_package_name(raw).is_some_and(|name| root_dependencies.contains(&name)) {
        return false;
    }
    path.parent().is_some_and(Path::exists)
}

fn raw_package_name(raw: &str) -> Option<String> {
    if raw.starts_with('.') || raw.starts_with('/') {
        return None;
    }
    let mut parts = raw.split('/');
    let first = parts.next()?;
    if first.starts_with('@') {
        let package = parts.next()?;
        return Some(format!("{first}/{package}"));
    }
    Some(first.to_string())
}

fn root_dependency_names(root: &Path) -> std::collections::HashSet<String> {
    let Ok(source) = std::fs::read_to_string(root.join("package.json")) else {
        return std::collections::HashSet::new();
    };
    let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&source) else {
        return std::collections::HashSet::new();
    };
    [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .iter()
    .filter_map(|field| package_json.get(field).and_then(|deps| deps.as_object()))
    .flat_map(|deps| deps.keys().cloned())
    .collect()
}

fn package_dir_entry(
    dir: &Path,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Option<PathBuf> {
    workspace
        .packages
        .iter()
        .find(|package| package.dir == dir)
        .and_then(|package| package.entry.clone())
        .or_else(|| {
            [
                "src/index.mts",
                "src/index.ts",
                "src/index.tsx",
                "src/index.cts",
                "src/index.js",
                "src/index.mjs",
                "src/index.jsx",
                "src/index.cjs",
                "index.mts",
                "index.ts",
                "index.tsx",
                "index.cts",
                "index.js",
                "index.mjs",
                "index.jsx",
                "index.cjs",
            ]
            .iter()
            .map(|candidate| dir.join(candidate))
            .find(|candidate| candidate.is_file())
        })
}
