use git2::{Repository, Tree};
use std::path::Path;

pub(super) fn normalize_entry(entry: &str) -> String {
    if entry.contains('/') {
        entry.trim_start_matches("./").to_string()
    } else {
        format!(".github/workflows/{entry}")
    }
}

pub(super) fn yaml_at(repo: &Repository, tree: &Tree<'_>, path: &str) -> Option<serde_yaml::Value> {
    let entry = tree.get_path(Path::new(path)).ok()?;
    let blob = entry.to_object(repo).ok()?.peel_to_blob().ok()?;
    serde_yaml::from_slice(blob.content()).ok()
}
