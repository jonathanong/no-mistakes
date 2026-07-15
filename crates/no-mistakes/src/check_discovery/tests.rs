use super::*;
use no_mistakes::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[path = "tests/mixed_availability.rs"]
mod mixed_availability;

fn discover_check_file_views(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
) -> super::views::CheckFileViews {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let root_files = no_mistakes::codebase::ts_source::git_visible_files(&root);
    discover_check_file_views_from_git_files(
        &root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
    )
}

fn discover_check_file_views_from_git_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    root_files: Option<Vec<String>>,
) -> super::views::CheckFileViews {
    super::views::discover_check_file_views_with_external_lookup(
        root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
        no_mistakes::codebase::ts_source::git_visible_files,
    )
}

fn fixture(path: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(path)
            .join("fixture"),
    )
}

fn materialized_git_case(path: &str) -> (TempDir, PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(path);
    let case = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(case.path());
    crate::test_support::git_add_all(case.path());
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&case.path().join("fixture"));
    (case, root)
}

fn load_config(root: &Path) -> NoMistakesConfig {
    load_v2_config(root, None).unwrap()
}

fn discover_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
) -> Vec<PathBuf> {
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(root);
    discover_check_file_views_from_snapshot(
        root,
        config,
        skip_directories,
        unique_exports_enabled,
        &snapshot,
    )
    .filesystem
}

fn unique_exports_project_roots(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let mut inferred_roots =
        no_mistakes::codebase::config::InferredRoots::from_visible(root, visible_paths.as_ref());
    super::unique_exports_project_roots_with_inferred(root, config, &mut inferred_roots)
}

