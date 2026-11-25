use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn display_config_path(root: &Path, config_path: Option<&Path>) -> Option<String> {
    let path = config_path?;
    let normalized_root = normalize_path(root);
    let normalized_path = normalize_path(path);
    if normalized_path.starts_with(&normalized_root) {
        let rel = relative_slash_path(&normalized_root, &normalized_path);
        return (!rel.is_empty()).then_some(rel);
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(rel) = relative_between(&normalize_path(&cwd), &normalized_path) {
            return Some(rel);
        }
    }
    let rel = relative_slash_path(&normalized_root, &normalized_path);
    (!rel.is_empty()).then_some(rel)
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
