use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn seeds(files: &[String], tracked: &BTreeSet<&str>) -> (Vec<String>, Vec<String>) {
    if files.is_empty() {
        let present = ["sgconfig.yml", "sgconfig.yaml"]
            .into_iter()
            .filter(|name| tracked.contains(name))
            .map(str::to_string)
            .collect();
        return (present, Vec::new());
    }
    let mut configs = Vec::new();
    let mut missing = Vec::new();
    for seed in files {
        if is_yaml(seed) {
            if tracked.contains(seed.as_str()) {
                configs.push(seed.clone());
            } else {
                missing.push(seed.clone());
            }
        } else {
            let prefix = format!("{}/", seed.trim_end_matches('/'));
            let found: Vec<_> = tracked
                .iter()
                .filter(|relative| relative.starts_with(&prefix) && is_yaml(relative))
                .map(|relative| (*relative).to_string())
                .collect();
            if found.is_empty() {
                missing.push(seed.clone());
            }
            configs.extend(found);
        }
    }
    (configs, missing)
}

pub(super) fn child_prefix(config_rel: &str, dir: &str) -> Option<String> {
    if dir.starts_with(['/', '\\']) || dir.split(['/', '\\']).any(|part| part == "..") {
        return None;
    }
    let dir = dir.trim_start_matches("./").trim_end_matches('/');
    if dir.is_empty() {
        return None;
    }
    let parent = Path::new(config_rel).parent().unwrap_or(Path::new(""));
    let joined = parent.join(dir);
    Some(format!("{}/", joined.to_string_lossy().replace('\\', "/")))
}

pub(super) fn is_yaml(relative: &str) -> bool {
    matches!(
        Path::new(relative).extension().and_then(|ext| ext.to_str()),
        Some("yml" | "yaml")
    )
}
