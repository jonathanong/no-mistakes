const LOCKFILE_NAMES: &[&str] = &[
    "pnpm-lock.yaml",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "bun.lock",
];

fn detect_lockfiles_from_head(git_root: &Path, head: &str, root: &Path) -> Vec<PathBuf> {
    let prefix = root
        .strip_prefix(git_root)
        .unwrap_or(std::path::Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    LOCKFILE_NAMES
        .iter()
        .filter(|name| {
            let rel = if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") };
            git_show_file(git_root, head, &rel).is_some()
        })
        .map(|name| root.join(name))
        .collect()
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

fn find_git_root(dir: &Path) -> Option<PathBuf> {
    let mut command = std::process::Command::new("git");
    command.args(["rev-parse", "--show-toplevel"]).current_dir(dir);
    let output = no_mistakes::invocation::command_output(&mut command).ok()?;
    if output.status.success() {
        Some(PathBuf::from(String::from_utf8(output.stdout).ok()?.trim()))
    } else {
        None
    }
}

fn git_ref_exists(root: &Path, git_ref: &str) -> bool {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root);
    no_mistakes::invocation::command_output(&mut command)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git_content_or_empty(git_root: &Path, git_ref: &str, rel: &str) -> Option<String> {
    match git_show_file(git_root, git_ref, rel) {
        Some(content) => Some(content),
        None if git_ref_exists(git_root, git_ref) => Some(String::new()),
        None => None,
    }
}

fn git_show_file(root: &Path, git_ref: &str, rel_path: &str) -> Option<String> {
    let mut command = std::process::Command::new("git");
    command
        .args(["show", &format!("{git_ref}:{rel_path}")])
        .current_dir(root);
    let output = no_mistakes::invocation::command_output(&mut command).ok()?;
    output.status.success().then(|| String::from_utf8(output.stdout).ok()).flatten()
}
