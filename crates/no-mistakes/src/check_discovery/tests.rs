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

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_project_files() {
    let root = fixture("config-v2/remix-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_vite_project_files() {
    let root = fixture("config-v2/remix-vite-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_vitejs_project_files() {
    let root = fixture("config-v2/vitejs-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_does_not_rescan_repository_root() {
    let root = fixture("check-discovery/repository-root-only");
    let config = load_config(&root);
    let mut expected = no_mistakes::codebase::ts_source::discover_files(&root, &[]);
    expected.sort();
    expected.dedup();

    let files = discover_check_files(&root, &config, &[], true);

    assert_eq!(files, expected);
}

#[test]
fn discover_check_files_preserves_included_fixture_roots() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &config.filesystem.skip_directories, false);

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
fn discover_check_files_preserves_forbidden_workspace_project_roots() {
    let root = fixture("rules/filesystem-dispatch/forbidden-workspace-project-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &config.filesystem.skip_directories, false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/app/package.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("packages/domain/package.json")));
}

#[test]
fn literal_include_prefix_stops_before_brace_alternation() {
    assert_eq!(
        literal_include_prefix("docs/{a,b}/**"),
        Some(PathBuf::from("docs"))
    );
    assert_eq!(
        leading_globstar_literal_prefix("**/fixtures/**"),
        Some(PathBuf::from("fixtures"))
    );
    assert_eq!(leading_globstar_literal_prefix("**/*.ts"), None);
    assert!(descendant_dirs_matching_suffix(
        &PathBuf::from("/missing-no-mistakes-fixture-root"),
        &PathBuf::from("fixtures"),
        &[],
        &mut GitFilesCache::new(),
    )
    .is_empty());
}

#[test]
fn include_preserved_roots_ignore_unknown_projects() {
    let root = PathBuf::from("/repo");
    let config = NoMistakesConfig {
        rules: vec![no_mistakes::config::v2::schema::RuleDef {
            rule: "test-email-domain-policy".to_string(),
            projects: vec!["missing".to_string()],
            include: vec!["fixtures/**".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    assert_eq!(
        include_preserved_roots(&root, &config, &[]),
        vec![root.join("fixtures")]
    );
}

#[test]
fn nextjs_project_without_single_config_root_is_ignored() {
    let root = fixture("check-discovery/nextjs-without-config");
    let config = load_config(&root);

    let roots = unique_exports_project_roots(&root, &config);

    assert!(roots.is_empty());
}

#[test]
fn descendant_dirs_matching_suffix_from_files_stops_descent_at_skip_dir() {
    let base = PathBuf::from("/repo");
    let files = vec!["generated/fixtures/ignored.json".to_string()];

    let roots = descendant_dirs_matching_suffix_from_files(
        &base,
        &PathBuf::from("fixtures"),
        &files,
        &["generated".to_string()],
    );

    assert!(roots.is_empty());
}

#[test]
fn descendant_dirs_matching_suffix_from_files_finds_nested_match_past_non_skip_dir() {
    let base = PathBuf::from("/repo");
    let files = vec!["backend/fixtures/users.json".to_string()];

    let roots =
        descendant_dirs_matching_suffix_from_files(&base, &PathBuf::from("fixtures"), &files, &[]);

    assert_eq!(roots, vec![base.join("backend/fixtures")]);
}

/// Regression test for the preserved-root discovery walk visiting large gitignored
/// directories (e.g. a dependency store) instead of deriving candidates from the
/// git-visible file list. Before the fix, `descendant_dirs_matching_suffix` always did
/// a raw recursive `std::fs::read_dir` walk with no `.gitignore` awareness, so a
/// directory name matching an include pattern's suffix (here "fixtures") anywhere
/// under a large ignored directory would still be visited and returned as a preserved
/// root, even though none of its contents are ever git-visible and thus can never
/// appear in the final discovered-file list.
#[test]
fn descendant_dirs_matching_suffix_does_not_walk_gitignored_directory() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "dependency-store/\n");
    write(dir.path(), "web/fixtures/tracked.json", "{}");
    write(
        dir.path(),
        "dependency-store/nested/fixtures/trap.json",
        "{}",
    );
    git_add_all(dir.path());

    let mut cache = GitFilesCache::new();
    let roots =
        descendant_dirs_matching_suffix(dir.path(), &PathBuf::from("fixtures"), &[], &mut cache);

    assert!(roots.contains(&dir.path().join("web/fixtures")));
    assert!(!roots
        .iter()
        .any(|root| root.starts_with(dir.path().join("dependency-store"))));
}

/// Directories that only exist on disk (git never tracks empty directories) must not
/// surface as preserved roots via the git-derived path: there is no git-visible file
/// under them, so nothing would ever be un-skipped by preserving them. This is the
/// clearest observable proof that the git-derived path — not the raw filesystem walk —
/// executed when git is available, since the raw walk would find this directory too.
#[test]
fn descendant_dirs_matching_suffix_ignores_disk_only_empty_directory() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "web/fixtures/tracked.json", "{}");
    std::fs::create_dir_all(dir.path().join("empty-branch/fixtures")).unwrap();
    git_add_all(dir.path());

    let mut cache = GitFilesCache::new();
    let roots =
        descendant_dirs_matching_suffix(dir.path(), &PathBuf::from("fixtures"), &[], &mut cache);

    assert!(roots.contains(&dir.path().join("web/fixtures")));
    assert!(!roots.contains(&dir.path().join("empty-branch/fixtures")));
}

/// Outside a git repository, `descendant_dirs_matching_suffix` still falls back to the
/// raw filesystem walk (exercising `collect_descendant_dirs_matching_suffix`'s match
/// and skip-descent logic directly), since there is no git-visible file list to derive
/// candidates from.
#[test]
fn descendant_dirs_matching_suffix_falls_back_to_walk_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    // A plain file alongside the walked directories exercises the raw walk's
    // "skip non-directory entries" branch, since "fixtures" itself is a hardcoded
    // skip dir and would otherwise prune descent before any file is ever seen.
    write(dir.path(), "README.md", "");
    write(dir.path(), "backend/components/button.tsx", "");
    write(dir.path(), "generated/components/ignored.tsx", "");

    let mut cache = GitFilesCache::new();
    let roots = descendant_dirs_matching_suffix(
        dir.path(),
        &PathBuf::from("components"),
        &["generated".to_string()],
        &mut cache,
    );

    assert_eq!(roots, vec![dir.path().join("backend/components")]);
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
    let files = discover_check_files(dir.path(), &config, &[], false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("web/fixtures/tracked.json")));
    assert!(!files
        .iter()
        .any(|path| path.starts_with(dir.path().join("dependency-store"))));
}
