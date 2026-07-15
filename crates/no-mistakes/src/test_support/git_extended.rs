use std::path::Path;

pub(crate) fn git_add_force(root: &Path, paths: &[&str]) {
    let mut args = vec!["add", "-f", "--"];
    args.extend_from_slice(paths);
    super::git::run_git(root, &args);
}

pub(crate) fn git_config(root: &Path, key: &str, value: &Path) {
    super::git::run_git(root, &["config", key, value.to_str().unwrap()]);
}
