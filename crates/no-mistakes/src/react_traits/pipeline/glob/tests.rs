use super::test_support::*;
use super::*;
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

#[test]
fn empty_patterns_return_no_files() {
    let files = expand_globs(Path::new("."), &[]).expect("empty globs should succeed");
    assert!(files.is_empty());
}

#[test]
fn skip_dir_matches_generated_and_dependency_directories() {
    for name in [
        ".git",
        ".next",
        ".hidden",
        "node_modules",
        "target",
        "dist",
        "build",
        "coverage",
    ] {
        assert!(is_skip_dir(Path::new(name)), "{name}");
    }
    assert!(!is_skip_dir(Path::new("app")));
}

#[test]
fn dot_directories_excluded_from_glob_expansion() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-glob/skip-dot-directories/fixture");
    let files =
        expand_globs(&root, &["**/*.tsx".to_string()]).expect("glob expansion should succeed");
    let names: Vec<&str> = files
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();
    assert!(names.contains(&"Button.tsx"), "should find Button.tsx");
    assert!(names.contains(&"Card.tsx"), "should find Card.tsx");
    assert!(
        !names.contains(&"Stale.tsx"),
        "should not find Stale.tsx in .hidden/"
    );
    assert!(
        !names.contains(&"Component.tsx"),
        "should not find Component.tsx in dot directories"
    );
}

/// Regression test for `expand_globs` doing a raw, `.gitignore`-blind
/// recursive walk instead of deriving candidates from the git-visible file
/// list. Before the fix, a matching file placed inside a gitignored
/// directory would still be visited and returned, even though it could never
/// appear in `git ls-files` output.
#[test]
fn expand_globs_prefers_git_visible_files_over_gitignored_directory() {
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

    let files =
        expand_globs(dir.path(), &["**/*.tsx".to_string()]).expect("glob expansion should succeed");
    let names: Vec<&str> = files
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();

    assert_eq!(names, vec!["App.tsx"]);
}

/// The git-derived path still applies `is_skip_dir`: a dot-prefixed
/// directory must be excluded even when git tracks it directly (i.e. it is
/// not merely relying on `.gitignore` to prune it).
#[test]
fn expand_globs_still_skips_hardcoded_dirs_when_git_tracked() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(
        dir.path(),
        "src/App.tsx",
        "export default function App() {}\n",
    );
    write(
        dir.path(),
        "node_modules/pkg/Index.tsx",
        "export default function Index() {}\n",
    );
    git_add_all(dir.path());

    let files =
        expand_globs(dir.path(), &["**/*.tsx".to_string()]).expect("glob expansion should succeed");
    let names: Vec<&str> = files
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();

    assert_eq!(names, vec!["App.tsx"]);
}

/// Outside a Git repository, React-specific skip-directory pruning still
/// applies to the shared ignore-aware candidate list.
#[test]
fn expand_globs_applies_skip_dirs_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("project");
    write(&root, "src/App.tsx", "export default function App() {}\n");
    write(
        &root,
        "node_modules/pkg/Index.tsx",
        "export default function Index() {}\n",
    );

    let files =
        expand_globs(&root, &["**/*.tsx".to_string()]).expect("glob expansion should succeed");
    let names: Vec<&str> = files
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();

    assert_eq!(names, vec!["App.tsx"]);
}

#[test]
fn expand_globs_applies_gitignore_outside_git() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");

    let files = expand_globs(dir.path(), &["**/*.tsx".to_string()]).unwrap();

    assert!(files.contains(&dir.path().join("app/page.tsx")));
    assert!(!files.contains(&dir.path().join("ignored/page.tsx")));
}

#[test]
fn expand_globs_accepts_root_with_parent_components() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-glob/skip-dot-directories/fixture")
        .join("src/..");

    let files = expand_globs(&root, &["**/*.tsx".to_string()]).unwrap();

    assert!(files.iter().any(|path| path.ends_with("src/Button.tsx")));
}
