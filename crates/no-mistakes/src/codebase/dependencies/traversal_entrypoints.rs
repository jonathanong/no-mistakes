struct EntrypointResolution<'a> {
    raw_entrypoints: &'a [PathBuf],
    symbol_entrypoints: &'a [Option<String>],
    structured_entrypoints: &'a [bool],
    root: &'a Path,
    cwd: &'a Path,
    graph_files: &'a graph::GraphFiles,
    include_symbols: bool,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    interner: &'a PathInterner,
}

fn resolve_entrypoints_with_files_and_workspace(
    input: EntrypointResolution<'_>,
) -> Vec<Entrypoint> {
    let EntrypointResolution {
        raw_entrypoints,
        symbol_entrypoints,
        structured_entrypoints,
        root,
        cwd,
        graph_files,
        include_symbols,
        workspace,
        interner,
    } = input;
    let root_dependencies = workspace.root_dependency_names();
    raw_entrypoints
        .iter()
        .enumerate()
        .map(|(index, raw)| {
            let raw_str = raw.to_string_lossy();
            let structured_symbol = symbol_entrypoints.get(index).cloned().flatten();
            let structured_entrypoint = structured_entrypoints.get(index).copied().unwrap_or(false);
            let (raw_file, parsed_symbol) = if structured_entrypoint {
                (raw.clone(), None)
            } else {
                parse_entrypoint(&raw_str)
            };
            let mut symbol = structured_symbol.or(parsed_symbol);
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
            let mut node = resolve_entrypoint_node(
                &raw_for_node,
                &normalized,
                workspace,
                root_dependencies,
                graph_files.visible(),
                interner,
            );
            let file = match &node {
                NodeId::File(path) | NodeId::Symbol { file: path, .. } => path.to_path_buf(),
                _ => normalized,
            };
            if let Some(workflow_node) = symbol
                .as_deref()
                .and_then(|suffix| workflow_node_from_suffix_in(interner, &file, suffix))
            {
                node = workflow_node;
                symbol = None;
            } else if include_symbols {
                if let (NodeId::File(file), Some(symbol)) = (&node, &symbol) {
                    node = NodeId::symbol_in(interner, file.clone(), symbol.clone());
                }
            }
            Entrypoint { file, node, symbol }
        })
        .collect()
}

fn resolve_entrypoint_node(
    raw: &str,
    path: &Path,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    root_dependencies: &std::collections::HashSet<String>,
    visible_files: &crate::fx::PathSet,
    interner: &PathInterner,
) -> NodeId {
    if path.is_dir() {
        if let Some(entry) = package_dir_entry(path, workspace, visible_files) {
            return NodeId::file_in(interner, entry);
        }
    }
    if workspace
        .resolve_specifier_from_visible(raw, visible_files)
        .is_none()
        && raw_package_name(raw).is_some_and(|name| root_dependencies.contains(&name))
    {
        return NodeId::module_in(interner, raw);
    }
    if path.exists() || raw.starts_with('.') || Path::new(raw).is_absolute() {
        return NodeId::file_in(interner, path);
    }
    if let Some(entry) = workspace.resolve_specifier_from_visible(raw, visible_files) {
        return NodeId::file_in(interner, entry);
    }
    if raw_looks_like_source_file(raw, path, root_dependencies) {
        return NodeId::file_in(interner, path);
    }
    NodeId::module_in(interner, raw)
}

fn raw_looks_like_source_file(
    raw: &str,
    path: &Path,
    root_dependencies: &std::collections::HashSet<String>,
) -> bool {
    let has_source_extension = Path::new(raw)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension));
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

fn package_dir_entry(
    dir: &Path,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &crate::fx::PathSet,
) -> Option<PathBuf> {
    workspace
        .package_by_dir(dir)
        .and_then(|package| package.entry.clone())
        .filter(|entry| {
            visible_files.contains(&crate::codebase::ts_resolver::normalize_path(entry))
        })
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
            .find(|candidate| visible_files.contains(candidate))
        })
}
