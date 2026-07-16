const LOCKFILE_NAMES: &[&str] = &[
    "pnpm-lock.yaml",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "bun.lock",
];

fn detect_lockfiles_from_head(git_root: &Path, head: &str, root: &Path) -> Result<Vec<PathBuf>> {
    let prefix = root
        .strip_prefix(git_root)
        .unwrap_or(std::path::Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    let mut lockfiles = Vec::new();
    for name in LOCKFILE_NAMES {
        let rel = if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") };
        if git_show_file(git_root, head, &rel)?.is_some() {
            lockfiles.push(root.join(name));
        }
    }
    Ok(lockfiles)
}

fn detect_lockfiles_in_root(root: &Path, visible_paths: &[PathBuf]) -> Vec<PathBuf> {
    LOCKFILE_NAMES
        .iter()
        .map(|name| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(name)))
        .filter(|candidate| visible_paths.iter().any(|path| {
            no_mistakes::codebase::ts_resolver::normalize_path(path) == *candidate
        }))
        .collect()
}

fn find_git_root(dir: &Path) -> Result<Option<PathBuf>> {
    let mut command = std::process::Command::new("git");
    command.args(["rev-parse", "--show-toplevel"]).current_dir(dir);
    let output = no_mistakes::invocation::command_output(&mut command)?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).ok().map(|value| PathBuf::from(value.trim())))
    } else {
        Ok(None)
    }
}

fn git_ref_exists(root: &Path, git_ref: &str) -> Result<bool> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root);
    Ok(no_mistakes::invocation::command_output(&mut command)?.status.success())
}

fn git_content_or_empty(git_root: &Path, git_ref: &str, rel: &str) -> Result<Option<String>> {
    match git_show_file(git_root, git_ref, rel)? {
        Some(content) => Ok(Some(content)),
        None if git_ref_exists(git_root, git_ref)? => Ok(Some(String::new())),
        None => Ok(None),
    }
}

fn git_show_file(root: &Path, git_ref: &str, rel_path: &str) -> Result<Option<String>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["show", &format!("{git_ref}:{rel_path}")])
        .current_dir(root);
    let output = no_mistakes::invocation::command_output(&mut command)?;
    Ok(output.status.success().then(|| String::from_utf8(output.stdout).ok()).flatten())
}
