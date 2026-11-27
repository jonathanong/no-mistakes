use std::path::{Path, PathBuf};

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
