/// Demand-driven import traversal used by `dependencies --relationship import`.
/// It parses only roots and files reached through static import edges.
pub fn lazy_import_deps_of(
    roots: &[NodeId],
    root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
) -> Result<Vec<NodeEntry>> {
    let mut graph_files = GraphFiles::discover(root);
    for path in roots.iter().filter_map(NodeId::as_file) {
        graph_files.add_explicit_root(path);
    }
    Ok(lazy_import_deps_of_with_files(
        roots,
        root,
        tsconfig,
        max_depth,
        &graph_files,
        None,
    ))
}

pub(crate) fn lazy_import_deps_of_with_files(
    roots: &[NodeId],
    root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry> {
    let context = TsFactContext::new(root);
    lazy_import_deps_of_with_files_and_facts(
        roots,
        root,
        tsconfig,
        max_depth,
        graph_files,
        allowed,
        LazyImportFacts::new(None, TsFactPlan::imports(), &context),
    )
    .0
}

pub(crate) fn lazy_import_deps_of_with_files_and_facts(
    roots: &[NodeId],
    root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
    facts: LazyImportFacts<'_>,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    let workspace = crate::codebase::workspaces::load_indexed_from_files(root, graph_files.all())
        .unwrap_or_default();
    lazy_import_deps_of_with_files_facts_and_workspace(
        roots,
        tsconfig,
        max_depth,
        graph_files,
        allowed,
        facts,
        &workspace,
    )
}

pub(crate) fn lazy_import_deps_of_with_files_facts_and_workspace(
    roots: &[NodeId],
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
    facts: LazyImportFacts<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    lazy_import_deps_of_with_files_facts_workspace_and_resolution_cache(LazyImportBuild {
        roots,
        tsconfig,
        tsconfig_catalog: None,
        max_depth,
        graph_files,
        allowed,
        facts,
        workspace,
        import_resolution_cache: None,
    })
}
