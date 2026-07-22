#[test]
fn test_edges_source_finds_test_file() {
    let root = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis").join("test-framework").join("fixture"));
    let tsconfig = TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None };
    let graph = build_graph(&root, &tsconfig);
    let index_mts = root.join("src/index.mts");
    let index_test = root.join("src/index.test.mts");
    let testof_filter: HashSet<EdgeKind> = [EdgeKind::TestOf].into();
    let dependents = graph.dependents_of(&[NodeId::File(index_mts.clone())], None, Some(&testof_filter));
    assert!(dependents.iter().any(|e| e.node.as_file() == Some(index_test.as_path())));
    let deps = graph.deps_of(&[NodeId::File(index_mts)], None, Some(&testof_filter));
    assert!(!deps.iter().any(|e| e.node.as_file() == Some(index_test.as_path())));
}

#[test]
fn md_edges_added_for_codebase_intel_fixture() {
    let root = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis").join("codebase-intel").join("fixture"));
    let tsconfig = TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None };
    let graph = build_graph(&root, &tsconfig);
    let deps = graph.deps_of(&[NodeId::File(root.join("README.md"))], None, Some(&[EdgeKind::MarkdownLink].into()));
    let linked_file = root.join("packages/api/src/index.mts");
    assert!(deps.iter().any(|e| e.node.as_file() == Some(linked_file.as_path())));
}
