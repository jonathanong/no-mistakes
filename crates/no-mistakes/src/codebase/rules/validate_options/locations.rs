use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn display_config_path(root: &Path, path: &Path) -> String {
    let root = normalize_path(root);
    let path = normalize_path(path);
    if path.starts_with(&root) {
        return relative_slash_path(&root, &path);
    }
    std::env::current_dir()
        .ok()
        .and_then(|cwd| relative_between(&normalize_path(&cwd), &path))
        .unwrap_or_else(|| relative_slash_path(&root, &path))
}

fn relative_between(base: &Path, target: &Path) -> Option<String> {
    let base_parts: Vec<_> = base.components().collect();
    let target_parts: Vec<_> = target.components().collect();
    let common = base_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return None;
    }
    let mut parts: Vec<String> = vec!["..".into(); base_parts.len() - common];
    parts.extend(
        target_parts[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    (!parts.is_empty()).then(|| parts.join("/"))
}
