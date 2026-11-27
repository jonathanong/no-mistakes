const LOCKFILE_NAMES: &[&str] = no_mistakes::codebase::lockfile::LOCKFILE_BASENAMES;

fn detect_lockfiles_in_root(root: &Path, visible_paths: &[PathBuf]) -> Vec<PathBuf> {
    LOCKFILE_NAMES
        .iter()
        .map(|name| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(name)))
        .filter(|candidate| {
            visible_paths
                .iter()
                .any(|path| no_mistakes::codebase::ts_resolver::normalize_path(path) == *candidate)
        })
        .collect()
}

fn find_git_root(dir: &Path) -> Result<Option<PathBuf>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir);
    let output = no_mistakes::invocation::command_output(&mut command)?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)
            .ok()
            .map(|value| PathBuf::from(value.trim())))
    } else {
        Ok(None)
    }
}

fn git_ref_exists(root: &Path, git_ref: &str) -> Result<bool> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root);
    Ok(no_mistakes::invocation::command_output(&mut command)?
        .status
        .success())
}

fn git_content(git_root: &Path, git_ref: &str, rel: &str) -> Result<Option<Option<String>>> {
    match lockfile::git_show_file(git_root, git_ref, rel) {
        Ok(Some(content)) => Ok(Some(Some(content))),
        Ok(None) if git_ref_exists(git_root, git_ref)? => Ok(Some(None)),
        Ok(None) => {
            eprintln!("warning: could not retrieve {rel} at ref {git_ref}");
            Ok(None)
        }
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            eprintln!("warning: `{rel}` at ref `{git_ref}` is not valid UTF-8; skipping it");
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}
