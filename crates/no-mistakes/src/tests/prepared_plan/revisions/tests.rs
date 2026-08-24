use super::*;
use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
use no_mistakes::codebase::ts_source::FileInventory;

#[test]
fn request_scoped_revision_projection_reuses_the_working_tree_source_store() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-plan/fixture");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    git_init(&root);
    git_commit_all(&root, "base");
    let package = root.join("package.json");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&package),
    ))));
    let args = plan_args(&root, None);
    let revisions = RevisionSources::prepare(&root, &args, Arc::clone(&sources));

    assert!(revisions.read_after(&package).is_some());
    assert!(revisions.read_after(&package).is_some());
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn diff_only_projection_never_substitutes_the_checkout_for_requested_head_content() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-plan/fixture");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    git_init(&root);
    git_commit_all(&root, "base");
    let package = root.join("package.json");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&package),
    ))));
    let args = plan_args(&root, Some("diff --git a/package.json b/package.json\n"));
    let revisions = RevisionSources::prepare(&root, &args, Arc::clone(&sources));

    assert!(revisions.is_diff_only());
    assert!(revisions.read_after(&package).is_none());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn request_scoped_revision_existence_cache_reuses_missing_base_and_head_probes() {
    let (revisions, _fixture) = missing_revision_sources("missing-base", "missing-head");

    assert!(!revisions.base_ref_exists());
    assert!(!revisions.base_ref_exists());
    assert!(!revisions.head_ref_exists());
    assert!(!revisions.head_ref_exists());
    assert_eq!(revisions.ref_probe_count(), 2);
}

#[test]
fn request_scoped_revision_existence_cache_shares_an_identical_base_and_head_probe() {
    let (revisions, _fixture) = missing_revision_sources("missing-ref", "missing-ref");

    assert!(!revisions.base_ref_exists());
    assert!(!revisions.head_ref_exists());
    assert_eq!(revisions.ref_probe_count(), 1);
}

fn missing_revision_sources(base: &str, head: &str) -> (RevisionSources, tempfile::TempDir) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-plan/fixture");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    git_init(&root);
    git_commit_all(&root, "base");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let mut args = plan_args(&root, None);
    args.base = Some(base.to_string());
    args.head = Some(head.to_string());
    (RevisionSources::prepare(&root, &args, sources), fixture)
}

fn plan_args(root: &Path, diff_content: Option<&str>) -> crate::tests::PlanArgs {
    crate::tests::PlanArgs {
        framework: None,
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: Vec::new(),
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: diff_content.map(str::to_string),
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
        include_glob: Vec::new(),
    }
}
