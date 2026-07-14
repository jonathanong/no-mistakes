use crate::playwright::analysis::app_collect::collect_app_selector_occurrences_from_visible;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{
    build_globset, relative_string, walk_files_from_snapshot, VisiblePathSnapshot,
};
use crate::playwright::selectors;
use crate::playwright::test_support::fixture_path;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

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

fn collect_app_selectors(
    root: &Path,
    settings: &Settings,
    selector_regexes: &selectors::SelectorRegexes,
) -> Result<Vec<selectors::AppSelector>> {
    let mut app_selectors = collect_app_selector_occurrences(root, settings, selector_regexes)?;
    app_selectors.sort();
    app_selectors.dedup();
    Ok(app_selectors)
}

fn collect_app_selector_occurrences(
    root: &Path,
    settings: &Settings,
    selector_regexes: &selectors::SelectorRegexes,
) -> Result<Vec<selectors::AppSelector>> {
    let snapshot = VisiblePathSnapshot::new(root);
    collect_app_selector_occurrences_from_visible(root, settings, selector_regexes, &snapshot)
}

fn walk_files(root: &Path) -> Vec<std::path::PathBuf> {
    let snapshot = VisiblePathSnapshot::new(root);
    walk_files_from_snapshot(root, &snapshot)
}

#[test]
fn skipped_directories_are_detected() {
    use crate::playwright::fsutil::is_skipped_dir;
    use std::path::Path;
    assert!(is_skipped_dir(Path::new("node_modules")));
    assert!(!is_skipped_dir(Path::new("src")));
}

#[test]
fn build_globset_rejects_invalid_patterns() {
    assert!(build_globset(&["[".to_string()]).is_err());
}

#[test]
fn walk_files_returns_files_and_skips_configured_directories() {
    let root = fixture_path(&["ast-snippets", "main", "walk-files"]);
    let files: Vec<String> = walk_files(&root)
        .into_iter()
        .map(|path| relative_string(&root, &path))
        .collect();
    assert_eq!(files, vec!["src/a.ts", "src/b.ts"]);
}

#[test]
fn snapshot_from_paths_keeps_caller_candidates_authoritative() {
    let root = fixture_path(&["ast-snippets", "main", "walk-files"]);
    let only_candidate = root.join("src/a.ts");
    let snapshot = VisiblePathSnapshot::from_paths(&root, std::slice::from_ref(&only_candidate));

    assert_eq!(
        walk_files_from_snapshot(&root, &snapshot),
        vec![crate::codebase::ts_resolver::normalize_path(
            &only_candidate
        )]
    );
}

#[test]
fn collect_app_selectors_skips_missing_roots_and_non_source_files() {
    let root = fixture_path(&["ast-snippets", "main", "selector-source"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["missing".to_string(), "web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };

    let selector_regexes = selectors::compile_selector_regexes(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
    );
    let selectors = collect_app_selectors(&root, &settings, &selector_regexes).unwrap();
    assert_eq!(selectors.len(), 1);
    assert_eq!(selectors[0].display_value(), "save");
}

#[test]
fn collect_app_selector_occurrences_rejects_invalid_include_and_exclude_globs() {
    let root = fixture_path(&["ast-snippets", "main", "selector-source"]);
    let selector_regexes =
        selectors::compile_selector_regexes(&["data-testid".to_string()], &BTreeMap::new());
    let base = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };

    let mut invalid_include = base.clone();
    invalid_include.selector_include = vec!["[".to_string()];
    assert!(collect_app_selector_occurrences(&root, &invalid_include, &selector_regexes).is_err());

    let mut invalid_exclude = base.clone();
    invalid_exclude.selector_exclude = vec!["[".to_string()];
    assert!(collect_app_selector_occurrences(&root, &invalid_exclude, &selector_regexes).is_err());
}

#[test]
fn collect_app_selectors_honors_include_and_exclude_globs() {
    let root = fixture_path(&["ast-snippets", "main", "selector-source"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec!["web/app/**".to_string()],
        selector_exclude: vec!["web/app/**".to_string()],
    };
    let selector_regexes = selectors::compile_selector_regexes(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
    );

    let selectors = collect_app_selectors(&root, &settings, &selector_regexes).unwrap();

    assert!(selectors.is_empty());
}

/// Regression test for `walk_files` doing a raw, `.gitignore`-blind recursive
/// walk instead of deriving candidates from the git-visible file list. Before
/// the fix, a matching file placed inside a gitignored directory would still
/// be visited and returned, even though no discovery consumer of `walk_files`
/// would ever see it survive `git ls-files`.
#[test]
fn walk_files_prefers_git_visible_files_over_gitignored_directory() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "vendor/\n");
    write(
        dir.path(),
        "src/App.tsx",
        "export default function App() {}\n",
    );
    write(
        dir.path(),
        "vendor/nested/Trap.tsx",
        "export default function Trap() {}\n",
    );
    git_add_all(dir.path());

    let files: Vec<String> = walk_files(dir.path())
        .into_iter()
        .map(|path| relative_string(dir.path(), &path))
        .collect();

    assert_eq!(files, vec![".gitignore", "src/App.tsx"]);
}

/// The git-derived path still applies `is_skipped_dir`: a directory on the
/// hardcoded denylist must be excluded even when git tracks it directly
/// (i.e. it is not merely relying on `.gitignore` to prune it).
#[test]
fn walk_files_still_skips_hardcoded_dirs_when_git_tracked() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/App.tsx", "");
    write(dir.path(), "node_modules/pkg/index.tsx", "");
    git_add_all(dir.path());

    let files: Vec<String> = walk_files(dir.path())
        .into_iter()
        .map(|path| relative_string(dir.path(), &path))
        .collect();

    assert_eq!(files, vec!["src/App.tsx"]);
}

/// Outside a Git repository, `walk_files` still applies its hardcoded skip
/// directories to the shared ignore-aware candidate list.
#[test]
fn walk_files_applies_skip_dirs_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/App.tsx", "");
    write(dir.path(), "node_modules/pkg/index.tsx", "");

    let files: Vec<String> = walk_files(dir.path())
        .into_iter()
        .map(|path| relative_string(dir.path(), &path))
        .collect();

    assert_eq!(files, vec!["src/App.tsx"]);
}

#[test]
fn walk_files_applies_gitignore_outside_git() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");

    let files: Vec<String> = walk_files(dir.path())
        .iter()
        .map(|path| relative_string(dir.path(), path))
        .collect();

    assert!(files.contains(&"app/page.tsx".to_string()));
    assert!(!files.contains(&"ignored/page.tsx".to_string()));
}

#[test]
fn snapshot_normalizes_parent_components_for_descendant_roots() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");
    let snapshot = VisiblePathSnapshot::new(dir.path());
    let root = dir.path().join("app/../app");

    let files: Vec<String> = walk_files_from_snapshot(&root, &snapshot)
        .iter()
        .map(|path| relative_string(dir.path(), path))
        .collect();

    assert_eq!(files, vec!["app/page.tsx"]);
}

#[test]
fn snapshot_discovers_descendant_git_worktrees_independently() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");
    let nested = dir.path().join("packages/visible");
    git_init(&nested);
    git_add_all(&nested);
    git_init(dir.path());
    let snapshot = VisiblePathSnapshot::new(dir.path());

    let files: Vec<String> = walk_files_from_snapshot(&nested, &snapshot)
        .iter()
        .map(|path| relative_string(&nested, path))
        .collect();

    assert_eq!(files, vec!["package.json"]);
}
