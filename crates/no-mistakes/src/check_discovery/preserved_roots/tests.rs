use super::*;
use ignore::WalkBuilder;
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

type GitFilesCache = HashMap<PathBuf, Option<Vec<String>>>;

fn include_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut git_files_cache = GitFilesCache::new();
    let mut inferred_roots = no_mistakes::codebase::config::InferredRoots::default();
    collect_preserved_roots(root, config, &mut inferred_roots, |roots, base, include| {
        push_include_preserved_roots(roots, base, include, skip_directories, &mut git_files_cache);
    })
}

fn push_include_preserved_roots(
    roots: &mut Vec<PathBuf>,
    base: &Path,
    include: &str,
    skip_directories: &[String],
    git_files_cache: &mut GitFilesCache,
) {
    if let Some(prefix) = literal_include_prefix(include) {
        roots.push(base.join(&prefix));
    }
    if let Some(suffix) = leading_globstar_literal_prefix(include) {
        roots.extend(descendant_dirs_matching_suffix(
            base,
            &suffix,
            skip_directories,
            git_files_cache,
        ));
    }
}

fn descendant_dirs_matching_suffix(
    base: &Path,
    suffix: &Path,
    skip_directories: &[String],
    git_files_cache: &mut GitFilesCache,
) -> Vec<PathBuf> {
    let git_files = git_files_cache
        .entry(base.to_path_buf())
        .or_insert_with(|| no_mistakes::codebase::ts_source::git_visible_files(base));
    match git_files {
        Some(files) => descendant_dirs_matching_suffix_from_files(
            base,
            suffix,
            files.as_slice(),
            skip_directories,
        ),
        None => {
            let mut roots = Vec::new();
            collect_descendant_dirs_matching_suffix(
                base,
                base,
                suffix,
                skip_directories,
                &mut roots,
            );
            roots
        }
    }
}

fn descendant_dirs_matching_suffix_from_files(
    base: &Path,
    suffix: &Path,
    files: &[String],
    skip_directories: &[String],
) -> Vec<PathBuf> {
    super::matching::descendant_dirs_matching_suffix_from_paths(
        base,
        suffix,
        files.iter().map(Path::new),
        skip_directories,
    )
}

fn collect_descendant_dirs_matching_suffix(
    base: &Path,
    dir: &Path,
    suffix: &Path,
    skip_directories: &[String],
    roots: &mut Vec<PathBuf>,
) {
    let base = base.to_path_buf();
    let suffix = suffix.to_path_buf();
    let skip_directories = skip_directories.to_vec();
    let matches = Arc::new(Mutex::new(Vec::new()));
    let matches_for_filter = Arc::clone(&matches);
    let filter_base = base.clone();
    let filter_suffix = suffix.clone();

    let mut builder = WalkBuilder::new(dir);
    builder
        .hidden(true)
        .require_git(false)
        .filter_entry(move |entry| {
            if entry.depth() == 0
                || !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
            {
                return true;
            }
            let path = entry.path();
            if path
                .strip_prefix(&filter_base)
                .ok()
                .is_some_and(|rel| rel.ends_with(&filter_suffix))
            {
                matches_for_filter
                    .lock()
                    .expect("preserved-root match lock should not be poisoned")
                    .push(path.to_path_buf());
            }
            let name = entry.file_name().to_str().unwrap_or_default();
            !no_mistakes::codebase::ts_source::is_skipped_dir(name)
                && !skip_directories.iter().any(|skip| skip == name)
        });
    for _ in builder.build() {}
    drop(builder);

    let mut matches = Arc::try_unwrap(matches)
        .expect("preserved-root walker should release its match collector")
        .into_inner()
        .expect("preserved-root match lock should not be poisoned");
    matches.sort();
    matches.dedup();
    roots.extend(matches);
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

fn root_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-discovery")
        .join(path)
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
        rules: vec![
            no_mistakes::config::v2::schema::RuleDef {
                rule: "test-email-domain-policy".to_string(),
                projects: vec!["missing".to_string()],
                include: vec!["fixtures/**".to_string()],
                ..Default::default()
            },
            no_mistakes::config::v2::schema::RuleDef {
                rule: no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE.to_string(),
                projects: vec!["missing".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_eq!(
        include_preserved_roots(&root, &config, &[]),
        vec![root.join("fixtures")]
    );
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

/// Outside a git repository, `descendant_dirs_matching_suffix` falls back to an
/// ignore-aware walk since there is no git-visible file list to derive candidates.
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

#[test]
fn ignore_aware_fallback_prunes_gitignored_nested_suffix() {
    let root = root_fixture("preserved-roots-ignore-walk");
    let mut roots = Vec::new();

    // Call the non-Git fallback directly because this checked-in fixture lives
    // inside the repository that supplies the git-visible fast path.
    collect_descendant_dirs_matching_suffix(&root, &root, Path::new("components"), &[], &mut roots);

    assert_eq!(roots, vec![root.join("backend/components")]);
}
