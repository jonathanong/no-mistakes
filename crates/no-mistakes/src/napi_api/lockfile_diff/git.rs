use std::path::{Path, PathBuf};

pub(super) fn detect_lockfiles_from_head(
    git_root: &Path,
    head: &str,
    root: &Path,
) -> std::io::Result<Vec<PathBuf>> {
    let candidates = [
        "pnpm-lock.yaml",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "bun.lock",
    ];
    let root_rel = root
        .strip_prefix(git_root)
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    let mut lockfiles = Vec::new();
    for name in candidates {
        let rel = if root_rel.is_empty() {
            name.to_string()
        } else {
            format!("{root_rel}/{name}")
        };
        if git_show_file(git_root, head, &rel)?.is_some() {
            lockfiles.push(root.join(name));
        }
    }
    Ok(lockfiles)
}

pub(super) fn find_git_root(dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir);
    let output = crate::invocation::command_output(&mut command)?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)
            .ok()
            .map(|value| PathBuf::from(value.trim())))
    } else {
        Ok(None)
    }
}

pub(super) fn git_ref_exists(root: &Path, git_ref: &str) -> std::io::Result<bool> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root);
    Ok(crate::invocation::command_output(&mut command)?
        .status
        .success())
}

pub(super) fn git_show_file(
    root: &Path,
    git_ref: &str,
    rel_path: &str,
) -> std::io::Result<Option<String>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["show", &format!("{git_ref}:{rel_path}")])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten())
}
