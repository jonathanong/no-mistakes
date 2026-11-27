use super::LOCKFILE_BASENAMES;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn detect_lockfiles_at_refs(
    git_root: &Path,
    root: &Path,
    base: &str,
    head: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut lockfiles = Vec::new();
    for name in LOCKFILE_BASENAMES {
        let path = root.join(name);
        let rel = path
            .strip_prefix(git_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if git_path_exists(git_root, head, &rel)? || git_path_exists(git_root, base, &rel)? {
            lockfiles.push(path);
        }
    }
    Ok(lockfiles)
}

pub fn git_show_file(root: &Path, reference: &str, rel: &str) -> io::Result<Option<String>> {
    let mut command = Command::new("git");
    command
        .args(["show", &format!("{reference}:{rel}")])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    output
        .status
        .success()
        .then(|| {
            String::from_utf8(output.stdout)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .transpose()
}

fn git_path_exists(root: &Path, reference: &str, rel: &str) -> io::Result<bool> {
    let mut command = Command::new("git");
    command
        .args(["cat-file", "-e", &format!("{reference}:{rel}")])
        .current_dir(root);
    Ok(crate::invocation::command_output(&mut command)?
        .status
        .success())
}
