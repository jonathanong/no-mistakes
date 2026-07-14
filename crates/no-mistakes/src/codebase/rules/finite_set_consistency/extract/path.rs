use super::*;

pub(in super::super) fn extract_path_regex_set(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<ExtractedSet> {
    let regex = Regex::new(&spec.pattern)?;
    let mut values = BTreeSet::new();
    for file in files {
        for rel in relative_paths_for_matching(root, file, target_roots) {
            let Some(captures) = regex.captures(&rel) else {
                continue;
            };
            let value = captures
                .name("value")
                .or_else(|| captures.get(1))
                .map(|capture| capture.as_str().to_string());
            values.extend(value);
        }
    }
    Ok(ExtractedSet {
        file: match spec.file.is_empty() {
            true => ".".to_string(),
            false => spec.file.clone(),
        },
        values,
    })
}

fn relative_paths_for_matching(root: &Path, file: &Path, target_roots: &[PathBuf]) -> Vec<String> {
    let mut paths = target_roots
        .iter()
        .filter(|target_root| file.starts_with(target_root))
        .map(|target_root| relative_slash_path(target_root, file))
        .collect::<Vec<_>>();
    let repo_rel = relative_slash_path(root, file);
    if !paths.contains(&repo_rel) {
        paths.push(repo_rel);
    }
    paths
}
