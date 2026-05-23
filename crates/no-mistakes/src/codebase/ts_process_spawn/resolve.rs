fn resolve_entry_file_from_shell(
    cmd: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
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

    resolve_entry_file(file_token, cwd, file_path, root)
}

/// Resolve a file path token against `cwd ?? config_dir ?? root`.
fn resolve_entry_file(
    token: &str,
    cwd: Option<&str>,
    file_path: &Path,
    root: &Path,
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

    let candidate = base.join(token);
    if candidate.is_file() {
        return Some(candidate);
    }

    // Also try relative to root
    let root_candidate = root.join(token);
    if root_candidate.is_file() {
        return Some(root_candidate);
    }

    None
}

fn looks_like_file_path(token: &str) -> bool {
    let has_file_shape = token.contains('.') || token.contains('/');
    let is_flag = token.starts_with('-');
    let is_url = token.starts_with("http");
    has_file_shape && !is_flag && !is_url
}

