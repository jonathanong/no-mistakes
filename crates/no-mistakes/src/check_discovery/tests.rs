use super::*;
use no_mistakes::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn fixture(path: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(path)
            .join("fixture"),
    )
}

fn load_config(root: &Path) -> NoMistakesConfig {
    load_v2_config(root, None).unwrap()
}

fn git_init(dir: &Path) {
    let output = Command::new("git")
        .args(["init", "-q", "--initial-branch=main"])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_add_all(dir: &Path) {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    let files = discover_check_files(&root, &config, &[], true, None);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_project_files() {
    let root = fixture("config-v2/remix-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true, None);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_vite_project_files() {
    let root = fixture("config-v2/remix-vite-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true, None);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_vitejs_project_files() {
    let root = fixture("config-v2/vitejs-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true, None);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_does_not_rescan_repository_root() {
    let root = fixture("check-discovery/repository-root-only");
    let config = load_config(&root);
    let mut expected = no_mistakes::codebase::ts_source::discover_files(&root, &[]);
    expected.sort();
    expected.dedup();

    let files = discover_check_files(&root, &config, &[], true, None);

    assert_eq!(files, expected);
}

#[test]
fn discover_check_files_preserves_included_fixture_roots() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);

    let files = discover_check_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

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
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);
    let expected_filesystem = discover_check_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );
    let expected_graph = discover_check_files(&root, &config, &[], false, None);

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
fn discover_check_file_views_fall_back_outside_git() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);
    let expected_filesystem = discover_check_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );
    let expected_graph = discover_check_files(&root, &config, &[], false, None);

    let views = super::views::discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

    assert_eq!(views.filesystem, expected_filesystem);
    assert_eq!(views.graph, expected_graph);
}

#[test]
fn external_project_discovery_falls_back_to_gitignore_aware_walk() {
    let root = fixture("check-discovery/include-preserved-roots");
    let expected = no_mistakes::codebase::ts_source::walk_files(&root, &[]);

    let files = super::views::complete_project_files_from_git(&root, None);

    assert_eq!(files, expected);
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
    let root = fixture("check-discovery/unique-exports-external-project");
    let config = load_config(&root);
    let expected_filesystem = discover_check_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        true,
        None,
    );
    let expected_graph = discover_check_files(&root, &config, &[], true, None);

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

    let files = discover_check_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/app/package.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("packages/domain/package.json")));
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
    git_init(dir.path());
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
    git_add_all(dir.path());

    let config = load_config(dir.path());
    let files = discover_check_files(dir.path(), &config, &[], false, None);

    assert!(files
        .iter()
        .any(|path| path.ends_with("web/fixtures/tracked.json")));
    assert!(!files
        .iter()
        .any(|path| path.starts_with(dir.path().join("dependency-store"))));
}
