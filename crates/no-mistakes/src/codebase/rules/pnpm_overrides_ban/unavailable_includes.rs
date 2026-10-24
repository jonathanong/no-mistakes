use super::{RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    source: &str,
    sources: &SourceStore,
    files: &[PathBuf],
) -> (Vec<RuleFinding>, Vec<String>) {
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return (Vec::new(), Vec::new());
    };
    let Some(includes) = yaml.get("include").and_then(serde_yaml::Value::as_mapping) else {
        return (Vec::new(), Vec::new());
    };
    let mut findings = Vec::new();
    let mut failed_sources = Vec::new();
    for (key, grant) in includes {
        let Some(key) = key.as_str() else { continue };
        let affects_overrides = grant.as_str() == Some("*")
            || grant
                .as_sequence()
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("overrides")));
        if !affects_overrides || !(key.starts_with("./") || key.starts_with("../")) {
            continue;
        }
        if !(key.ends_with(".yaml") || key.ends_with(".yml")) || key.contains(['*', '?', '[', ']'])
        {
            continue;
        }
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(key));
        if !path.starts_with(root)
            || !files
                .iter()
                .any(|file| crate::codebase::ts_resolver::normalize_path(file) == path)
        {
            continue;
        }
        let error = match sources.read_path(&path) {
            Err(error) => Some(error.to_string()),
            Ok(text) => match serde_yaml::from_str::<serde_yaml::Value>(&text) {
                Err(error) => Some(format!("failed to parse YAML: {error}")),
                Ok(value) if !value.is_mapping() => {
                    Some("included overrides YAML must be a mapping".to_owned())
                }
                Ok(_) => None,
            },
        };
        if let Some(error) = error {
            failed_sources.push(key.to_owned());
            findings.push(super::super::manifest_analysis::unavailable(
                root, &path, RULE_ID, error,
            ));
        }
    }
    (findings, failed_sources)
}
