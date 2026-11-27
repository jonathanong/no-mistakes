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
        let rel = git_lockfile_path(git_root, &path);
        if git_path_exists(git_root, head, &rel)? || git_path_exists(git_root, base, &rel)? {
            lockfiles.push(path);
        }
    }
    Ok(lockfiles)
}

pub fn git_lockfile_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    if cfg!(windows) {
        relative.replace('\\', "/")
    } else {
        relative.into_owned()
    }
}

pub fn git_show_file(root: &Path, reference: &str, rel: &str) -> io::Result<Option<String>> {
    let mut command = Command::new("git");
    command
        .args(["cat-file", "blob", &format!("{reference}:{rel}")])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    if output.status.success() {
        String::from_utf8(output.stdout).map(Some).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("`{rel}` at ref `{reference}` is not valid UTF-8: {error}"),
            )
        })
    } else if git_path_exists(root, reference, rel)? {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("`{rel}` at ref `{reference}` could not be read as a file"),
        ))
    } else {
        Ok(None)
    }
}

fn git_path_exists(root: &Path, reference: &str, rel: &str) -> io::Result<bool> {
    let mut command = Command::new("git");
    command
        .args([
            "--literal-pathspecs",
            "ls-tree",
            "--full-tree",
            "-z",
            reference,
            "--",
            rel,
        ])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    Ok(output.status.success() && !output.stdout.is_empty())
}
