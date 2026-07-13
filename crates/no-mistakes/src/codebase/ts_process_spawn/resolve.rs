pub(crate) fn resolve_entry_file_from_shell(
    cmd: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
) -> Option<PathBuf> {
    resolve_entry_file_from_shell_inner(cmd, cwd, file_path, root, None)
}

pub(crate) fn resolve_entry_file_from_shell_from_visible(
    cmd: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Option<PathBuf> {
    resolve_entry_file_from_shell_inner(cmd, cwd, file_path, root, Some(visible_files))
}

fn resolve_entry_file_from_shell_inner(
    cmd: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    let file_token = tokens
        .iter()
        .skip_while(|t| {
            // Skip env var assignments like VAR=value or VAR=
            if t.contains('=') {
                return true;
            }
            // Skip runtime/tool prefixes
            matches!(
                **t,
                "node" | "tsx" | "npx" | "pnpm" | "npm" | "yarn" | "bunx" | "bun" | "run"
            )
        })
        .find(|t| looks_like_file_path(t))?;

    resolve_entry_file_inner(file_token, cwd, file_path, root, visible_files)
}

/// Resolve a file path token against `cwd ?? config_dir ?? root`.
pub(crate) fn resolve_entry_file(
    token: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
) -> Option<PathBuf> {
    resolve_entry_file_inner(token, cwd, file_path, root, None)
}

pub(crate) fn resolve_entry_file_from_visible(
    token: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Option<PathBuf> {
    resolve_entry_file_inner(token, cwd, file_path, root, Some(visible_files))
}

fn resolve_entry_file_inner(
    token: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    let base = if let Some(cwd) = cwd {
        let cwd_path = PathBuf::from(cwd);
        if cwd_path.is_absolute() {
            cwd_path
        } else {
            root.join(cwd_path)
        }
    } else {
        let fallback = root.to_path_buf();
        file_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(fallback)
    };

    let candidate = crate::codebase::ts_resolver::normalize_path(&base.join(token));
    if path_is_visible_file(&candidate, visible_files) {
        return Some(candidate);
    }

    // Also try relative to root
    let root_candidate = crate::codebase::ts_resolver::normalize_path(&root.join(token));
    if path_is_visible_file(&root_candidate, visible_files) {
        return Some(root_candidate);
    }

    None
}

fn path_is_visible_file(
    path: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> bool {
    visible_files.map_or_else(
        || path.is_file(),
        |visible| visible.contains(&crate::codebase::ts_resolver::normalize_path(path)),
    )
}

fn looks_like_file_path(token: &str) -> bool {
    let has_file_shape = token.contains('.') || token.contains('/');
    let is_flag = token.starts_with('-');
    let is_url = token.starts_with("http");
    has_file_shape && !is_flag && !is_url
}
