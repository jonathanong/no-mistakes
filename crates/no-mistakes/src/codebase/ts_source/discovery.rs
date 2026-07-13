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

    let mut files = walk_non_ignored_files(root, &extra_skip, &[]);
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

fn walk_non_ignored_files(
    root: &Path,
    extra_skip: &HashSet<String>,
    preserved_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let entry_extra_skip = extra_skip.clone();
    let file_extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
    let preserved_roots = preserved_roots.to_vec();
    let filter_preserved_roots = preserved_roots.clone();
    WalkBuilder::new(root)
        .hidden(true)
        .filter_entry(move |e| {
            let path = normalize_discovery_path(e.path());
            let name = e.file_name().to_str().unwrap_or("");
            if e.depth() > 0
                && e.file_type().is_some_and(|ft| ft.is_dir())
                && (SKIP_DIRS.contains(&name) || entry_extra_skip.contains(name))
            {
                return filter_preserved_roots
                    .iter()
                    .any(|root| root.starts_with(&path));
            }
            true
        })
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .map(|e| normalize_discovery_path(e.path()))
        .filter(|path| {
            !is_under_skipped_dir(root, path, &file_extra_skip)
                || preserved_roots.iter().any(|root| {
                    path.starts_with(root) && !is_under_skipped_dir(root, path, &file_extra_skip)
                })
        })
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
        .filter_entry(move |e| {
            let name = e.file_name().to_str().unwrap_or("");
            if e.depth() > 0
                && e.file_type().is_some_and(|ft| ft.is_dir())
                && (SKIP_DIRS.contains(&name) || extra_skip.contains(name))
            {
                return false;
            }
            e.depth() == 0 || is_github_workflows_prefix(&filter_root, e.path())
        })
        .build()
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

/// Return all tracked and untracked non-ignored files under `root`.
///
/// This follows the repo-wide convention that git is the source of truth for
/// file discovery: tracked files plus untracked files that are not hidden by
/// `.gitignore`. The result is repo-relative, sorted, and deduplicated.
pub fn git_visible_files(root: &Path) -> Option<Vec<String>> {
    git_ls_files(root)
}

/// Return git-visible files as absolute paths. Falls back to the ignore-based
/// walker outside git repositories so unit tests and ad-hoc directories still
/// behave sensibly.
pub fn discover_files(root: &Path, extra_skip: &[String]) -> Vec<PathBuf> {
    discover_files_from_git_files(root, extra_skip, None)
}

/// Same as [`discover_files`], but lets a caller reuse a git-visible file list it
/// already fetched via [`git_visible_files`] instead of spawning `git ls-files`
/// again for the same root. Pass `git_files: None` to fetch it internally —
/// identical to calling [`discover_files`] directly. Pass `Some(files)` when the
/// caller already has the list from an earlier call within the same invocation
/// (e.g. `no-mistakes check` discovering the same root twice with different
/// skip-directory filters).
pub fn discover_files_from_git_files(
    root: &Path,
    extra_skip: &[String],
    git_files: Option<&[String]>,
) -> Vec<PathBuf> {
    let root = normalize_discovery_path(root);
    let fetched;
    let files: Option<&[String]> = match git_files {
        Some(files) => Some(files),
        None => {
            fetched = git_visible_files(&root);
            fetched.as_deref()
        }
    };
    match files {
        Some(files) => {
            let extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
            files
                .iter()
                .map(|rel| normalize_discovery_path(&root.join(rel)))
                .filter(|p| p.exists())
                .filter(|p| !is_under_skipped_dir(&root, p, &extra_skip))
                .collect()
        }
        None => walk_files(&root, extra_skip),
    }
}

pub fn discover_source_files(root: &Path, extra_skip: &[String]) -> Vec<PathBuf> {
    discover_files(root, extra_skip)
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| TS_JS_EXTENSIONS.contains(&ext))
        })
        .collect()
}

pub fn discover_with_extensions(
    root: &Path,
    extra_skip: &[String],
    extensions: &[&str],
) -> Vec<PathBuf> {
    discover_files(root, extra_skip)
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| extensions.contains(&ext))
        })
        .collect()
}

pub fn discover_with_basenames(
    root: &Path,
    extra_skip: &[String],
    basenames: &[&str],
) -> Vec<PathBuf> {
    discover_files(root, extra_skip)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| basenames.contains(&n))
        })
        .collect()
}

pub fn relative_slash_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn line_number(source: &str, start: u32) -> usize {
    byte_offset_to_line(source, start as usize) as usize
}
