fn resolve_entrypoints_with_files(
    raw_entrypoints: &[PathBuf],
    symbol_entrypoints: &[Option<String>],
    root: &Path,
    cwd: &Path,
    graph_files: &graph::GraphFiles,
    include_symbols: bool,
) -> Vec<Entrypoint> {
    let workspace =
        crate::codebase::workspaces::load_from_files(root, graph_files.all()).unwrap_or_default();
    let root_dependencies = root_dependency_names(root);
    raw_entrypoints
        .iter()
        .enumerate()
        .map(|(index, raw)| {
            let raw_str = raw.to_string_lossy();
            let structured_symbol = symbol_entrypoints.get(index).cloned().flatten();
            let (raw_file, parsed_symbol) = if structured_symbol.is_some() {
                (raw.clone(), None)
            } else {
                parse_entrypoint(&raw_str)
            };
            let symbol = structured_symbol.or(parsed_symbol);
            let raw_for_node = raw_file.to_string_lossy().to_string();
            let file = if raw_file.is_absolute() {
                raw_file
            } else {
                let from_root = root.join(&raw_file);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(&raw_file)
                }
            };
            let normalized = crate::codebase::ts_resolver::normalize_path(&file);
            let mut node =
                resolve_entrypoint_node(&raw_for_node, &normalized, &workspace, &root_dependencies);
            let file = match &node {
                NodeId::File(path) | NodeId::Symbol { file: path, .. } => path.clone(),
                _ => normalized,
            };
            if include_symbols {
                if let (NodeId::File(file), Some(symbol)) = (&node, &symbol) {
                    node = NodeId::Symbol {
                        file: file.clone(),
                        symbol: symbol.clone(),
                    };
                }
            }
            Entrypoint { file, node, symbol }
        })
        .collect()
}

fn resolve_entrypoint_node(
    raw: &str,
    path: &Path,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    root_dependencies: &std::collections::HashSet<String>,
) -> NodeId {
    if path.is_dir() {
        if let Some(entry) = package_dir_entry(path, workspace) {
            return NodeId::File(entry);
        }
    }
    if workspace.resolve_specifier(raw).is_none()
        && raw_package_name(raw).is_some_and(|name| root_dependencies.contains(&name))
    {
        return NodeId::Module(raw.to_string());
    }
    if path.exists() || raw.starts_with('.') || Path::new(raw).is_absolute() {
        return NodeId::File(path.to_path_buf());
    }
    if let Some(entry) = workspace.resolve_specifier(raw) {
        return NodeId::File(entry);
    }
    if raw_looks_like_source_file(raw, path, root_dependencies) {
        return NodeId::File(path.to_path_buf());
    }
    NodeId::Module(raw.to_string())
}

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
