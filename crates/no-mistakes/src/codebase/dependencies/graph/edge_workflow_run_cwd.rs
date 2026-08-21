fn workflow_run_working_directory(
    root: &Path,
    workflow: &serde_yaml::Value,
    job: &serde_yaml::Value,
    step: &serde_yaml::Value,
) -> Option<PathBuf> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let raw = step
        .get("working-directory")
        .and_then(serde_yaml::Value::as_str)
        .or_else(|| default_run_working_directory(job))
        .or_else(|| default_run_working_directory(workflow));
    let Some(raw) = raw else {
        return Some(root);
    };
    if !is_static_path_token(raw) {
        return None;
    }
    let resolved = crate::codebase::ts_resolver::normalize_path(&root.join(raw));
    resolved.starts_with(&root).then_some(resolved)
}

fn default_run_working_directory(value: &serde_yaml::Value) -> Option<&str> {
    value
        .get("defaults")?
        .get("run")?
        .get("working-directory")?
        .as_str()
}
