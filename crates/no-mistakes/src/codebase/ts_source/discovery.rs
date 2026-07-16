pub const TS_JS_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "mts", "cjs", "cts", "ts", "tsx"];

pub const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    ".git",
    ".next",
    "coverage",
    "fixtures",
    "target",
    "build",
];

pub fn is_skipped_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

/// Walk all non-ignored files under `root`.
///
/// Uses the `ignore` crate so `.gitignore` rules and hidden directories are
/// excluded, except `.github` because CI workflow analysis needs those files
/// when no `.git` metadata is available. `node_modules` is also always excluded
/// as a safety net for repos where it is not gitignored.
///
/// `extra_skip` is an optional list of additional directory names to prune
/// (e.g. `config.filesystem.skip_directories`).
pub fn walk_files(root: &Path, extra_skip: &[String]) -> Vec<PathBuf> {
    let extra_skip: HashSet<String> = extra_skip.iter().cloned().collect();

    let mut files = walk_non_ignored_files(root, &extra_skip);
    files.extend(walk_github_workflow_files(root, &extra_skip));
    sort_dedup(files)
}

fn sort_dedup(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    if !files.is_empty() {
        files.sort();
        files.dedup();
    }
    files
}

fn walk_non_ignored_files(root: &Path, extra_skip: &HashSet<String>) -> Vec<PathBuf> {
    let entry_extra_skip = extra_skip.clone();
    let file_extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
    WalkBuilder::new(root)
        .hidden(true)
        // Outside a Git checkout, `ignore` defaults to requiring `.git`
        // metadata before it applies `.gitignore`. Discovery must keep the
        // same ignore semantics for source archives and ad-hoc directories.
        .require_git(false)
        .filter_entry(move |e| {
            let within_deadline = crate::invocation::check_timeout().is_ok();
            let name = e.file_name().to_str().unwrap_or("");
            within_deadline
                && !(e.depth() > 0
                && e.file_type().is_some_and(|ft| ft.is_dir())
                && (SKIP_DIRS.contains(&name) || entry_extra_skip.contains(name)))
        })
        .build()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .map(|e| normalize_discovery_path(e.path()))
        .filter(|path| !is_under_skipped_dir(root, path, &file_extra_skip))
        .collect()
}

fn walk_github_workflow_files(root: &Path, extra_skip: &HashSet<String>) -> Vec<PathBuf> {
    let github = root.join(".github");
    if !std::fs::symlink_metadata(github)
        .ok()
        .is_some_and(|metadata| metadata.file_type().is_dir())
    {
        return Vec::new();
    }

    let extra_skip = extra_skip.clone();
    let filter_root = root.to_path_buf();
    let file_root = root.to_path_buf();
    WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(move |e| {
            let within_deadline = crate::invocation::check_timeout().is_ok();
            let name = e.file_name().to_str().unwrap_or("");
            let allowed_directory = !(e.depth() > 0
                && e.file_type().is_some_and(|ft| ft.is_dir())
                && (SKIP_DIRS.contains(&name) || extra_skip.contains(name)));
            within_deadline
                && allowed_directory
                && (e.depth() == 0 || is_github_workflows_prefix(&filter_root, e.path()))
        })
        .build()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .filter(|e| is_github_workflows_prefix(&file_root, e.path()))
        .map(|e| normalize_discovery_path(e.path()))
        .collect()
}

fn is_github_workflows_prefix(root: &Path, path: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let mut components = rel
        .components()
        .filter_map(|component| component.as_os_str().to_str().filter(|name| *name != "."));

    if components.next() != Some(".github") {
        return false;
    }

    matches!(components.next(), None | Some("workflows"))
}

/// Return git-visible files as absolute paths. Falls back to the ignore-based
/// walker outside git repositories so unit tests and ad-hoc directories still
/// behave sensibly.
pub fn discover_files(root: &Path, extra_skip: &[String]) -> Vec<PathBuf> {
    let visible_paths = discover_visible_paths(root);
    discover_files_from_visible(root, extra_skip, &visible_paths)
}

/// Filter a request-scoped visible-path snapshot using the standard source
/// discovery exclusions without walking the filesystem again.
pub fn discover_files_from_visible(
    root: &Path,
    extra_skip: &[String],
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let root = normalize_discovery_path(root);
    let extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
    visible_paths
        .iter()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .map(|path| normalized_visible_path(path))
        .filter(|path| path.starts_with(&root))
        .filter(|path| !is_under_skipped_dir(&root, path, &extra_skip))
        .collect()
}

fn normalized_visible_path(path: &Path) -> PathBuf {
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    }) {
        normalize_discovery_path(path)
    } else {
        path.to_path_buf()
    }
}

pub fn discover_source_files(root: &Path, extra_skip: &[String]) -> Vec<PathBuf> {
    let visible_paths = discover_visible_paths(root);
    discover_source_files_from_visible(root, extra_skip, &visible_paths)
}

/// Return TypeScript/JavaScript files from a request-scoped visible snapshot.
pub fn discover_source_files_from_visible(
    root: &Path,
    extra_skip: &[String],
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    discover_files_from_visible(root, extra_skip, visible_paths)
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| TS_JS_EXTENSIONS.contains(&ext))
        })
        .collect()
}

include!("discovery/helpers.rs");
include!("discovery/visible.rs");
include!("discovery/path_views.rs");