fn write(dir: &Path, path: &str, content: &str) {
    let full = dir.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

#[test]
fn unique_exports_project_roots_cover_target_variants() {
    let root = fixture("check-discovery/unique-exports-target-roots");
    let config = load_config(&root);

    let roots = unique_exports_project_roots(&root, &config);

    assert_eq!(
        roots,
        vec![root.clone(), root.join("backend"), root.join("web")]
    );
}

#[test]
fn discover_check_files_includes_inferred_nextjs_project_files() {
    let root = fixture("config-v2/nextjs-inferred-root");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_project_files() {
    let root = fixture("config-v2/remix-inferred-root");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_vite_project_files() {
    let root = fixture("config-v2/remix-vite-inferred-root");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_vitejs_project_files() {
    let root = fixture("config-v2/vitejs-inferred-root");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_does_not_rescan_repository_root() {
    let root = fixture("check-discovery/repository-root-only");
    let config = load_config(&root);
    let mut expected = no_mistakes::codebase::ts_source::discover_files(&root, &[]);
    expected.sort();
    expected.dedup();

    let files = discover_files(&root, &config, &[], true);

    assert_eq!(files, expected);
}

#[cfg(unix)]
#[test]
fn automatic_check_views_exclude_a_tracked_broken_symlink() {
    let root = fixture("codebase-analysis/tests-impact");
    let config = load_config(&root);
    let broken = root.join("broken.test.mts");
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);

    // Keep the lexical entry available for explicit changed-file handling and
    // memoized failure reporting, but do not treat it as an automatic file.
    assert!(sources.inventory().paths().contains(&broken));
    let classification = sources
        .inventory()
        .classification_for_path(&broken)
        .expect("tracked symlink is classified during discovery");
    assert!(classification.is_lexical_symlink());
    assert!(!classification.target_is_file());

    let views = discover_check_file_views_from_snapshot(&root, &config, &[], false, &snapshot);
    assert!(!views.filesystem.contains(&broken));
    assert!(!views.graph.contains(&broken));

    let first = sources.read_path(&broken).unwrap_err();
    let second = sources.read_path(&broken).unwrap_err();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn automatic_check_views_keep_a_file_deleted_after_the_snapshot() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("src/frozen.mts");
    write(
        dir.path(),
        "src/frozen.mts",
        "export const frozen = true;\n",
    );
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    std::fs::remove_file(&file).unwrap();

    let views = discover_check_file_views_from_snapshot(
        dir.path(),
        &NoMistakesConfig::default(),
        &[],
        false,
        &snapshot,
    );
    assert!(views.filesystem.contains(&file));
    assert!(views.graph.contains(&file));

    let first = sources.read_path(&file).unwrap_err();
    let second = sources.read_path(&file).unwrap_err();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn discover_check_files_preserves_included_fixture_roots() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &config.filesystem.skip_directories, false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/users.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("backend/fixtures/backend-users.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("web/fixtures/project-users.json")));
    assert!(!files
        .iter()
        .any(|path| path.ends_with("generated/fixtures/ignored-users.json")));
}

#[test]
fn discover_check_file_views_derive_filesystem_scope_from_complete_universe() {
    let (_case, root) = materialized_git_case("check-discovery/include-preserved-roots");
    let config = load_config(&root);
    let expected_filesystem =
        discover_files(&root, &config, &config.filesystem.skip_directories, false);
    let expected_graph = discover_files(&root, &config, &[], false);

    let views =
        discover_check_file_views(&root, &config, &config.filesystem.skip_directories, false);

    // The graph retains files beneath filesystem-only skips.
    assert!(views
        .graph
        .iter()
        .any(|path| path.ends_with("generated/fixtures/ignored-users.json")));
    // The derived filesystem view still preserves explicit include roots.
    assert!(views
        .filesystem
        .iter()
        .any(|path| path.ends_with("fixtures/users.json")));
    assert!(views
        .filesystem
        .iter()
        .any(|path| path.ends_with("backend/fixtures/backend-users.json")));
    assert!(!views
        .filesystem
        .iter()
        .any(|path| path.ends_with("generated/fixtures/ignored-users.json")));
    assert_eq!(views.filesystem, expected_filesystem);
    assert_eq!(views.graph, expected_graph);
}

#[test]
fn repository_inventory_retains_files_pruned_from_source_views() {
    let fixture = crate::test_support::materialize_gitignore_fixture("banned-paths-source-skips");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let config = load_config(&root);
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let inventory = snapshot.paths_for(&root);
    let views = discover_check_file_views_from_snapshot(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        &snapshot,
    );

    for relative in [
        "build/blocked.patch",
        "dist/blocked.patch",
        "fixtures/blocked.patch",
        "target/blocked.patch",
    ] {
        let path = root.join(relative);
        assert!(inventory.contains(&path), "{relative}");
        assert!(!views.filesystem.contains(&path), "{relative}");
        assert!(!views.graph.contains(&path), "{relative}");
    }
    let nested = root.join("nested/blocked.patch");
    assert!(inventory.contains(&nested));
    assert!(views.filesystem.contains(&nested));
    assert!(views.graph.contains(&nested));
}

#[test]
fn discover_check_file_views_fall_back_outside_git() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);
    let views = discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

    assert!(views
        .filesystem
        .iter()
        .any(|path| path.ends_with("backend/fixtures/backend-users.json")));
    assert!(!views
        .filesystem
        .iter()
        .any(|path| path.ends_with("generated/fixtures/ignored-users.json")));
    assert!(views
        .graph
        .iter()
        .any(|path| path.ends_with("generated/fixtures/ignored-users.json")));
    assert!(!views
        .graph
        .iter()
        .any(|path| path.ends_with(".no-mistakes.yml")));
}

#[test]
fn discover_check_file_views_preserve_unique_export_project_scope() {
    let root = fixture("check-discovery/unique-exports-under-skipped-root");
    let config = load_config(&root);

    let views =
        discover_check_file_views(&root, &config, &config.filesystem.skip_directories, true);

    assert!(views
        .filesystem
        .iter()
        .any(|path| path.ends_with("fixtures/app/src/index.ts")));
    assert!(!views
        .filesystem
        .iter()
        .any(|path| path.ends_with("fixtures/app/generated/skipped.ts")));
    assert!(views
        .graph
        .iter()
        .any(|path| path.ends_with("fixtures/app/generated/skipped.ts")));
}

#[test]
fn discover_check_file_views_scope_normalized_external_unique_export_project() {
    let (_case, root) = materialized_git_case("check-discovery/unique-exports-external-project");
    let config = load_config(&root);
    let expected_filesystem =
        discover_files(&root, &config, &config.filesystem.skip_directories, true);
    let expected_graph = discover_files(&root, &config, &[], true);

    let views =
        discover_check_file_views(&root, &config, &config.filesystem.skip_directories, true);

    assert_eq!(views.filesystem, expected_filesystem);
    assert!(views
        .filesystem
        .iter()
        .any(|path| path.ends_with("external-project/src/index.ts")));
    assert!(!views
        .filesystem
        .iter()
        .any(|path| path.ends_with("external-project/generated/skipped.ts")));
    assert!(views
        .graph
        .iter()
        .any(|path| path.ends_with("external-project/generated/skipped.ts")));
    assert!(!views
        .graph
        .iter()
        .any(|path| path.ends_with("unconfigured-project/leak.ts")));
    assert!(!views
        .graph
        .iter()
        .any(|path| path.ends_with("fixture/fixtures/unrelated.ts")));
    assert_eq!(views.graph, expected_graph);
}

#[test]
fn discover_check_files_preserves_forbidden_workspace_project_roots() {
    let root = fixture("rules/filesystem-dispatch/forbidden-workspace-project-root");
    let config = load_config(&root);

    let files = discover_files(&root, &config, &config.filesystem.skip_directories, false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/app/package.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("packages/domain/package.json")));
    let mut inferred = no_mistakes::codebase::config::InferredRoots::default();
    assert_eq!(
        preserved_project_roots_with_inferred(&root, &config, &mut inferred),
        vec![root.join("fixtures/app")]
    );
}

#[test]
fn nextjs_project_without_single_config_root_is_ignored() {
    let root = fixture("check-discovery/nextjs-without-config");
    let config = load_config(&root);

    let roots = unique_exports_project_roots(&root, &config);

    assert!(roots.is_empty());
}

/// End-to-end regression coverage: `discover_check_files` on a repo with a large
/// gitignored directory whose nested contents would match a `**/<literal>/**` include
/// pattern must not surface those files, and must do so via the git-visible file list
/// rather than a full filesystem walk of the ignored directory.
#[test]
fn discover_check_files_preserves_roots_without_descending_into_gitignored_directory() {
    let dir = TempDir::new().unwrap();
    crate::test_support::git_init(dir.path());
    write(dir.path(), ".gitignore", "dependency-store/\n");
    write(dir.path(), "web/fixtures/tracked.json", "{}");
    write(
        dir.path(),
        "dependency-store/nested/fixtures/trap.json",
        "{}",
    );
    write(
        dir.path(),
        ".no-mistakes.yml",
        "rules:\n  - rule: test-email-domain-policy\n    scope: repository\n    include:\n      - \"**/fixtures/**\"\n    options:\n      bannedDomains:\n        - example.com\n",
    );
    crate::test_support::git_add_all(dir.path());

    let config = load_config(dir.path());
    let files = discover_files(dir.path(), &config, &[], false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("web/fixtures/tracked.json")));
    assert!(!files
        .iter()
        .any(|path| path.starts_with(dir.path().join("dependency-store"))));
}
