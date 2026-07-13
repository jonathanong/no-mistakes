/// Build the conservative runtime-import graph used by Playwright route
/// reachability. This intentionally does not apply ordinary call-scope
/// pruning: a literal dynamic import anywhere in a route-reachable module may
/// be executed at runtime, even when the static call graph cannot prove it.
fn collect_route_import_edges(
    files: &[PathBuf],
    facts: &dyn TsFactLookup,
    tsconfig: &TsConfig,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    let resolver = ImportResolver::new(tsconfig);
    let import_files = files
        .par_iter()
        .filter_map(|path| facts.get_ts_facts(path).map(|file_facts| (path, file_facts)))
        .filter(|(_, file_facts)| file_facts.parse_error.is_none())
        .filter(|(_, file_facts)| {
            file_facts
                .imports
                .iter()
                .any(|import| matches!(import.kind, ImportKind::Static | ImportKind::Dynamic))
        })
        .collect::<Vec<_>>();

    // Resolve each distinct import-source directory once. This preserves
    // `..` semantics through ancestor directory symlinks without restoring the
    // former per-import realpath storm.
    let source_directories = import_files
        .iter()
        .filter_map(|(path, _)| path.parent().map(Path::to_path_buf))
        .collect::<std::collections::BTreeSet<_>>();
    let canonical_directories = source_directories
        .into_par_iter()
        .filter_map(|directory| match directory.canonicalize() {
            Ok(canonical) => Some((directory, canonical)),
            Err(_) => None,
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    // A symlink-resolved target may live in a directory with no imports of its
    // own. Index all visible files by name in memory, then touch only the small
    // same-name candidate set when a real-to-visible remap is actually needed.
    let mut visible_by_name = std::collections::BTreeMap::<
        std::ffi::OsString,
        Vec<PathBuf>,
    >::new();
    let mut visible_files = graph_files.indexable().to_vec();
    visible_files.sort();
    for visible in visible_files {
        if let Some(name) = visible.file_name() {
            visible_by_name
                .entry(name.to_os_string())
                .or_default()
                .push(visible);
        }
    }

    import_files
        .par_iter()
        .flat_map_iter(|(path, file_facts)| {
            let resolution_source =
                route_import_resolution_source(path, &canonical_directories);
            file_facts
                .imports
                .iter()
                .filter(|import| matches!(import.kind, ImportKind::Static | ImportKind::Dynamic))
                .filter_map(|import| resolver.resolve(&import.specifier, &resolution_source))
                .filter_map(|target| {
                    route_import_visible_target(target, graph_files, &visible_by_name)
                })
                .filter(|target| is_indexable(target))
                .map(|target| {
                    (
                        NodeId::File((*path).clone()),
                        NodeId::File(target),
                        EdgeKind::RouteImport,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn route_import_resolution_source(
    path: &Path,
    canonical_directories: &std::collections::BTreeMap<PathBuf, PathBuf>,
) -> PathBuf {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            return match path.canonicalize() {
                Ok(canonical) => canonical,
                Err(_) => path.to_path_buf(),
            };
        }
    }
    let Some(parent) = path.parent() else {
        return path.to_path_buf();
    };
    let Some(canonical_parent) = canonical_directories.get(parent) else {
        return path.to_path_buf();
    };
    let Some(name) = path.file_name() else {
        return path.to_path_buf();
    };
    canonical_parent.join(name)
}

fn route_import_visible_target(
    target: PathBuf,
    graph_files: &GraphFiles,
    visible_by_name: &std::collections::BTreeMap<std::ffi::OsString, Vec<PathBuf>>,
) -> Option<PathBuf> {
    if graph_files.is_visible(&target) {
        return Some(target);
    }
    let canonical_target = target.canonicalize().ok()?;
    let candidates = visible_by_name.get(target.file_name()?)?;
    candidates
        .iter()
        .find(|visible| visible.canonicalize().ok().as_ref() == Some(&canonical_target))
        .cloned()
}
